use alloy::network::{Ethereum, EthereumWallet};
use alloy::primitives::{Bytes, FixedBytes};
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::providers::RootProvider;
use alloy::pubsub::PubSubFrontend;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BeaconBlockHeader {
    pub proposer_index: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockProposerResponse {
    pub beacon_block_header: BeaconBlockHeader,
    pub proposer_index_proof: Vec<FixedBytes<32>>,
    pub validator_pubkey: Bytes,
    pub validator_pubkey_proof: Vec<FixedBytes<32>>,
}

pub type MyProvider = FillProvider<
    JoinFill<
        JoinFill<
            alloy::providers::Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<PubSubFrontend>,
    PubSubFrontend,
    Ethereum,
>;
