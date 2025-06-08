# integration-rust-rabbitmq

A production-ready example project for scheduled and delayed notifications using **Rust**, **RabbitMQ**, and **Actix Web**.

## 🚀 Features

- **REST API** (Actix Web) to schedule, delay, and send notifications
- **Asynchronous worker** for processing and delivering notifications
- **RabbitMQ integration** with the [delayed message exchange plugin](https://github.com/rabbitmq/rabbitmq-delayed-message-exchange) for precise scheduling
- **Multi-day scheduling** with automatic requeueing for notifications scheduled more than 7 days in advance
- **🎛️ Fully configurable** through environment variables - no hardcoded values!
- **🛡️ Production-ready** with proper error handling and connection pooling
- **📦 Easy setup** with automated scripts for Windows
- **🐳 Docker support** with configurable compose file

## ⚙️ Configuration

The application is **100% configurable** through environment variables. No more hardcoded connections!

### 🎯 Quick Setup (Recommended)

Run the automated setup script:
```cmd
setup.bat
```
This will:
- Create your `.env` file from the template
- Start RabbitMQ with Docker
- Show you the management UI URL

### 📝 Manual Configuration

#### Option 1: Using .env file (Development)

1. Copy the example configuration:
```cmd
copy .env.example .env
```

2. Edit `.env` with your values:
```env
# RabbitMQ Configuration
RABBITMQ_HOST=localhost
RABBITMQ_PORT=5672
RABBITMQ_USER=guest
RABBITMQ_PASSWORD=guest
RABBITMQ_VHOST=%2f

# HTTP Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8081
```

#### Option 2: Environment Variables (Production)

```cmd
set RABBITMQ_USER=myuser
set RABBITMQ_PASSWORD=mypassword
set SERVER_PORT=3000
cargo run --release
```

#### Option 3: Complete Connection URL

For cloud services or complex setups:
```cmd
set RABBITMQ_URL=amqps://user:pass@cloud-rabbitmq.com:5671/vhost
cargo run --release
```

### 📋 Configuration Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `RABBITMQ_HOST` | `localhost` | RabbitMQ server hostname |
| `RABBITMQ_PORT` | `5672` | RabbitMQ AMQP port |
| `RABBITMQ_USER` | `guest` | RabbitMQ username |
| `RABBITMQ_PASSWORD` | `guest` | RabbitMQ password |
| `RABBITMQ_VHOST` | `%2f` | RabbitMQ virtual host (URL encoded `/` = `%2f`) |
| `RABBITMQ_URL` | - | **Complete AMQP URL** (overrides individual settings) |
| `SERVER_HOST` | `127.0.0.1` | HTTP server bind address (`0.0.0.0` for production) |
| `SERVER_PORT` | `8081` | HTTP server port |
| `DOCKER_RABBITMQ_USER` | `guest` | RabbitMQ user for Docker Compose |
| `DOCKER_RABBITMQ_PASSWORD` | `guest` | RabbitMQ password for Docker Compose |
| `RABBITMQ_MANAGEMENT_PORT` | `15672` | RabbitMQ management UI port |
| `RUST_LOG` | `info` | Logging level (`debug`, `info`, `warn`, `error`) |
| `ENVIRONMENT` | `development` | Environment identifier |

## 🎯 How it Works

### API Endpoints

- **`POST /notify`**: Send immediate notification
- **`POST /notify-delayed`**: Send notification after X seconds delay
- **`POST /notify-at`**: Schedule notification for specific date/time (RFC3339)

### Smart Scheduling

- **Short delays** (< 7 days): Direct RabbitMQ delayed exchange
- **Long delays** (> 7 days): Automatic requeueing system
- **Robust handling**: Connection failures, retries, and graceful shutdowns

## 🚀 Quick Start

### Method 1: Automated Setup (Recommended)

```cmd
# 1. Run automated setup
setup.bat

# 2. Start the API server
start-server.bat

# 3. Start the worker (in another terminal)
start-worker.bat
```

### Method 2: Manual Setup

#### 1. Configure Environment
```cmd
copy .env.example .env
# Edit .env with your settings
```

#### 2. Start RabbitMQ
```cmd
docker compose up -d
```

#### 3. Run the Application
```cmd
# Terminal 1: API Server
cargo run --release

# Terminal 2: Worker
cargo run --release --bin worker
```

## 🧪 Testing the API

### Immediate Notification
```cmd
curl -X POST http://localhost:8081/notify ^
  -H "Content-Type: application/json" ^
  -d "{\"user_id\":\"user123\",\"message\":\"Hello immediately!\"}"
```

### Delayed Notification (60 seconds)
```cmd
curl -X POST http://localhost:8081/notify-delayed ^
  -H "Content-Type: application/json" ^
  -d "{\"user_id\":\"user123\",\"message\":\"Hello in 1 minute!\",\"delay_secs\":60}"
```

### Scheduled Notification
```cmd
curl -X POST http://localhost:8081/notify-at ^
  -H "Content-Type: application/json" ^
  -d "{\"user_id\":\"user123\",\"message\":\"Hello future!\",\"scheduled_at\":\"2025-06-08T18:00:00Z\"}"
```

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │────│   RabbitMQ      │────│     Worker      │
│  (Actix Web)    │    │ (Delayed Msgs)  │    │  (Background)   │
│                 │    │                 │    │                 │
│ • /notify       │    │ • Exchanges     │    │ • Consumes      │
│ • /notify-delay │    │ • Queues        │    │ • Processes     │
│ • /notify-at    │    │ • Scheduling    │    │ • Delivers      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### File Structure
- **`src/main.rs`**: Actix Web API server
- **`src/bin/worker.rs`**: Background notification worker
- **`src/config.rs`**: 🆕 Configuration management
- **`src/connection.rs`**: RabbitMQ connection pool
- **`src/handlers.rs`**: API request handlers
- **`src/models.rs`**: Shared data models
- **`src/worker_utils.rs`**: Worker logic and utilities
- **`docker-compose.yml`**: 🆕 Configurable RabbitMQ setup
- **Setup scripts**: 🆕 `setup.bat`, `start-server.bat`, `start-worker.bat`

## 🌍 Environment Examples

### Development
```env
RABBITMQ_HOST=localhost
RABBITMQ_USER=guest
RABBITMQ_PASSWORD=guest
SERVER_HOST=127.0.0.1
SERVER_PORT=8081
RUST_LOG=debug
```

### Production
```env
RABBITMQ_URL=amqps://prod-user:secure-pass@rabbitmq.company.com:5671/prod
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info
ENVIRONMENT=production
```

### Cloud Services (example with CloudAMQP)
```env
RABBITMQ_URL=amqps://username:password@rattlesnake.rmq.cloudamqp.com/username
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

## 🐳 Docker Support

The `docker-compose.yml` is fully configurable through environment variables:

```cmd
# Use custom credentials
set DOCKER_RABBITMQ_USER=myuser
set DOCKER_RABBITMQ_PASSWORD=mypass

# Start with custom settings
docker compose up -d
```

## 🛠️ Development Tools

### Available Scripts
- **`setup.bat`**: Initial project setup and RabbitMQ start
- **`start-server.bat`**: Start the API server with environment info
- **`start-worker.bat`**: Start the background worker

### Management UI
Access RabbitMQ Management at: `http://localhost:15672`
- Default credentials: `guest/guest` (or your configured values)

## 🚀 Production Deployment

1. **Set environment variables**:
```cmd
set RABBITMQ_URL=your-production-url
set SERVER_HOST=0.0.0.0
set SERVER_PORT=8080
set RUST_LOG=info
```

2. **Build optimized release**:
```cmd
cargo build --release
```

3. **Run services**:
```cmd
# API Server
target\release\integration-rust-rabbitmq.exe

# Worker (separate process/container)
target\release\worker.exe
```

## 📚 Notes

- **🏭 Production Ready**: Proper error handling, connection pooling, and graceful shutdowns
- **📈 Scalable**: Run multiple API servers and workers independently
- **🔒 Secure**: No hardcoded credentials, environment-based configuration
- **🧪 Testable**: Easy to configure for different test environments
- **📊 Observable**: Configurable logging levels and structured output

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

---

**Made with ❤️ using Rust, Actix Web, and RabbitMQ 🦀**
