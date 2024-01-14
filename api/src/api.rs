use std::ops::Add;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    extract::{cookie::Cookie, CookieJar},
    headers::UserAgent,
    TypedHeader,
};
use cookie::time::{Duration, OffsetDateTime};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite};
use tower_http::{services::ServeDir, trace};
use tracing::Level;
use uuid::Uuid;

use database::{
    notification::Notification, subscribe_data::SubscribeData, subscription::Subscription,
};

use crate::response::AppError;

pub struct AppConfig {
    pub assets_dir: String,
    pub pool: Pool<Sqlite>,
    pub database_url: String,
    pub vapid_public_key: String,
    pub vapid_private_key: String,
    pub secure: bool,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    vapid_private_key: String,
    vapid_public_key: String,
    secure: bool,
}

pub async fn create_app(config: AppConfig) -> anyhow::Result<Router> {
    let app_state = AppState {
        pool: config.pool,
        vapid_private_key: config.vapid_private_key,
        vapid_public_key: config.vapid_public_key,
        secure: config.secure,
    };

    let api = Router::new()
        .route("/me", get(get_user_me))
        .route("/me/send", post(post_send_to_me))
        .route("/public-key", get(get_public_key))
        .route("/subscriptions", get(get_subscriptions))
        .route("/subscribe", post(subscribe))
        .route("/send-to-all", post(send_to_all))
        .route("/message", get(send_to_all_query));

    let app = Router::new()
        .nest("/api", api)
        .nest_service("/", ServeDir::new(config.assets_dir))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state);

    Ok(app)
}

#[derive(Serialize)]
struct PostAuthResponseBody {
    device_id: String,
}

// #[derive(FromRow)]
// struct Device {
//     id: String,
//     user_agent: String,
//     name: Option<String>,
// }

async fn get_user_me(
    State(app_state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    jar: CookieJar,
) -> Result<CookieJar, AppError> {
    let device_id = jar.get("session_id");

    let device_id = device_id
        .map(|x| x.value())
        .unwrap_or(&Uuid::new_v4().to_string())
        .to_owned();

    let record = sqlx::query!("SELECT id FROM device WHERE id = $1", device_id)
        .fetch_one(&app_state.pool)
        .await;

    let device_id = match record {
        Ok(record) => {
            // Device exists...do nothing
            record.id
        }
        Err(_) => {
            let user_agent = user_agent.to_string();

            sqlx::query!(
                "INSERT INTO device(id, user_agent) VALUES($1, $2) RETURNING id;",
                device_id,
                user_agent
            )
            .fetch_one(&app_state.pool)
            .await?
            .id
        }
    };

    Ok(jar.add(
        Cookie::build(("session_id", device_id))
            .http_only(true)
            .secure(app_state.secure)
            .expires(OffsetDateTime::now_utc().add(Duration::weeks(52)))
            .build(),
    ))
}

#[derive(Serialize)]
struct GetPublicKeyResponseBody {
    #[serde(rename = "vapidPublicKey")]
    vapid_public_key: String,
}

async fn get_public_key(State(app_state): State<AppState>) -> impl IntoResponse {
    Json(GetPublicKeyResponseBody {
        vapid_public_key: app_state.vapid_public_key,
    })
}

async fn get_subscriptions(State(app_state): State<AppState>) -> impl IntoResponse {
    let subscriptions = sqlx::query_as!(
        Subscription,
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>", device_id FROM subscription;"#
    )
    .fetch_all(&app_state.pool)
    .await
    .expect("Failed to query subscriptions.");

    Json(json!({
        "subscriptions": subscriptions,
    }))
}

// POST /subscribe
async fn subscribe(
    jar: CookieJar,
    State(app_state): State<AppState>,
    Json(subscribe_data): Json<SubscribeData>,
) -> Result<StatusCode, AppError> {
    let device_id = jar.get("session_id").context("Missing session_id")?.value();

    let subscription = Subscription::find_by_subscribe_data(&app_state.pool, &subscribe_data)
        .await
        .ok();

    match subscription {
        Some(_) => Ok(StatusCode::NOT_MODIFIED),
        None => {
            sqlx::query!(
                r#"DELETE FROM subscription WHERE device_id = $1"#,
                device_id,
            )
            .execute(&app_state.pool)
            .await?;

            sqlx::query!(
                r#"INSERT INTO subscription (data, device_id) VALUES ($1, $2);"#,
                subscribe_data,
                device_id
            )
            .execute(&app_state.pool)
            .await?;

            Ok(StatusCode::OK)
        }
    }
}

#[derive(Deserialize)]
struct SendToMePayload {
    notification: Option<Notification>,
}

// POST /send
async fn post_send_to_me(
    State(app_state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<SendToMePayload>,
) -> Result<StatusCode, AppError> {
    let device_id = jar
        .get("session_id")
        .context("No session id")?
        .value()
        .to_owned();

    let subscription = Subscription::new_with_device_id(&app_state.pool, device_id).await?;

    let notification = payload.notification.unwrap_or(Notification {
        title: "Test".into(),
        body: "This is a test message.".into(),
    });

    notification
        .send(&app_state.vapid_private_key, &subscription)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SendToAllPayload {
    notification: Notification,
}

// POST /send-to-all
async fn send_to_all(
    State(app_state): State<AppState>,
    Json(payload): Json<SendToAllPayload>,
) -> Result<StatusCode, AppError> {
    let notification = payload.notification;

    let rows = sqlx::query_as!(
        Subscription,
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>", device_id FROM subscription;"#,
    )
    .fetch(&app_state.pool)
    .map_ok(|subscription| {
        let notification = notification.clone();
        let sub = subscription.clone();
        let private_key = app_state.vapid_private_key.clone();
        tokio::spawn(async move {
            println!("Send notification to {}.", subscription.id);
            match &notification.send(&private_key, &sub).await {
                Ok(()) => {
                    println!("Send notification successfully to: {}", subscription.id);
                }
                Err(err) => {
                    println!(
                        r#"Sent notification failed to: {}. Reason: "{}"."#,
                        subscription.id, err
                    );
                }
            };
        })
    })
    .collect::<Vec<_>>();

    rows.await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct MessageQueryParams {
    title: String,
    message: String,
}

impl From<MessageQueryParams> for Notification {
    fn from(value: MessageQueryParams) -> Self {
        Notification {
            title: value.title,
            body: value.message,
        }
    }
}

// GET /message
async fn send_to_all_query(
    State(app_state): State<AppState>,
    payload: Query<MessageQueryParams>,
) -> Result<StatusCode, AppError> {
    let notification: Notification = payload.0.into();

    let rows = sqlx::query_as!(
        Subscription,
        r#"SELECT id, device_id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscription;"#,
    )
    .fetch(&app_state.pool)
    .map_ok(|subscription| {
        let notification = notification.clone();
        let sub = subscription.clone();
        let private_key = app_state.vapid_private_key.clone();
        tokio::spawn(async move {
            println!("Send notification to {}.", subscription.id);
            match &notification.send(&private_key, &sub).await {
                Ok(()) => {
                    println!("Send notification successfully to: {}", subscription.id);
                }
                Err(err) => {
                    println!(
                        r#"Sent notification failed to: {}. Reason: "{}"."#,
                        subscription.id, err
                    );
                }
            };
        })
    })
    .collect::<Vec<_>>();

    rows.await;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::env;

    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;

    use crate::{api::*, subscribe_data::Keys};

    #[tokio::test]
    async fn it_should_get_index() -> anyhow::Result<()> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();

        dotenv::dotenv()?;

        // from_args doesn't seem to exist?
        // let opt = Opt::from_args();

        let vapid_private_key = env::var("VAPID_PRIVATE_KEY")?;
        let vapid_public_key = env::var("VAPID_PUBLIC_KEY")?;

        let app = create_app(AppConfig {
            secure: false,
            assets_dir: "web/assets".into(),
            database_url: ":memory:".into(),
            vapid_public_key: vapid_public_key.clone(),
            vapid_private_key: vapid_private_key.clone(),
        })
        .await?;

        let server = TestServer::new(app).unwrap();

        assert_eq!(
            server.get("/api/subscriptions").await.text(),
            json!({ "subscriptions": [] }).to_string()
        );

        assert_eq!(
            server.get("/api/public-key").await.text(),
            json!({ "vapidPublicKey": &vapid_public_key }).to_string()
        );

        let subscription_data = SubscribeData {
            endpoint: "https://fcm.googleapis.com/fcm/send/dajXAsLXLTQ:APA91bEuLx2gWxPsSwGUarNuRv_amL-DYb4zxB7f_S5HJD5HLWmj_yA7207xMGTPjGt6JEw43a9v2gErMgTQKFWnQLMscgLerzMA53pjc29q2XPU9Zcp5eJmA49Duxbz7jI78olkQAGO".into(),
            keys: Keys {
                p256dh: "BMKQlz6BHaqg_50X-keDzECQscc72EFiYKfoBBH46eknEAiDqwafG6yj4yhbcEEdCV5wE_b6okIgLy_j5Yfi80E".into(),
                auth: "JaOBe4ogueg1zizYIR9fYQ".into()
            },
            expiration_time: None,
        };

        server
            .post("/api/subscribe")
            .json(&subscription_data)
            .await
            .assert_status(StatusCode::OK);

        server
            .post("/api/subscribe")
            .json(&subscription_data)
            .await
            .assert_status(StatusCode::NOT_MODIFIED);

        Ok(())
    }
}
