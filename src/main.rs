mod handlers;
mod models;

use actix_web::{App, HttpServer};
use env_logger::init;
use handlers::{send_notification, schedule_notification, notification_scheduler_task, send_notification_delayed};
use tokio::task;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init(); // for logs
    println!("🚀 Server running at http://localhost:8081");

    // Launch the background task for the scheduler
    task::spawn(notification_scheduler_task());

    HttpServer::new(|| {
        App::new()
            .service(send_notification_delayed)
            .service(send_notification)
            .service(schedule_notification)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
