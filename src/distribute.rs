use crate::beacon_api::BeaconApi;
use crate::config::get_config;
use crate::contract::DistributorContract::DistributorContractInstance;
use crate::types::{BlockProposerResponse, MyProvider};
use alloy::network::primitives::BlockTransactionsKind;
use alloy::primitives::TxHash;
use alloy::providers::Provider;
use std::sync::Arc;
use std::time::Duration;
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
                let result = distribute(provider, resp, ts).await;
                tracing::info!(?result, "Submitted tx");
            } else {
                tracing::info!(?ts, "timestamp not actionable (already distributed)");
            }
        }
        Err(err) => {
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
        Ok(result) => result.actionable,
        Err(err) => {
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
    let cfg = get_config();
    let contract = DistributorContractInstance::new(cfg.distributor, provider);
    let proposer_index =
        u64::from_str_radix(resp.beacon_block_header.proposer_index.trim_start_matches("0x"), 16)?;
    tracing::info!(?ts, "Submitting");
    let tx_hash = contract
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
        .await?;
    Ok(*tx_hash.tx_hash())
}
