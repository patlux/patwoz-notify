use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Sqlite, SqlitePool};
use structopt::StructOpt;
use tower_http::{
    cors::{Any, CorsLayer},
    trace,
};
use tracing::{info, Level};

mod app_error;
use app_error::AppError;
use web_push::WebPushClient;
use web_push::{
    ContentEncoding, IsahcWebPushClient, VapidSignatureBuilder, WebPushMessageBuilder,
    URL_SAFE_NO_PAD,
};

#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    dotenv::dotenv()?;

    let opt = Opt::from_args();

    info!("DATABASE_URL={}", opt.database_url);

    let pool = SqlitePool::connect(&opt.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let app_state = AppState {
        pool,
        vapid_private_key: opt.vapid_private_key,
        vapid_public_key: opt.vapid_public_key,
    };

    let app = axum::Router::new()
        .route("/", get(hello))
        .route("/public-key", get(get_public_key))
        .route("/subscriptions", get(get_subscriptions))
        .route("/subscribe", post(subscribe))
        .route("/send", post(send))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await?;

    info!("Start http server at {}.", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> impl IntoResponse {
    "Hello!"
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

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct SubscriptionRaw {
    id: i64,
    data: String,
}

#[derive(Serialize, Deserialize)]
struct Subscription {
    id: i64,
    data: sqlx::types::Json<SubscribeData>,
}

async fn get_subscriptions(
    State(app_state): State<AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let subscriptions_raw = sqlx::query_as!(SubscriptionRaw, "SELECT id, data FROM subscriptions;")
        .fetch_all(&app_state.pool)
        .await?;

    let subscriptions: Vec<Subscription> = subscriptions_raw
        .into_iter()
        .filter_map(|row| {
            let data: SubscribeData = serde_json::from_str(&row.data).ok()?;

            Some(Subscription {
                id: row.id,
                data: sqlx::types::Json(data),
            })
        })
        .collect();

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
    let subscription_existing = sqlx::query_as!(
        SubscriptionRaw,
        "SELECT * FROM subscriptions WHERE json_extract(data, '$.keys.auth') == $1",
        payload.keys.auth
    )
    .fetch_optional(&app_state.pool)
    .await?;

    return match subscription_existing {
        Some(_data) => Ok(StatusCode::NOT_MODIFIED),
        None => {
            let payload_str = serde_json::to_string(&payload)?;

            sqlx::query_as!(
                SubscriptionRaw,
                "INSERT INTO subscriptions (data) VALUES ($1) RETURNING *;",
                payload_str,
            )
            .fetch_one(&app_state.pool)
            .await?;

            Ok(StatusCode::OK)
        }
    };
}

async fn send(
    State(app_state): State<AppState>,
    Json(payload): Json<SubscribeData>,
) -> Result<StatusCode, AppError> {
    sqlx::query_as!(
        SubscriptionRaw,
        "SELECT * FROM subscriptions WHERE json_extract(data, '$.keys.auth') == $1",
        payload.keys.auth
    )
    .fetch_one(&app_state.pool)
    .await?;

    let subscription_info = web_push::SubscriptionInfo::new(
        &payload.endpoint,
        &payload.keys.p256dh,
        &payload.keys.auth,
    );

    let sig_builder = VapidSignatureBuilder::from_base64(
        &app_state.vapid_private_key,
        URL_SAFE_NO_PAD,
        &subscription_info,
    )?
    .build()?;

    let mut builder = WebPushMessageBuilder::new(&subscription_info);

    let data = json!({
    "title": "Test",
    "body": "This is a test message."
    });
    let body = serde_json::to_string(&data)?;
    let bb = body.into_bytes();

    builder.set_payload(ContentEncoding::Aes128Gcm, &bb);
    builder.set_vapid_signature(sig_builder);

    let client = IsahcWebPushClient::new()?;

    //Finally, send the notification!
    client.send(builder.build()?).await?;

    Ok(StatusCode::NO_CONTENT)
}
