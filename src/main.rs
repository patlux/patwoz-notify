use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite, SqlitePool};
use structopt::StructOpt;
use tower_http::{services::ServeDir, trace};
use tracing::{info, Level};

mod app_error;
use app_error::AppError;
// use web_push::HyperWebPushClient;
use web_push::IsahcWebPushClient;
use web_push::WebPushClient;
use web_push::{ContentEncoding, VapidSignatureBuilder, WebPushMessageBuilder, URL_SAFE_NO_PAD};

#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
    #[structopt(long, env = "hostname", default_value = "0.0.0.0")]
    hostname: String,

    #[structopt(short, long, env = "PORT", default_value = "3000")]
    port: usize,

    #[structopt(long, env = "ASSETS_DIR", default_value = "web/dist")]
    assets_dir: String,

    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,

    #[structopt(long, env = "VAPID_PRIVATE_KEY")]
    vapid_private_key: String,

    #[structopt(long, env = "VAPID_PUBLIC_KEY")]
    vapid_public_key: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
    vapid_private_key: String,
    vapid_public_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let app = create_app(AppConfig {
        assets_dir: opt.assets_dir,
        database_url: opt.database_url,
        vapid_public_key: opt.vapid_public_key,
        vapid_private_key: opt.vapid_private_key,
    })
    .await
    .expect("Failed to create app.");

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", opt.hostname, opt.port))
        .await
        .expect("Failed to run tcp listener.");

    info!("Start http server at {}.", listener.local_addr()?);
    axum::serve(listener, app)
        .await
        .expect("Failed to serve app.");

    Ok(())
}

struct AppConfig {
    assets_dir: String,
    database_url: String,
    vapid_public_key: String,
    vapid_private_key: String,
}

async fn create_app(config: AppConfig) -> anyhow::Result<Router> {
    let pool = SqlitePool::connect(&config.database_url)
        .await
        .expect("Couldn't connect to database.");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations.");

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
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

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

async fn get_public_key(State(app_state): State<AppState>) -> impl IntoResponse {
    return (
        StatusCode::OK,
        Json(json!({ "vapidPublicKey": app_state.vapid_public_key })),
    );
}

#[derive(Serialize, Deserialize)]
struct Keys {
    p256dh: String,
    auth: String,
}

#[derive(Serialize, Deserialize)]
struct SubscribeData {
    #[serde(rename = "expirationTime")]
    expiration_time: Option<i64>,
    endpoint: String,
    keys: Keys,
}

impl SubscribeData {
    fn into_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)?)
    }
}

#[derive(Serialize, Deserialize)]
struct Subscription {
    id: i64,
    data: sqlx::types::Json<SubscribeData>,
}

async fn get_subscriptions(
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let subscriptions = sqlx::query_as!(
        Subscription,
        r#"SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions;"#
    )
    .fetch_all(&app_state.pool)
    .await
    .expect("Failed to query subscriptions.");

    println!("HELLO");

    Ok((
        StatusCode::OK,
        Json(json!({
        "subscriptions": subscriptions,
        })),
    ))
}

async fn subscribe(
    State(app_state): State<AppState>,
    Json(payload): Json<SubscribeData>,
) -> Result<StatusCode, AppError> {
    let subscription = sqlx::query_as!(
        Subscription,
        r#"
            SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions
            WHERE
                json_extract(data, '$.keys.auth') == $1 
                AND
                json_extract(data, '$.keys.p256dh') == $2
                AND
                json_extract(data, '$.endpoint') == $3
            LIMIT 1
        "#,
        payload.keys.auth,
        payload.keys.p256dh,
        payload.endpoint,
    )
    .fetch_optional(&app_state.pool)
    .await?;

    return match subscription {
        Some(_data) => Ok(StatusCode::NOT_MODIFIED),
        None => {
            let data = payload.into_json()?;
            sqlx::query!("INSERT INTO subscriptions (data) VALUES ($1);", data)
                .execute(&app_state.pool)
                .await?;
            Ok(StatusCode::OK)
        }
    };
}

async fn send(
    State(app_state): State<AppState>,
    Json(payload): Json<SubscribeData>,
) -> Result<StatusCode, AppError> {
    let subscription = sqlx::query_as!(
        Subscription,
        r#"
            SELECT id, data as "data: sqlx::types::Json<SubscribeData>" FROM subscriptions
            WHERE
                json_extract(data, '$.keys.auth') == $1 
                AND
                json_extract(data, '$.keys.p256dh') == $2
                AND
                json_extract(data, '$.endpoint') == $3
            LIMIT 1
        "#,
        payload.keys.auth,
        payload.keys.p256dh,
        payload.endpoint,
    )
    .fetch_one(&app_state.pool)
    .await
    .expect("Failed to query subscription.");

    let subscription_info = web_push::SubscriptionInfo::new(
        &subscription.data.endpoint,
        &subscription.data.keys.p256dh,
        &subscription.data.keys.auth,
    );

    let sig_builder = VapidSignatureBuilder::from_base64(
        &app_state.vapid_private_key,
        URL_SAFE_NO_PAD,
        &subscription_info,
    )?
    .build()
    .expect("Failed to create signature.");

    let mut builder = WebPushMessageBuilder::new(&subscription_info);

    let data = json!({
    "title": "Test",
    "body": "This is a test message."
    });
    let body = serde_json::to_string(&data).expect("Failed to serialize data.");
    let bb = body.into_bytes();

    builder.set_payload(ContentEncoding::Aes128Gcm, &bb);
    builder.set_vapid_signature(sig_builder);

    // let client = HyperWebPushClient::new();
    let client = IsahcWebPushClient::new()?;

    //Finally, send the notification!
    client
        .send(builder.build()?)
        .await
        .expect("Failed to send notification.");

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{create_app, Keys, SubscribeData};
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;

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

        let app = create_app(crate::AppConfig {
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
