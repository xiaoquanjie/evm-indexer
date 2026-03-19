mod config;
mod db;
mod models;
mod process;
mod rpc;
mod scanner;
mod record;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter};

use config::AppConfig;
use crate::db::Repository;
use crate::record::Record;
use crate::rpc::Rpc;
use crate::scanner::historical::HistoricalScanner;

#[tokio::main]
async fn main() -> Result<()> {
    // ── Logging setup ─────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("evm_indexer=info,sqlx=warn")),
        )
        .with_target(false)
        .compact()
        .init();

    info!("EVM Indexer starting...");

    // ── Configuration ─────────────────────────────────────────────────────────
    let config = AppConfig::load().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    info!("Config loaded. DB={}, RPC={}", config.database.url, config.rpc.http_url);

    // ── Database connection pool ───────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            e
        })?;

    let rpc = Arc::new(Rpc::new(config.rpc.clone()));
    let http = rpc.get_http_provider().await?;

    let repo = Arc::new(Repository::new(pool));
    repo.get_sync_state().await?;

    let historical = HistoricalScanner::new(http);
    let record = Record::new(config.clone(), repo, historical);

    // 历史扫描,保存数据
    record.run().await.map_err(|e| {
        error!("Historical scanner failed: {}", e);
        e
    })?;



    //
    // // ── Phase 2: Live subscription ────────────────────────────────────────────
    // info!("Switching to live subscription mode from block {}", last_block);
    // let live = LiveScanner::new(config.clone(), repo.clone());
    //
    // // Retry loop: reconnect on disconnect / error
    // loop {
    //     match live.run(last_block).await {
    //         Ok(()) => {
    //             info!("Live scanner exited cleanly, reconnecting in 5s...");
    //         }
    //         Err(e) => {
    //             error!("Live scanner error: {}. Reconnecting in 5s...", e);
    //         }
    //     }
    //     tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    // }

    Ok(())

}
