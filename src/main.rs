mod connection;
mod handlers;
mod models;
mod config;

use actix_web::{App, HttpServer, middleware::Logger};
use connection::init_rabbitmq_pool;
use config::Config;
use handlers::{
    notification_scheduler_task, schedule_notification, send_notification, send_notification_at,
    send_notification_delayed,
};
use tokio::task;
use tracing::{error, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,integration_rust_rabbitmq=debug,actix_web=info")
        .init();

    info!("🚀 Starting notification service...");

    // Load configuration
    let config = match Config::from_env() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            info!("📊 RabbitMQ URL: {}", config.rabbitmq_url);
            info!("🌐 Server will bind to {}:{}", config.server_host, config.server_port);
            config
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize connection pool
    if let Err(e) = init_rabbitmq_pool().await {
        error!("❌ Failed to initialize RabbitMQ pool: {}", e);
        std::process::exit(1);
    }

    info!("✅ RabbitMQ connection pool initialized");

    // Launch background scheduler
    info!("📅 Starting notification scheduler task");
    task::spawn(notification_scheduler_task());

    info!("🌐 Starting HTTP server at http://{}:{}", config.server_host, config.server_port);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(send_notification_delayed)
            .service(send_notification)
            .service(schedule_notification)
            .service(send_notification_at)
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .run()
    .await
}
