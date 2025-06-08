@echo off
title RabbitMQ Integration - Worker

echo 🔧 Starting RabbitMQ Integration Worker...
echo.
echo 📊 Configuration:
echo    - Loading from .env file if available
echo    - RabbitMQ: %RABBITMQ_HOST%:%RABBITMQ_PORT%
echo.

cargo run --release --bin worker

pause
