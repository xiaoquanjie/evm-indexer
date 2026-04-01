mod config;
mod db;
mod models;
mod process;
mod record;
mod rpc;
mod scanner;

use anyhow::Result;
use futures::{pin_mut, StreamExt};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use alloy::primitives::Address;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use alloy::sol_types::SolEvent;

use crate::db::Repository;
use crate::record::Record;
use crate::rpc::Rpc;
use crate::scanner::historical::HistoricalScanner;
use crate::scanner::live::LiveScanner;
use crate::scanner::subscription::WithSubOption;
use crate::scanner::Sub;
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
        "Config loaded. DB={}, HTTP={}, WS={}",
        config.database.url, config.rpc.http_url, config.rpc.ws_url
    );

    // 扫描历史
    // scan_history(config).await?;

    scan_live2(config).await?;

    //scan_events(config).await?;
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
        .subscribe()
        .concurrent(10)
        .start_block(Some(10564501 - 30))
        .end_block(Some(10564501 + 10))
        .build();

    let mut last_block: u64 = 0;
    loop {
        let h = match sub.recv().await {
            Err(e) => {
                println!("error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
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

async fn scan_live2(config: AppConfig) -> Result<()> {
    // rpc
    let rpc = Arc::new(Rpc::new(config.rpc.clone()));

    // 实时扫描
    let live = LiveScanner::new(rpc.get_ws_provider().await?);

    let sub = live
        .subscribe()
        .concurrent(10)
        .start_block(Some(10565933-30))
        //.end_block(Some(10564501+10))
        .build();

    let stream = sub.into_stream();
    pin_mut!(stream);

    while let Some(head) = stream.next().await {
        match head {
            Err(e) => {
                println!("error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
            Ok(head) => {
                println!("block {} head: {:?}", head.number, head.hash);
            }
        }
    }

    Ok(())
}

alloy::sol! {
    event RECORD(uint256 indexed, string);
    event NO(string indexed, string);
}

async fn scan_events(config: AppConfig) -> Result<()> {
    let rpc = Arc::new(Rpc::new(config.rpc.clone()));
    let scanner = scanner::event::EventScanner::new(rpc.get_ws_provider().await?);
    let stream = scanner
        .subscribe()
        //.event("RECORD(uint256,string)")
        .events(["RECORD(uint256,string)", "NO(string,string)"])
        .start_block(Some(10565755))
        .address(
            "0xc190F57d98Fc4F50EFD27Dd340F07Cd42D268F1A".parse::<Address>()?,
        )
        .build()
        .into_stream();

    pin_mut!(stream);
    while let Some(log) = stream.next().await {
        match log {
            Err(e) => {
                println!("error: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
            Ok(log) => {
                if log.topics()[0] == RECORD::SIGNATURE_HASH {
                    let record = RECORD::decode_log(&log.inner)?;
                    println!("block {:?} index: {:?} data:{:?}", log.block_number, log.log_index, (record._0, record._1.clone()));
                } else {
                    let no = NO::decode_log(&log.inner)?;
                    println!("block {:?} index: {:?} data:{:?}", log.block_number, log.log_index, (no._0, no._1.clone()));
                };
            }
        }
    }
    Ok(())
}

