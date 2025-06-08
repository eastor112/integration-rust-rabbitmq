@echo off
echo 🚀 Setting up RabbitMQ Integration Project
echo.

:: Check if .env exists
if not exist .env (
    echo 📝 Creating .env file from .env.example...
    copy .env.example .env >nul
    echo ✅ .env file created! Please edit it with your configuration.
    echo.
    pause
    notepad .env
) else (
    echo ✅ .env file already exists
)

echo.
echo 🐰 Starting RabbitMQ with Docker Compose...
docker compose up -d

echo.
echo ⏳ Waiting for RabbitMQ to be ready...
timeout /t 10 /nobreak >nul

echo.
echo 🎯 RabbitMQ Management UI available at: http://localhost:%RABBITMQ_MANAGEMENT_PORT%
echo    Default credentials: %DOCKER_RABBITMQ_USER%/%DOCKER_RABBITMQ_PASSWORD%
echo.
echo 📋 To test the setup:
echo    1. Run: cargo run --release
echo    2. In another terminal: cargo run --release --bin worker
echo    3. Test API: curl -X POST http://localhost:%SERVER_PORT%/notify
echo.
echo ✅ Setup completed!
pause
