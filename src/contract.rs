use alloy::sol;

sol! {
    #[sol(rpc)]
    contract DistributorContract {
        #[derive(Debug)]
        function distributeFor(
            uint64 nextTimestamp,
            uint64 proposerIndex,
            bytes calldata pubkey,
            bytes32[] calldata proposerIndexProof,
            bytes32[] calldata pubkeyProof
        );

        function isTimestampActionable(uint64 timestamp) external view returns (bool actionable);
    }
}
