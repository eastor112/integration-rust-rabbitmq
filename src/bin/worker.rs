use integration_rust_rabbitmq::models::Notification;
use lapin::{options::*, Connection, ConnectionProperties, types::FieldTable};
use futures_util::stream::StreamExt;
use std::time::Duration;
use tracing::{info, error, warn, debug};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(5);
const PROCESSING_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar tracing
    tracing_subscriber::fmt()
        .with_env_filter("worker=info,integration_rust_rabbitmq=info")
        .init();

    info!("🔧 Worker starting...");

    let mut retry_count = 0;

    loop {
        match run_worker().await {
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

                warn!("Retrying in {:?} (attempt {}/{})", RETRY_DELAY, retry_count, MAX_RETRIES);
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }

    Ok(())
}

async fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔌 Connecting to RabbitMQ...");

    let conn = Connection::connect(
        "amqp://guest:guest@localhost:5672/%2f",
        ConnectionProperties::default()
    ).await?;

    info!("✅ Connected to RabbitMQ successfully");

    let channel = conn.create_channel().await?;

    // ✨ Configurar QoS para procesar mensajes de uno en uno
    info!("⚙️  Setting QoS to 1 message at a time...");
    channel.basic_qos(1, BasicQosOptions::default()).await?;

    // Verificar que la queue existe
    debug!("🔍 Verifying main_queue exists...");
    channel.queue_declare(
        "main_queue",
        QueueDeclareOptions {
            passive: true,
            ..Default::default()
        },
        FieldTable::default(),
    ).await?;

    info!("📡 Creating consumer...");
    let mut consumer = channel
        .basic_consume(
            "main_queue",
            "push_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("✅ Worker ready to receive messages from main_queue");
    info!("🔍 Waiting for messages. To exit press CTRL+C");

    // Configurar graceful shutdown
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            // Procesar mensajes
            delivery_result = consumer.next() => {
                match delivery_result {
                    Some(Ok(delivery)) => {
                        if let Err(e) = process_message_with_timeout(delivery).await {
                            error!("Failed to process message: {}", e);
                        }
                    }
                    Some(Err(e)) => {
                        error!("Error receiving message: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    None => {
                        warn!("Consumer stream ended");
                        break;
                    }
                }
            }
            // Graceful shutdown
            _ = &mut shutdown => {
                info!("🛑 Shutdown signal received. Closing gracefully...");
                break;
            }
        }
    }

    info!("👋 Worker shutdown completed");
    Ok(())
}

async fn process_message_with_timeout(
    delivery: lapin::message::Delivery
) -> Result<(), Box<dyn std::error::Error>> {
    // Procesar con timeout
    match tokio::time::timeout(PROCESSING_TIMEOUT, process_message(delivery)).await {
        Ok(result) => result,
        Err(_) => {
            error!("⏰ Message processing timed out after {:?}", PROCESSING_TIMEOUT);
            Err("Processing timeout".into())
        }
    }
}

async fn process_message(delivery: lapin::message::Delivery) -> Result<(), Box<dyn std::error::Error>> {
    let payload = delivery.data.clone();
    info!("📦 Received message of {} bytes", payload.len());
    debug!("📄 Raw payload: {}", String::from_utf8_lossy(&payload));

    match serde_json::from_slice::<Notification>(&payload) {
        Ok(notification) => {
            info!("📩 Processing notification: user_id={}, type={}, delay={}s",
                  notification.user_id, notification.notification_type, notification.delay_secs);

            // ✨ Procesar la notificación
            match process_notification(&notification).await {
                Ok(_) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                    info!("✅ Message acknowledged successfully");
                }
                Err(e) => {
                    error!("❌ Failed to process notification: {}", e);
                    // Requeue en caso de error temporal
                    delivery.nack(BasicNackOptions { requeue: true, ..Default::default() }).await?;
                    warn!("🔄 Message requeued for retry");
                }
            }
        }
        Err(e) => {
            error!("❌ Error deserializing message: {}", e);
            error!("📄 Invalid payload: {}", String::from_utf8_lossy(&payload));

            // Mensaje malformado - enviar a DLQ (no requeue)
            delivery.nack(BasicNackOptions { requeue: false, ..Default::default() }).await?;
            warn!("🗑️ Malformed message sent to dead letter queue");
        }
    }

    Ok(())
}

async fn process_notification(notification: &Notification) -> Result<(), Box<dyn std::error::Error>> {
    info!("📲 Sending push notification to user: {}", notification.user_id);
    info!("💬 Message: {}", notification.message);
    info!("🏷️  Type: {}, Original delay: {}s", notification.notification_type, notification.delay_secs);

    // Simular tiempo de procesamiento realista
    let processing_time = match notification.notification_type.as_str() {
        "immediate" => Duration::from_millis(100),
        "delayed" => Duration::from_millis(200),
        "scheduled" => Duration::from_millis(150),
        _ => Duration::from_millis(100),
    };

    debug!("⏱️  Simulating {}ms processing time", processing_time.as_millis());
    tokio::time::sleep(processing_time).await;

    // Aquí iría la lógica real de envío (FCM, APNS, etc.)
    // Por ahora solo simulamos

    info!("✨ Push notification sent successfully to {}", notification.user_id);

    Ok(())
}
