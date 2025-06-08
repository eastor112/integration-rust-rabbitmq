use lapin::{Channel, Connection, ConnectionProperties};
use std::env;
use std::sync::Arc;

pub struct RabbitMQPool {
    connection: Arc<Connection>,
}

impl RabbitMQPool {
    pub async fn new(amqp_url: &str) -> Result<Self, lapin::Error> {
        let connection = Connection::connect(amqp_url, ConnectionProperties::default()).await?;
        Ok(Self {
            connection: Arc::new(connection),
        })
    }

    pub async fn get_channel(&self) -> Result<Channel, lapin::Error> {
        self.connection.create_channel().await
    }

    // pub fn is_connected(&self) -> bool {
    //     self.connection.status().connected()
    // }
}

// Singleton global
lazy_static::lazy_static! {
    pub static ref RABBITMQ_POOL: tokio::sync::OnceCell<RabbitMQPool> = tokio::sync::OnceCell::new();
}

pub async fn init_rabbitmq_pool() -> Result<(), Box<dyn std::error::Error>> {
    let user = env::var("RABBITMQ_DEFAULT_USER").unwrap_or_else(|_| "guest".to_string());
    let pass = env::var("RABBITMQ_DEFAULT_PASS").unwrap_or_else(|_| "guest".to_string());
    let host = env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());

    let amqp_url = format!("amqp://{}:{}@{}:{}/%2f", user, pass, host, port);
    println!("Connecting to RabbitMQ at: {}", amqp_url);
    let pool = RabbitMQPool::new(&amqp_url).await?;
    RABBITMQ_POOL
        .set(pool)
        .map_err(|_| "Failed to set RabbitMQ pool")?;
    Ok(())
}

pub fn get_rabbitmq_pool() -> Result<&'static RabbitMQPool, &'static str> {
    RABBITMQ_POOL.get().ok_or("RabbitMQ pool not initialized")
}
