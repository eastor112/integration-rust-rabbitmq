@echo off
echo Running RabbitMQ initialization script...
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare exchange name=delayed_exchange type=x-delayed-message arguments='{\"x-delayed-type\":\"direct\"}'"
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare exchange name=dlx_exchange type=direct"
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare queue name=main_queue arguments='{\"x-dead-letter-exchange\":\"dlx_exchange\"}'"
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare binding source=delayed_exchange destination=main_queue routing_key=main"
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare queue name=dead_letter_queue"
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest declare binding source=dlx_exchange destination=dead_letter_queue routing_key=main"
echo RabbitMQ configuration completed successfully.
echo.
echo Created resources:
echo Exchanges:
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest list exchanges name"
echo.
echo Queues:
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest list queues name"
echo.
echo Bindings:
docker exec rabbitmq /bin/bash -c "rabbitmqadmin -u guest -p guest list bindings"
