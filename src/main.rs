use crate::beacon_api::BeaconApi;
use crate::cli::{App, Command};
use crate::config::get_config;
use crate::distributor::Distributor;
use clap::Parser;
use tracing_subscriber::fmt::format::{FmtSpan, Format};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod beacon_api;
mod cli;
mod config;
mod contract;
mod distribute;
mod distributor;
mod healthcheck;
mod types;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .event_format(Format::default())
        .with_span_events(FmtSpan::NONE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let cfg = get_config();

    match App::parse().command {
        Command::Distributor {
            fee_recipient,
            wss_url,
            beacon_url,
            keystore,
            password,
            backfill_blocks,
            fallback_mode,
        } => {
            let fallback_delay = if fallback_mode && backfill_blocks.is_none() {
                Some(cfg.fallback_wait_interval)
            } else {
                None
            };
            let ds = Distributor::new(
                fee_recipient,
                wss_url,
                beacon_url,
                keystore,
                password,
                fallback_delay,
            )
            .await?;
            if let Some(backfill_blocks) = backfill_blocks {
                ds.run_backfill(backfill_blocks).await?;
            } else {
                ds.run().await?;
            }
        }
        Command::Beacon { beacon_url, timestamp } => {
            let beacon_api = BeaconApi::new(beacon_url);
            let result = beacon_api.get_block_proposer(timestamp).await?;
            println!("{:#?}", result);
        }
    }

    Ok(())
}
