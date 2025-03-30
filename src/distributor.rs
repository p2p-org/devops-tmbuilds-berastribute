use crate::beacon_api::BeaconApi;
use crate::config::get_config;
use crate::distribute::poll_proof_and_distribute;
use crate::healthcheck::ping_healthcheck;
use crate::types::MyProvider;
use crate::utils::prompt_password;
use alloy::network::primitives::BlockTransactionsKind::Hashes;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::rpc::types::Header;
use alloy::signers::local::LocalSigner;
use futures_util::StreamExt;
use indicatif::ProgressBar;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct Distributor {
    fee_recipient: Option<Address>,
    provider: Arc<MyProvider>,
    beacon_api: Arc<BeaconApi>,
    fallback_delay: Option<Duration>,
}

impl Distributor {
    pub async fn new(
        fee_recipient: Option<Address>,
        wss_url: String,
        beacon_url: String,
        keystore: PathBuf,
        password: Option<String>,
        fallback_delay: Option<Duration>,
    ) -> eyre::Result<Self> {
        let cfg = get_config();
        let password = match password {
            Some(pwd) => pwd,
            None => {
                if let Some(cfg_pwd) = cfg.keystore_password.clone() {
                    if LocalSigner::decrypt_keystore(&keystore, &cfg_pwd).is_ok() {
                        cfg_pwd
                    } else {
                        prompt_password()?
                    }
                } else {
                    prompt_password()?
                }
            }
        };
        let signer = LocalSigner::decrypt_keystore(keystore, password)?;
        let wallet = EthereumWallet::from(signer);
        let ws = WsConnect::new(wss_url);
        let provider = Arc::new(
            ProviderBuilder::new().with_recommended_fillers().wallet(wallet).on_ws(ws).await?,
        );
        let beacon_api = Arc::new(BeaconApi::new(beacon_url));
        Ok(Self { fee_recipient, provider, beacon_api, fallback_delay })
    }

    pub async fn run_backfill(&self, blocks: u64) -> eyre::Result<()> {
        tracing::info!(fee_recipient=?self.fee_recipient, "Starting backfill distributor");
        let current_bn = self.provider.get_block_number().await?;
        let start = current_bn - blocks + 1;
        tracing::info!(?start, end=?current_bn, "Target range");

        let bar = ProgressBar::new(blocks);
        for bn in start..=current_bn {
            let healthcheck_id = get_config().healthcheck_id.clone();
            tokio::spawn(async move {
                ping_healthcheck(healthcheck_id.as_deref()).await;
            });

            if let Some(block) = self.provider.get_block(bn.into(), Hashes).await? {
                if check_if_target(&block.header, self.fee_recipient) {
                    poll_proof_and_distribute(
                        self.provider.clone(),
                        self.beacon_api.clone(),
                        block.header.number + 1,
                        Some(Duration::from_secs(0)), // force is_actionable check
                    )
                    .await;
                }
            }
            bar.inc(1);
        }
        bar.finish();
        Ok(())
    }

    pub async fn run(&self) -> eyre::Result<()> {
        let sub = self.provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        let provider = self.provider.clone();
        let fee_recipient = self.fee_recipient;
        let ba = self.beacon_api.clone();
        let fallback_delay = self.fallback_delay;
        tracing::info!(?fee_recipient, "Starting distributor");

        let handle = tokio::spawn(async move {
            while let Some(header) = stream.next().await {
                let healthcheck_id = get_config().healthcheck_id.clone();
                tokio::spawn(async move {
                    ping_healthcheck(healthcheck_id.as_deref()).await;
                });

                let should_distribute = check_if_target(&header, fee_recipient);
                tracing::info!(bn=?header.number, fee_recipient=?header.beneficiary, ?should_distribute, "Received block");
                if should_distribute {
                    tokio::spawn(poll_proof_and_distribute(
                        provider.clone(),
                        ba.clone(),
                        header.number + 1,
                        fallback_delay,
                    ));
                }
            }
        });

        handle.await?;
        Ok(())
    }
}

fn check_if_target(header: &Header, fee_recipient: Option<Address>) -> bool {
    match fee_recipient {
        None => true,
        Some(addr) => addr == header.beneficiary,
    }
}
