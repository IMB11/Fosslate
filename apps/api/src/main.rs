use fosslate_api::{app, app::AppState, config::Config, db};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    let setup_bootstrap = setup_bootstrap(&pool).await?;

    let addr = config.socket_addr();
    let listener = TcpListener::bind(addr).await?;
    let app = app::build(
        AppState::new(
            pool,
            &config,
            setup_bootstrap.setup_secret,
            setup_bootstrap.secrets_key,
        ),
        &config,
    );

    tracing::info!(%addr, "starting fosslate api");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

struct SetupBootstrap {
    setup_secret: String,
    secrets_key: String,
}

async fn setup_bootstrap(
    pool: &sqlx::PgPool,
) -> Result<SetupBootstrap, Box<dyn std::error::Error>> {
    let setup_completed = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        r#"
        SELECT completed_at
        FROM instance_setup
        WHERE id = 1
        "#,
    )
    .fetch_one(pool)
    .await?
    .is_some();

    let secrets_key = sqlx::query_scalar::<_, String>(
        r#"
        SELECT secrets_key
        FROM instance_setup
        WHERE id = 1
        "#,
    )
    .fetch_one(pool)
    .await?;

    let setup_secret = format!("fs_setup_{}", uuid::Uuid::new_v4());
    tracing::warn!(
        setup_secret = %setup_secret,
        setup_completed,
        "generated setup/admin code"
    );

    Ok(SetupBootstrap {
        setup_secret,
        secrets_key,
    })
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("fosslate_api=debug,tower_http=info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
