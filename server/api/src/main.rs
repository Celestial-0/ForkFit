use api::{app, config::Config, observability};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    observability::init();
    let config = Config::from_env()?;
    let bind_addr = config.bind_addr;
    let router = app::build_app(config).await?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    tracing::info!("api listening on http://{bind_addr}");
    axum::serve(listener, router).await?;
    Ok(())
}
