use alloy::primitives::{address, Address};
use config::Config as ConfigHelper;
use lazy_static::lazy_static;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_DISTRIBUTOR_ADDRESS: Address = address!("0x211bE45338B7C6d5721B5543Eb868547088Aca39");
const DEFAULT_BLOCK_POLL_INTERVAL: u64 = 250;

const DEFAULT_BEACON_POLL_INTERVAL: u64 = 250;
const DEFAULT_BEACON_MAX_RETRIES: usize = 12;

#[derive(Clone, Debug)]
pub struct Config {
    pub distributor: Address,
    pub block_poll_interval: Duration,
    pub beacon_poll_interval: Duration,
    pub beacon_max_retries: usize,
}

fn build_config() -> eyre::Result<Config> {
    let cfg = ConfigHelper::builder().add_source(config::File::with_name("config")).build()?;
    let distributor = cfg.get::<Address>("distributor").unwrap_or(DEFAULT_DISTRIBUTOR_ADDRESS);
    let block_poll_interval = Duration::from_millis(
        cfg.get::<u64>("block_poll_interval").unwrap_or(DEFAULT_BLOCK_POLL_INTERVAL),
    );
    let beacon_poll_interval = Duration::from_millis(
        cfg.get::<u64>("beacon_poll_interval").unwrap_or(DEFAULT_BEACON_POLL_INTERVAL),
    );
    let beacon_max_retries =
        cfg.get::<usize>("beacon_max_retries").unwrap_or(DEFAULT_BEACON_MAX_RETRIES);

    Ok(Config { distributor, block_poll_interval, beacon_poll_interval, beacon_max_retries })
}

lazy_static! {
    static ref CONFIG: Arc<Config> = {
        let config = build_config().expect("Failed to build config");
        Arc::new(config)
    };
}

pub fn get_config() -> Arc<Config> {
    Arc::clone(&CONFIG)
}
