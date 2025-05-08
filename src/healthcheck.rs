use healthchecks::ping::get_client;

/// Sends a ping to healthchecks.io with a success signal
/// Does nothing if healthcheck_id is not set

pub async fn ping_healthcheck(healthcheck_id: Option<&str>) {
    // If no healthcheck_id is provided, do nothing
    let Some(id) = healthcheck_id else {
        tracing::debug!("No healthcheck_id provided, skipping ping");
        return;
    };

    tracing::debug!("Attempting to ping healthcheck with ID: {}", id);

    // Create the healthchecks client
    let client = match get_client(id) {
        Ok(client) => {
            tracing::debug!("Successfully created healthcheck client");
            client
        }
        Err(e) => {
            tracing::warn!("Failed to create healthcheck client: {}", e);
            return;
        }
    };

    // Send the ping
    tracing::debug!("Sending ping to healthcheck...");
    let success = client.report_success();
    tracing::debug!("report_success() returned: {}", success);

    if success {
        tracing::debug!("Healthcheck ping sent successfully");
    } else {
        tracing::warn!(
            "Healthcheck ping failed to send. ID: {}, URL: https://hc-ping.com/{}",
            id,
            id
        );
    }
}
