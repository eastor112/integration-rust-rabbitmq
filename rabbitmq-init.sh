#!/bin/bash
# Script de inicialización para crear exchanges y colas en RabbitMQ

# Esperar a que RabbitMQ esté listo
until rabbitmqctl status; do
  echo "Esperando a que RabbitMQ esté listo..."
  sleep 2
done

# Crear exchange delayed
rabbitmqadmin declare exchange name=delayed_exchange type=x-delayed-message arguments='{"x-delayed-type":"direct"}'

# Crear DLX
rabbitmqadmin declare exchange name=dlx_exchange type=direct

# Crear cola principal con DLX
rabbitmqadmin declare queue name=main_queue arguments='{"x-dead-letter-exchange":"dlx_exchange"}'
rabbitmqadmin declare binding source=delayed_exchange destination=main_queue routing_key=main

# Crear cola de errores y binding
rabbitmqadmin declare queue name=dead_letter_queue
rabbitmqadmin declare binding source=dlx_exchange destination=dead_letter_queue routing_key=main

echo "Configuración de RabbitMQ completada."
