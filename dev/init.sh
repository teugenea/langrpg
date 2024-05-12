#!/bin/bash

set -a
source ./.env
set +a

mkdir -p ${DB_DATA_DIR}

docker-compose up -d postgres
until docker inspect --format "{{json .State.Health.Status }}" garptea-postgres|grep -m 1 "healthy"; do sleep 1 ; done

docker exec garptea-postgres sh -c "psql -U postgres -c \"CREATE USER ${CASDOOR_DB_USER} WITH ENCRYPTED PASSWORD '${CASDOOR_DB_PASSWORD}'\""
docker exec garptea-postgres sh -c "psql -U postgres -c \"CREATE DATABASE ${CASDOOR_DB_NAME} OWNER ${CASDOOR_DB_USER}\""

docker exec garptea-postgres sh -c "psql -U postgres -c \"CREATE USER ${APP_DB_USER} WITH ENCRYPTED PASSWORD '${APP_DB_PASSWORD}'\""
docker exec garptea-postgres sh -c "psql -U postgres -c \"CREATE DATABASE ${APP_DB} OWNER ${APP_DB_USER}\""

docker-compose stop