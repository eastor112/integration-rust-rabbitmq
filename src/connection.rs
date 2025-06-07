use lapin::{Connection, ConnectionProperties, Channel};
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
    let pool = RabbitMQPool::new("amqp://guest:guest@localhost:5672/%2f").await?;
    RABBITMQ_POOL.set(pool).map_err(|_| "Failed to set RabbitMQ pool")?;
    Ok(())
}

pub fn get_rabbitmq_pool() -> Result<&'static RabbitMQPool, &'static str> {
    RABBITMQ_POOL.get().ok_or("RabbitMQ pool not initialized")
}
