#!/bin/bash
# Initialization script to create exchanges and queues in RabbitMQ

# Wait until RabbitMQ is ready (port 5672 open)
until nc -z localhost 5672; do
  echo "Waiting for RabbitMQ to be ready..."
  sleep 2
done

# Wait until the management plugin is ready (port 15672 open)
until nc -z localhost 15672; do
  echo "Waiting for RabbitMQ Management to be ready..."
  sleep 2
done

# Enable the delayed_message_exchange plugin (online mode)
rabbitmq-plugins enable rabbitmq_delayed_message_exchange

# Create delayed exchange
rabbitmqadmin declare exchange name=delayed_exchange type=x-delayed-message arguments='{"x-delayed-type":"direct"}'

# Create DLX
rabbitmqadmin declare exchange name=dlx_exchange type=direct

# Create main queue with DLX
rabbitmqadmin declare queue name=main_queue arguments='{"x-dead-letter-exchange":"dlx_exchange"}'
rabbitmqadmin declare binding source=delayed_exchange destination=main_queue routing_key=main

# Create dead letter queue and binding
rabbitmqadmin declare queue name=dead_letter_queue
rabbitmqadmin declare binding source=dlx_exchange destination=dead_letter_queue routing_key=main

echo "RabbitMQ configuration completed."
