use anyhow::Result;
use clap::Parser;
use porch_node::{Config, Node, api, models::ModelSpec, network};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about = "Infinite Porch: an opt-in, governed peer node")]
struct Args {
    #[arg(long, default_value = "state")]
    data: PathBuf,
    #[arg(long)]
    alias: Option<String>,
    #[arg(long)]
    api: Option<String>,
    #[arg(long)]
    listen: Vec<String>,
    #[arg(long)]
    no_mdns: bool,
    #[arg(long)]
    mock_model: Option<String>,
    #[arg(long)]
    share_hash: bool,
    #[arg(long)]
    storage_quota: Option<u64>,
    #[arg(long)]
    ollama: Option<String>,
    #[arg(long, default_value = "apps/desktop/dist")]
    ui: PathBuf,
}
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "porch_node=info".into()),
        )
        .init();
    let a = Args::parse();
    let mut config = Config::load(&a.data)?;
    if let Some(alias) = a.alias {
        config.alias = alias;
    }
    if let Some(api) = a.api {
        config.api = api;
    }
    if let Some(ollama) = a.ollama {
        config.ollama_url = ollama;
    }
    if let Some(quota) = a.storage_quota {
        config.storage_quota = quota;
    }
    if a.no_mdns {
        config.mdns = false;
    }
    if a.share_hash {
        config.compute_hash = true;
    }
    if !a.listen.is_empty() {
        config.listen = a.listen;
    }
    if let Some(name) = a.mock_model {
        config.models.retain(|model| model.name != name);
        config.models.push(ModelSpec {
            name,
            provider: "mock".into(),
            exposed: true,
            version: Some("deterministic-v1".into()),
        });
    }
    let node = Node::open(&a.data, config)?;
    network::start(node.clone()).await?;
    tracing::info!(peer=%node.id,api=%node.config.read().unwrap().api,token_file=%a.data.join("api.token").display(),"node ready; operator token is stored locally");
    let provider = node.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            let _ = porch_node::models::scan(&provider).await;
        }
    });
    let background = node.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let _ = background.retry_outbox().await;
        }
    });
    api::serve(node, a.ui).await
}
