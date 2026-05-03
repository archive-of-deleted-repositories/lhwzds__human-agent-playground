use anyhow::Result;
use hap_runtime::HumanAgentPlaygroundRuntime;
use human_agent_playground_backend::{build_api_router, runtime_config_from_env};
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,human_agent_playground_backend=debug")),
        )
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8790);
    let runtime = HumanAgentPlaygroundRuntime::new(runtime_config_from_env()).await?;
    let app = build_api_router(runtime, CancellationToken::new());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
