use crate::metrics::{BEACON_API_DURATION, RETRY_ATTEMPTS};
use crate::types::BlockProposerResponse;
use eyre::eyre;
use reqwest::Client;
use std::time::Instant;
use tokio::time::sleep;

pub struct BeaconApi {
    client: Client,
    endpoint: String,
    max_retries: usize,
    poll_interval: std::time::Duration,
}

impl BeaconApi {
    pub fn new(endpoint: String, max_retries: usize, poll_interval: std::time::Duration) -> Self {
        Self { client: Client::new(), endpoint, max_retries, poll_interval }
    }

    pub async fn get_block_proposer_with_retry(
        &self,
        timestamp: u64,
    ) -> eyre::Result<BlockProposerResponse> {
        let mut attempt = 0;
        while attempt < self.max_retries {
            RETRY_ATTEMPTS.with_label_values(&["beacon_api"]).inc();
            match self.get_block_proposer(timestamp).await {
                Ok(resp) => return Ok(resp),
                Err(_) => {
                    attempt += 1;
                    sleep(self.poll_interval).await;
                }
            }
        }
        Err(eyre!("max retries exceeded"))
    }

    pub async fn get_block_proposer(&self, timestamp: u64) -> eyre::Result<BlockProposerResponse> {
        let start_time = Instant::now();
        let url = format!("{}/bkit/v1/proof/block_proposer/t{}", self.endpoint, timestamp);

        let result = match self.client.get(&url).send().await?.error_for_status() {
            Ok(response) => {
                let duration = start_time.elapsed().as_secs_f64();
                BEACON_API_DURATION.observe(duration);
                response.json::<BlockProposerResponse>().await?
            }
            Err(e) => {
                RETRY_ATTEMPTS.with_label_values(&["beacon_api"]).inc();
                return Err(e.into());
            }
        };

        Ok(result)
    }
}
