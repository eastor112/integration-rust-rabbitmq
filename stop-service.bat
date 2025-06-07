@echo off
echo ========================================
echo   🛑 DETENIENDO SERVICIO RABBITMQ + RUST
echo ========================================
echo.

echo 🔄 Deteniendo procesos Rust...
taskkill /F /IM integration-rust-rabbitmq.exe 2>nul
taskkill /F /IM worker.exe 2>nul

echo 📦 Deteniendo contenedores Docker...
docker-compose down

echo ✅ Servicios detenidos correctamente
pause
