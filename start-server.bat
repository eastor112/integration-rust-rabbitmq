@echo off
title RabbitMQ Integration - API Server

echo 🚀 Starting RabbitMQ Integration API Server...
echo.
echo 📊 Configuration:
echo    - Loading from .env file if available
echo    - RabbitMQ: %RABBITMQ_HOST%:%RABBITMQ_PORT%
echo    - Server: %SERVER_HOST%:%SERVER_PORT%
echo.

cargo run --release

pause
