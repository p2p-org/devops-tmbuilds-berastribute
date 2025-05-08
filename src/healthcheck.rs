use reqwest::Client;
use std::time::Duration;

/// Sends a ping to healthchecks.io with a success signal
/// Does nothing if healthcheck_id is not set
pub async fn ping_healthcheck(healthcheck_id: Option<&str>) {
    // If no healthcheck_id is provided, do nothing
    let Some(id) = healthcheck_id else {
        tracing::debug!("No healthcheck_id provided, skipping ping");
        return;
    };

    tracing::debug!("Attempting to ping healthcheck with ID: {}", id);

    // Create a client with timeout and retry settings
    let client = Client::builder().timeout(Duration::from_secs(10)).build().unwrap_or_else(|e| {
        tracing::warn!("Failed to create HTTP client: {}", e);
        Client::new()
    });

    let url = format!("https://hc-ping.com/{}", id);
    tracing::debug!("Sending ping to: {}", url);

    // Try up to 5 times
    for attempt in 1..=5 {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::debug!("Healthcheck ping sent successfully");
                    return;
                } else {
                    tracing::warn!(
                        "Healthcheck ping failed with status {} (attempt {}/5)",
                        response.status(),
                        attempt
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Healthcheck ping failed with error: {} (attempt {}/5)", e, attempt);
            }
        }
        if attempt < 5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    tracing::warn!("Healthcheck ping failed after 5 attempts. ID: {}, URL: {}", id, url);
}
