use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};
use env_logger;
use lapin::{BasicProperties, Connection, ConnectionProperties, options::*, types::FieldTable};
use push_rabbit_demo::Notification;
use serde_json;
use std::sync::Arc;

#[post("/notify")]
async fn notify(
    payload: web::Json<Notification>,
    amqp: web::Data<Arc<lapin::Channel>>,
) -> impl Responder {
    let body = serde_json::to_vec(&*payload).unwrap();

    amqp.basic_publish(
        "notifications",
        "",
        BasicPublishOptions::default(),
        body,
        BasicProperties::default(),
    )
    .await
    .unwrap()
    .await
    .unwrap();

    HttpResponse::Ok().body("Notification scheduled")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let conn = Connection::connect(
        "amqp://guest:guest@localhost:5672/%2f",
        ConnectionProperties::default(),
    )
    .await
    .unwrap();

    let channel = conn.create_channel().await.unwrap();

    channel
        .exchange_declare(
            "notifications",
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let shared_channel = web::Data::new(Arc::new(channel));

    println!("🚀 Actix server running on http://localhost:8080");

    HttpServer::new(move || App::new().app_data(shared_channel.clone()).service(notify))
        .bind(("0.0.0.0", 8081))?
        .run()
        .await
}
