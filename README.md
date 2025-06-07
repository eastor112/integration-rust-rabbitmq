# integration-rust-rabbitmq

A production-ready example project for scheduled and delayed notifications using **Rust**, **RabbitMQ**, and **Actix Web**.

## Features

- REST API (Actix Web) to schedule, delay, and send notifications.
- Asynchronous worker for processing and delivering notifications.
- Uses RabbitMQ with the [delayed message exchange plugin](https://github.com/rabbitmq/rabbitmq-delayed-message-exchange) for precise scheduling (including multi-day delays).
- Automatic requeueing for notifications scheduled more than 7 days in advance.
- Idiomatic, modular, and production-ready Rust code.

## How it works

- **/notify**: Immediate notification.
- **/notify-delayed**: Notification with a delay in seconds.
- **/notify-at**: Notification scheduled for a specific date/time (RFC3339). If the delay exceeds 7 days, the system automatically requeues until the target date.

The **worker** consumes messages from RabbitMQ and delivers notifications at the correct time, handling requeueing for long-term scheduling.

## Quick Start

### 1. Start RabbitMQ (with management UI)

```sh
docker compose up -d
```

### 2. Initialize exchanges and queues (only the first time)

```sh
init-rabbitmq.bat
```

### 3. Build and run the API

```sh
cargo run --release
```

### 4. Build and run the worker (in another terminal)

```sh
cargo run --release --bin worker
```

### 5. Test the API

- Scheduled notification:
    ```sh
    curl -X POST http://localhost:8081/notify-at \
      -H "Content-Type: application/json" \
      -d '{"user_id":"user123","message":"Hello future!","scheduled_at":"2025-06-08T18:00:00Z"}'
    ```
- Delayed notification:
    ```sh
    curl -X POST http://localhost:8081/notify-delayed \
      -H "Content-Type: application/json" \
      -d '{"user_id":"user123","message":"Wait...","delay_secs":60}'
    ```

## Architecture

- **src/main.rs**: Actix Web API server.
- **src/bin/worker.rs**: Worker for consuming and delivering notifications.
- **src/models.rs**: Shared data models.
- **src/connection.rs**: RabbitMQ connection pool.
- **src/worker_utils.rs**: Worker logic and helpers.
- **docker-compose.yml**: Example RabbitMQ service.

## Notes

- The project is production-ready: you can scale the API and worker independently.
- All scheduling logic is robust against RabbitMQ delay limitations.
- For real persistence, replace the in-memory "DB" with a real database.

---

**Made with Rust, Actix, and RabbitMQ 🦀**
