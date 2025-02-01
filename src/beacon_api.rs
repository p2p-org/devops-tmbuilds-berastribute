use crate::config::get_config;
use crate::types::BlockProposerResponse;
use eyre::eyre;
use reqwest::Client;
use tokio::time::sleep;

pub struct BeaconApi {
    client: Client,
    endpoint: String,
}

impl BeaconApi {
    pub fn new(endpoint: String) -> Self {
        Self { client: Client::new(), endpoint }
    }

    pub async fn get_block_proposer_with_retry(
        &self,
        timestamp: u64,
    ) -> eyre::Result<BlockProposerResponse> {
        let cfg = get_config();
        let mut attempt = 0;
        while attempt < cfg.beacon_max_retries {
            match self.get_block_proposer(timestamp).await {
                Ok(resp) => return Ok(resp),
                Err(_) => {
                    attempt += 1;
                    sleep(cfg.beacon_poll_interval).await;
                }
            }
        }
        Err(eyre!("max retries exceeded"))
    }

    pub async fn get_block_proposer(&self, timestamp: u64) -> eyre::Result<BlockProposerResponse> {
        let url = format!("{}/bkit/v1/proof/block_proposer/t{}", self.endpoint, timestamp);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let proposer_data = response.json::<BlockProposerResponse>().await?;
        Ok(proposer_data)
    }
}
