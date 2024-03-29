use sqlx::{migrate::MigrateDatabase, SqlitePool};
use structopt::StructOpt;
use tracing::info;

type BoolDefaultTrue = bool;

#[derive(StructOpt, Debug)]
#[structopt(name = "env")]
struct Opt {
    #[structopt(long, env = "HOSTNAME", default_value = "127.0.0.1")]
    hostname: String,

    #[structopt(short, long, env = "PORT", default_value = "3000")]
    port: usize,

    #[structopt(
        long,
        env = "SECURE",
        about = "Server running https or not",
        default_value = "true"
    )]
    secure: BoolDefaultTrue,

    #[structopt(
        long,
        env = "ASSETS_DIR",
        about = "Dir of the builded web app",
        default_value = "web/dist"
    )]
    assets_dir: String,

    #[structopt(long, env = "DATABASE_URL", about = "sqlite:// path to the database")]
    database_url: String,

    #[structopt(long, env = "VAPID_PRIVATE_KEY")]
    vapid_private_key: String,

    #[structopt(long, env = "VAPID_PUBLIC_KEY")]
    vapid_public_key: String,
}

async fn create_db(database_url: &str) -> anyhow::Result<SqlitePool> {
    if !sqlx::Sqlite::database_exists(database_url).await? {
        sqlx::Sqlite::create_database(database_url).await?;
    }

    let pool = SqlitePool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv::dotenv().ok();
    let opt = Opt::from_args();

    let pool = create_db(&opt.database_url).await?;

    let app = api::api::create_app(api::api::AppConfig {
        assets_dir: opt.assets_dir,
        pool,
        database_url: opt.database_url,
        vapid_public_key: opt.vapid_public_key,
        vapid_private_key: opt.vapid_private_key,
        secure: opt.secure,
    })
    .await;

    match app {
        Ok(app) => {
            let listener = tokio::net::TcpListener::bind(format!("{}:{}", opt.hostname, opt.port))
                .await
                .expect("Failed to run tcp listener.");

            info!("Start http server at {}.", listener.local_addr()?);
            axum::serve(listener, app)
                .await
                .expect("Failed to serve app.");
        }
        Err(error) => {
            // TODO: match error.kind() { ... }
            panic!("Failed to serve app: {}.", error);
        }
    }

    Ok(())
}
