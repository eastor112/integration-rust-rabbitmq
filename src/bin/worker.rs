use integration_rust_rabbitmq::worker_utils;
use tracing::{error, info, warn};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter("worker=info,integration_rust_rabbitmq=info")
        .init();

    info!("🔧 Worker starting...");

    // Initialize RabbitMQ pool once at startup
    integration_rust_rabbitmq::connection::init_rabbitmq_pool().await?;

    let mut retry_count = 0;
    loop {
        match worker_utils::run_worker().await {
            Ok(_) => {
                info!("Worker completed successfully");
                break;
            }
            Err(e) => {
                error!("Worker failed: {}", e);
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    error!("Max retries ({}) exceeded. Exiting.", MAX_RETRIES);
                    return Err(e);
                }
                warn!(
                    "Retrying in {:?} (attempt {}/{})",
                    RETRY_DELAY, retry_count, MAX_RETRIES
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }
    Ok(())
}
