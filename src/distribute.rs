use crate::beacon_api::BeaconApi;
use crate::config::get_config;
use crate::contract::DistributorContract::DistributorContractInstance;
use crate::metrics::{
    BEACON_API_ERRORS, CONTRACT_CALLS, DISTRIBUTION_ATTEMPTS, DISTRIBUTION_DURATION, GAS_USED,
};
use crate::types::{BlockProposerResponse, MyProvider};
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::TxHash;
use alloy::providers::Provider;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub async fn poll_proof_and_distribute(
    provider: Arc<MyProvider>,
    beacon_api: Arc<BeaconApi>,
    block_number: u64,
    fallback_delay: Option<Duration>,
) {
    let cfg = get_config();
    let target = block_number.into();
    let next_block = loop {
        if let Ok(Some(block)) = provider.get_block(target, BlockTransactionsKind::Hashes).await {
            break block;
        }
        sleep(cfg.block_poll_interval).await;
    };

    let ts = next_block.header.timestamp;
    match beacon_api.get_block_proposer_with_retry(ts).await {
        Ok(resp) => {
            if wait_and_fallback(provider.clone(), ts, fallback_delay).await {
                match distribute(provider, resp, ts).await {
                    Ok(hash) => {
                        DISTRIBUTION_ATTEMPTS.with_label_values(&["success"]).inc();
                        tracing::info!(?hash, "Submitted tx");
                    }
                    Err(e) => {
                        DISTRIBUTION_ATTEMPTS.with_label_values(&["failure"]).inc();
                        tracing::error!(?e, "Failed to distribute");
                    }
                }
            } else {
                DISTRIBUTION_ATTEMPTS.with_label_values(&["skipped"]).inc();
                tracing::info!(?ts, "timestamp not actionable (already distributed)");
            }
        }
        Err(err) => {
            BEACON_API_ERRORS.with_label_values(&["proposer_fetch"]).inc();
            tracing::error!(?err, "error fetching block proposer data");
        }
    }
}

async fn wait_and_fallback(provider: Arc<MyProvider>, ts: u64, delay: Option<Duration>) -> bool {
    let Some(delay) = delay else {
        return true;
    };
    sleep(delay).await;
    match DistributorContractInstance::new(get_config().distributor, provider)
        .isTimestampActionable(ts)
        .call()
        .await
    {
        Ok(result) => {
            CONTRACT_CALLS.with_label_values(&["isTimestampActionable", "success"]).inc();
            result.actionable
        }
        Err(err) => {
            CONTRACT_CALLS.with_label_values(&["isTimestampActionable", "failure"]).inc();
            tracing::error!(?ts, ?err, "error checking distributor.isTimestampActionable()");
            true // still attempt to distribute if read err
        }
    }
}

async fn distribute(
    provider: Arc<MyProvider>,
    resp: BlockProposerResponse,
    ts: u64,
) -> eyre::Result<TxHash> {
    let start_time = Instant::now();
    let cfg = get_config();
    let contract = DistributorContractInstance::new(cfg.distributor, provider.clone());
    let proposer_index =
        u64::from_str_radix(resp.beacon_block_header.proposer_index.trim_start_matches("0x"), 16)?;
    tracing::info!(?ts, "Submitting");

    let result = contract
        .distributeFor(
            ts,
            proposer_index,
            resp.validator_pubkey,
            resp.proposer_index_proof,
            resp.validator_pubkey_proof,
        )
        .chain_id(cfg.chain_id)
        .gas(1000000)
        .send()
        .await;

    let duration = start_time.elapsed().as_secs_f64();
    match result {
        Ok(tx_hash) => {
            DISTRIBUTION_DURATION.with_label_values(&["success"]).observe(duration);
            CONTRACT_CALLS.with_label_values(&["distributeFor", "success"]).inc();

            // Wait for transaction receipt to get gas used
            if let Ok(Some(receipt)) = provider.get_transaction_receipt(*tx_hash.tx_hash()).await {
                GAS_USED.observe(receipt.gas_used as f64);
            }

            Ok(*tx_hash.tx_hash())
        }
        Err(e) => {
            DISTRIBUTION_DURATION.with_label_values(&["failure"]).observe(duration);
            CONTRACT_CALLS.with_label_values(&["distributeFor", "failure"]).inc();
            Err(e.into())
        }
    }
}
