use lazy_static::lazy_static;
use prometheus::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, Encoder, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge,
    TextEncoder,
};
use warp::Filter;

lazy_static! {
    // Distribution attempts
    pub static ref DISTRIBUTION_ATTEMPTS: IntCounterVec = register_int_counter_vec!(
        "berastribute_distribution_attempts_total",
        "Total number of distribution attempts",
        &["status"]
    ).unwrap();

    // Block processing metrics
    pub static ref BLOCKS_PROCESSED: IntCounter = register_int_counter!(
        "berastribute_blocks_processed_total",
        "Total number of blocks processed"
    ).unwrap();

    pub static ref BLOCKS_SKIPPED: IntCounter = register_int_counter!(
        "berastribute_blocks_skipped_total",
        "Total number of blocks skipped due to fee recipient mismatch"
    ).unwrap();

    // Beacon API metrics
    pub static ref BEACON_API_ERRORS: IntCounterVec = register_int_counter_vec!(
        "berastribute_beacon_api_errors_total",
        "Total number of Beacon API errors",
        &["error_type"]
    ).unwrap();

    // Latest block number processed
    pub static ref LATEST_BLOCK_NUMBER: IntGauge = register_int_gauge!(
        "berastribute_latest_block_number",
        "Latest block number processed"
    ).unwrap();

    // Enhanced distribution metrics
    pub static ref DISTRIBUTION_DURATION: HistogramVec = register_histogram_vec!(
        "berastribute_distribution_duration_seconds",
        "Time taken to complete distribution",
        &["status"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).unwrap();

    pub static ref BEACON_API_DURATION: Histogram = register_histogram!(
        "berastribute_beacon_api_duration_seconds",
        "Time taken for beacon API calls",
        vec![0.1, 0.5, 1.0, 2.0, 5.0]
    ).unwrap();

    pub static ref GAS_USED: Histogram = register_histogram!(
        "berastribute_gas_used",
        "Gas used for distribution transactions",
        vec![500_000.0, 750_000.0, 1_000_000.0, 1_250_000.0, 1_500_000.0]
    ).unwrap();

    // Backfill metrics
    pub static ref BACKFILL_BLOCKS_REMAINING: IntGauge = register_int_gauge!(
        "berastribute_backfill_blocks_remaining",
        "Number of blocks remaining to backfill"
    ).unwrap();

    pub static ref BACKFILL_PROGRESS_PERCENT: IntGauge = register_int_gauge!(
        "berastribute_backfill_progress_percent",
        "Backfill progress as a percentage"
    ).unwrap();

    // Contract interaction metrics
    pub static ref CONTRACT_CALLS: IntCounterVec = register_int_counter_vec!(
        "berastribute_contract_calls_total",
        "Total number of contract calls",
        &["method", "status"]
    ).unwrap();

    // System metrics
    pub static ref MEMORY_USAGE_BYTES: IntGauge = register_int_gauge!(
        "berastribute_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap();

    // Chain sync metrics
    pub static ref CHAIN_HEAD_BLOCK: IntGauge = register_int_gauge!(
        "berastribute_chain_head_block",
        "Latest chain head block number"
    ).unwrap();

    pub static ref BLOCK_LAG: IntGauge = register_int_gauge!(
        "berastribute_block_lag",
        "Number of blocks behind chain head"
    ).unwrap();

    // Retry metrics
    pub static ref RETRY_ATTEMPTS: IntCounterVec = register_int_counter_vec!(
        "berastribute_retry_attempts_total",
        "Total number of retry attempts",
        &["operation"]
    ).unwrap();
}

pub async fn init_metrics() {
    let metrics_route = warp::path!("metrics").map(|| {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    });

    tokio::spawn(warp::serve(metrics_route).run(([0, 0, 0, 0], 11111)));
    tracing::info!("Metrics server started on :11111");
}
