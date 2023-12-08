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
use tower_http::trace;
use tracing::{info, Level};

mod app_error;
use app_error::AppError;

#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv::dotenv()?;

    let opt = Opt::from_args();

    info!("DATABASE_URL={}", opt.database_url);

    let pool = SqlitePool::connect(&opt.database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let app = axum::Router::new()
        .route("/", get(hello))
        .route("/subscriptions", get(get_subscriptions))
        .route("/subscribe", post(subscribe))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await?;

    info!("Start http server at {}.", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn hello() -> impl IntoResponse {
    "Hello!"
}

#[derive(Serialize, Deserialize)]
struct Keys {
    p256dh: String,
    auth: String,
}

#[derive(Serialize, Deserialize)]
struct SubscribeData {
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
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let subscriptions_raw = sqlx::query_as!(SubscriptionRaw, "SELECT id, data FROM subscriptions;")
        .fetch_all(&pool)
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
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<SubscribeData>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let payload_str = serde_json::to_string(&payload)?;

    let subscription = sqlx::query_as!(
        SubscriptionRaw,
        "INSERT INTO subscriptions (data) VALUES ($1) RETURNING *;",
        payload_str,
    )
    .fetch_one(&pool)
    .await?;

    info!("id created = {}", subscription.id);

    Ok((StatusCode::OK, Json(json!(subscription))))
}
