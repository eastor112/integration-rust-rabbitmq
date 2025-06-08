use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub rabbitmq_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let rabbitmq_host = env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
        let rabbitmq_port: u16 = env::var("RABBITMQ_PORT")
            .unwrap_or_else(|_| "5672".to_string())
            .parse()
            .unwrap_or(5672);
        let rabbitmq_user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string());
        let rabbitmq_password = env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string());
        let rabbitmq_vhost = env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "%2f".to_string());

        // Build the full AMQP URL
        let rabbitmq_url = env::var("RABBITMQ_URL")
            .unwrap_or_else(|_| {
                format!(
                    "amqp://{}:{}@{}:{}/{}",
                    rabbitmq_user, rabbitmq_password, rabbitmq_host, rabbitmq_port, rabbitmq_vhost
                )
            });

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port: u16 = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8081".to_string())
            .parse()
            .unwrap_or(8081);

        Ok(Config {
            rabbitmq_url,
            server_host,
            server_port,
        })
    }

}

impl Default for Config {
    fn default() -> Self {
        Config {
            rabbitmq_url: "amqp://guest:guest@localhost:5672/%2f".to_string(),
            server_host: "127.0.0.1".to_string(),
            server_port: 8081,
        }
    }
}
