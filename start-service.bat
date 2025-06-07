@echo off
echo ========================================
echo   🚀 LEVANTANDO SERVICIO RABBITMQ + RUST
echo ========================================
echo.

echo 📦 1. Levantando RabbitMQ...
docker-compose up -d rabbitmq

echo ⏳ 2. Esperando que RabbitMQ esté listo...
timeout /t 20 >nul

echo ⚙️  3. Inicializando RabbitMQ (exchanges y queues) automáticamente...
docker-compose up rabbitmq-setup

echo 🔨 4. Compilando aplicación Rust...
cargo build

echo 👷 5. Levantando Worker...
start "RabbitMQ Worker" cmd /k "cargo run --bin worker"

echo 🌐 6. Levantando Servidor API...
start "API Server" cmd /k "cargo run"

echo ⏳ 7. Esperando que los servicios arranquen...
timeout /t 5 >nul

echo 🧪 8. Probando la API...
curl -X POST http://localhost:8081/notify -H "Content-Type: application/json" -d "{\"user_id\":\"test\",\"message\":\"¡Servicio funcionando!\"}"

echo.
echo ========================================
echo ✅ SERVICIO LEVANTADO CORRECTAMENTE
echo ========================================
echo.
echo 📊 RabbitMQ Management: http://localhost:15672 (guest/guest)
echo 🔗 API Server: http://localhost:8081
echo.
echo Endpoints:
echo   POST /notify              - Notificación inmediata
echo   POST /notify-delayed      - Notificación con delay
echo   POST /schedule-notification - Notificar en fecha específica
echo.
echo ⚠️  Para detener: stop-service.bat
pause
