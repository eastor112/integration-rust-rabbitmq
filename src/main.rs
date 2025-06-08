mod connection;
mod handlers;
mod models;

use actix_web::{App, HttpServer, middleware::Logger};
use connection::init_rabbitmq_pool;
use handlers::{
    notification_scheduler_task, schedule_notification, send_notification, send_notification_at,
    send_notification_delayed,
};
use tokio::task;
use tracing::{error, info};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,integration_rust_rabbitmq=debug,actix_web=info")
        .init();

    info!("🚀 Starting notification service...");

    // Initialize connection pool
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
            .service(send_notification_at)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
