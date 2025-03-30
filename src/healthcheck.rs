use healthchecks::ping::get_client;

/// Sends a ping to healthchecks.io with a success signal
/// Does nothing if healthcheck_id is not set

pub async fn ping_healthcheck(healthcheck_id: Option<&str>) {
    // If no healthcheck_id is provided, do nothing
    let Some(id) = healthcheck_id else {
        return;
    };

    // Create the healthchecks client
    let client = match get_client(id) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!("Failed to create healthcheck client: {}", e);
            return;
        }
    };

    // Send the ping
    let success = client.report_success();
    if success {
        tracing::debug!("Healthcheck ping sent successfully");
    } else {
        tracing::warn!("Healthcheck ping failed to send");
    }
}
