use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite, SqlitePool};
use tower_http::{services::ServeDir, trace};
use tracing::Level;

use crate::{
    notification::Notification, response::AppError, subscribe_data::SubscribeData,
    subscription::Subscription,
};

pub struct AppConfig {
    pub assets_dir: String,
    pub database_url: String,
    pub vapid_public_key: String,
    pub vapid_private_key: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    vapid_private_key: String,
    vapid_public_key: String,
}

pub async fn create_app(config: AppConfig) -> anyhow::Result<Router> {
    let pool = SqlitePool::connect(&config.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let app_state = AppState {
        pool,
        vapid_private_key: config.vapid_private_key,
        vapid_public_key: config.vapid_public_key,
    };

    let api = Router::new()
        .route("/public-key", get(get_public_key))
        .route("/subscriptions", get(get_subscriptions))
        .route("/subscribe", post(subscribe))
        .route("/send", post(send))
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
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions;"#
    )
    .fetch_all(&app_state.pool)
    .await
    .expect("Failed to query subscriptions.");

    Json(json!({
        "subscriptions": subscriptions,
    }))
}

async fn subscribe(
    State(app_state): State<AppState>,
    Json(subscribe_data): Json<SubscribeData>,
) -> Result<StatusCode, AppError> {
    let subscription = Subscription::find_by_subscribe_data(&app_state.pool, &subscribe_data)
        .await
        .ok();

    match subscription {
        Some(_) => Ok(StatusCode::NOT_MODIFIED),
        None => {
            sqlx::query!(
                r#"INSERT INTO subscriptions (data) VALUES ($1);"#,
                subscribe_data
            )
            .execute(&app_state.pool)
            .await?;
            Ok(StatusCode::OK)
        }
    }
}

#[derive(Deserialize)]
struct SendPayload {
    subscription: SubscribeData,
    notification: Option<Notification>,
}

async fn send(
    State(app_state): State<AppState>,
    Json(payload): Json<SendPayload>,
) -> Result<StatusCode, AppError> {
    let subscription =
        Subscription::find_by_subscribe_data(&app_state.pool, &payload.subscription).await?;

    let notification = payload.notification.unwrap_or(Notification {
        title: "Test".into(),
        body: "This is a test message.".into(),
    });

    notification
        .send(&app_state.vapid_private_key, &subscription)
        .await
        .expect("Failed to send notification.");

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SendToAllPayload {
    notification: Notification,
}

async fn send_to_all(
    State(app_state): State<AppState>,
    Json(payload): Json<SendToAllPayload>,
) -> Result<StatusCode, AppError> {
    let notification = payload.notification;

    let rows = sqlx::query_as!(
        Subscription,
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions;"#,
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

async fn send_to_all_query(
    State(app_state): State<AppState>,
    payload: Query<MessageQueryParams>,
) -> Result<StatusCode, AppError> {
    let notification: Notification = payload.0.into();

    let rows = sqlx::query_as!(
        Subscription,
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions;"#,
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

    use crate::{server::*, subscribe_data::Keys};

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
