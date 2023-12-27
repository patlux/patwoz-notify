use structopt::StructOpt;
use tracing::info;

mod app_error;
mod notification;
mod server;
mod subscribe_data;
mod subscription;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let app = server::create_app(server::AppConfig {
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
