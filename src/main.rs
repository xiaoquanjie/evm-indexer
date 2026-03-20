mod config;
mod db;
mod models;
mod process;
mod record;
mod rpc;
mod scanner;

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::db::Repository;
use crate::record::Record;
use crate::rpc::Rpc;
use crate::scanner::historical::HistoricalScanner;
use crate::scanner::live::LiveScanner;
use config::AppConfig;

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

    info!(
        "Config loaded. DB={}, RPC={}",
        config.database.url, config.rpc.http_url
    );

    // 扫描历史
    // scan_history(config).await?;

    scan_live(config).await?;

    Ok(())
}

#[allow(unused)]
async fn scan_history(config: AppConfig) -> Result<()> {
    // ── Database connection pool ───────────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .map_err(|e| {
            error!("Failed to connect to database: {}", e);
            e
        })?;

    // 初始化repo
    let repo = Arc::new(Repository::new(pool));
    repo.get_sync_state().await?;

    // rpc
    let rpc = Arc::new(Rpc::new(config.rpc.clone()));

    // 历史扫描,保存数据
    let historical = HistoricalScanner::new(rpc.get_http_provider().await?);
    let record = Record::new(config.clone(), repo, historical);
    record.run().await.map_err(|e| {
        error!("Historical scanner failed: {}", e);
        e
    })?;

    Ok(())
}

async fn scan_live(config: AppConfig) -> Result<()> {
    // rpc
    let rpc = Arc::new(Rpc::new(config.rpc.clone()));

    // 实时扫描
    let live = LiveScanner::new(rpc.get_ws_provider().await?);

    let mut sub = live
        .subscribe_blocks()
        .concurrent(10)
        //.start_block_id(Some(10))
        .start_block_id(None)
        //.end_block_id(Some(40))
        //.end_block_id(None)
        //.end_block_id(Some(10484791+50))
        .build();

    let mut last_block: u64 = 0;
    loop {
        let h = match sub.recv().await {
            Err(e) => {
                println!("error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            },
            Ok(None) => break,
            Ok(Some(h)) => h,
        };

        if last_block == 0 || last_block <= h.number {
            last_block = h.number;
        } else {
            panic!("error block:{}", h.number);
        }
        println!("block {} head: {:?}", h.number, h.hash);
    }

    Ok(())
}
