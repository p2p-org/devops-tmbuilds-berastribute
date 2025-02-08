use crate::beacon_api::BeaconApi;
use crate::config::get_config;
use crate::distribute::poll_proof_and_distribute;
use crate::types::MyProvider;
use crate::utils::prompt_password;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::signers::local::LocalSigner;
use futures_util::StreamExt;
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

    pub async fn run(&self) -> eyre::Result<()> {
        let sub = self.provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        let provider = self.provider.clone();
        let fee_recipient = self.fee_recipient;
        let ba = self.beacon_api.clone();
        let fallback_delay = self.fallback_delay.clone();
        tracing::info!(?fee_recipient, "Starting distributor");

        let handle = tokio::spawn(async move {
            while let Some(header) = stream.next().await {
                let should_distribute = match fee_recipient {
                    None => true,
                    Some(addr) => addr == header.beneficiary,
                };
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
