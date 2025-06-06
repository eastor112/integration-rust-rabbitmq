mod handlers;
mod models;

use actix_web::{App, HttpServer};
use env_logger::init;
use handlers::send_notification;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init(); // para logs
    println!("🚀 Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(send_notification)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
