# 🚀 RabbitMQ + Rust - Setup Simplificado

## 📁 Archivos FINALES (Limpieza Completada)

### ✅ **Archivos NECESARIOS:**
```
├── docker-compose.yml          # RabbitMQ simplificado
├── init-rabbitmq.bat          # Inicialización manual
├── start-service.bat          # 🚀 INICIAR TODO
├── stop-service.bat           # 🛑 DETENER TODO
├── Cargo.toml                 # Dependencias Rust
├── src/
│   ├── main.rs               # Servidor API
│   ├── handlers.rs           # Handlers HTTP
│   ├── models.rs             # Modelos de datos
│   └── bin/worker.rs         # Worker consumidor
└── plugins/                  # Plugin delayed exchange
    └── rabbitmq_delayed_message_exchange-v4.0.7.ez
```

### ❌ **Archivos ELIMINADOS (ya no existen):**
- ~~rabbitmq-init.sh~~ (obsoleto)
- ~~enable-plugins.sh~~ (obsoleto)
- ~~rabbitmq-init container~~ (del docker-compose)

## 🎯 **Uso SÚPER SIMPLE:**

### Para INICIAR todo:
```cmd
start-service.bat
```

### Para DETENER todo:
```cmd
stop-service.bat
```

## 📊 **Lo que hace cada script:**

### `start-service.bat`:
1. 📦 Levanta RabbitMQ con Docker
2. ⏳ Espera que esté listo
3. ⚙️ Crea exchanges y queues
4. 🔨 Compila la app Rust
5. 👷 Inicia Worker (consumidor)
6. 🌐 Inicia API Server
7. 🧪 Prueba que funcione

### `stop-service.bat`:
1. 🔄 Mata procesos Rust
2. 📦 Para contenedores Docker

## 🔗 **Endpoints API:**
- `POST /notify` - Notificación inmediata
- `POST /notify-delayed` - Notificación con delay
- `POST /schedule-notification` - Notificación programada

## 📊 **Monitoreo:**
- RabbitMQ UI: http://localhost:15672 (guest/guest)
- API Server: http://localhost:8081

¡Ahora es mucho más simple y limpio! 🎉
