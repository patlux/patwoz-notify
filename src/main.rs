use axum::{response::IntoResponse, routing::post};
use tracing::{info, Level};
use tower_http::trace;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = axum::Router::new().route("/subscribe", post(subscribe)).layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await?;

    info!("Start http server at {}.", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn subscribe() -> impl IntoResponse {
    "Hey"
}
