use crate::beacon_api::BeaconApi;
use crate::distribute::poll_proof_and_distribute;
use crate::types::MyProvider;
use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use alloy::signers::local::LocalSigner;
use futures_util::StreamExt;
use rpassword::read_password;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

pub struct Distributor {
    fee_recipient: Option<Address>,
    provider: Arc<MyProvider>,
    beacon_api: Arc<BeaconApi>,
}

impl Distributor {
    pub async fn new(
        fee_recipient: Option<Address>,
        wss_url: String,
        beacon_url: String,
        keystore: PathBuf,
        password: Option<String>,
    ) -> eyre::Result<Self> {
        let password = match password {
            Some(pwd) => pwd,
            None => {
                print!("Enter keystore password: ");
                std::io::stdout().flush()?;
                read_password()?
            }
        };
        let signer = LocalSigner::decrypt_keystore(keystore, password)?;
        let wallet = EthereumWallet::from(signer);
        let ws = WsConnect::new(wss_url);
        let provider = Arc::new(
            ProviderBuilder::new().with_recommended_fillers().wallet(wallet).on_ws(ws).await?,
        );
        let beacon_api = Arc::new(BeaconApi::new(beacon_url));
        Ok(Self { fee_recipient, provider, beacon_api })
    }

    pub async fn run(&self) -> eyre::Result<()> {
        let sub = self.provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream();

        let provider = self.provider.clone();
        let fee_recipient = self.fee_recipient;
        let ba = self.beacon_api.clone();
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
                    ));
                }
            }
        });

        handle.await?;

        Ok(())
    }
}
