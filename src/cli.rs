use alloy::primitives::Address;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Listen and call distributeFor
    Distributor {
        #[arg(short, long, help = "fee recipient to check for. distributes for all if empty")]
        fee_recipient: Option<Address>,

        #[arg(short, long)]
        wss_url: String,

        #[arg(short, long)]
        beacon_url: String,

        #[arg(short, long)]
        keystore: PathBuf,

        #[arg(short, long)]
        password: Option<String>,
    },

    /// Check beacon response
    Beacon {
        #[arg(short, long)]
        beacon_url: String,

        #[arg(short, long)]
        timestamp: u64,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "app", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}
