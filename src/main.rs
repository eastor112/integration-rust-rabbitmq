mod handlers;
mod models;
mod connection;

use actix_web::{App, HttpServer, middleware::Logger};
use handlers::{send_notification, schedule_notification, notification_scheduler_task, send_notification_delayed};
use connection::init_rabbitmq_pool;
use tokio::task;
use tracing::{info, error};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configurar tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,integration_rust_rabbitmq=debug,actix_web=info")
        .init();

    info!("🚀 Starting notification service...");

    // Inicializar connection pool
    if let Err(e) = init_rabbitmq_pool().await {
        error!("❌ Failed to initialize RabbitMQ pool: {}", e);
        std::process::exit(1);
    }

    info!("✅ RabbitMQ connection pool initialized");

    // Launch background scheduler
    info!("📅 Starting notification scheduler task");
    task::spawn(notification_scheduler_task());

    info!("🌐 Starting HTTP server at http://localhost:8081");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(send_notification_delayed)
            .service(send_notification)
            .service(schedule_notification)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
