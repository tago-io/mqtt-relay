# TagoIO MQTT Relay Examples

This directory contains example configurations for setting up the TagoIO MQTT Relay using Docker Compose. These examples demonstrate how to configure and deploy the relay with different setups.

## Table of Contents

- [Docker Compose Example](#docker-compose-example)
- [Mosquitto Docker Compose Example](#mosquitto-docker-compose-example)
- [Mosquitto Docker Compose Example with Auth Plugin](#mosquitto-docker-compose-example-with-auth-plugin)

## Docker Compose Example

This example demonstrates how to set up the TagoIO MQTT Relay using Docker Compose with a basic configuration.

### Configuration

- **File**: `docker-compose.yml`
- **Description**: This configuration sets up the TagoIO MQTT Relay service with the necessary ports and volumes. You can configure the relay using a `.tagoio-mqtt-relay.toml` file or environment variables.

### Usage

1. Navigate to the `docker-compose` directory.
2. Ensure the `.tagoio-mqtt-relay.toml` file is correctly configured.
3. Run the following command to start the service:

   ```sh
   docker-compose up -d
   ```
## Mosquitto Docker Compose Example

This example demonstrates how to set up the TagoIO MQTT Relay with an Eclipse Mosquitto broker using Docker Compose.

### Configuration

- **File**: `docker-compose.yml`
- **Description**: This configuration sets up the TagoIO MQTT Relay service and a Mosquitto broker. The relay is configured to connect to the Mosquitto broker using environment variables.

### Usage

1. Navigate to the `mosquitto-docker-compose` directory.
2. Ensure the `.tagoio-mqtt-relay.toml` file is correctly configured if you prefer using it over environment variables.
3. Ensure the `mosquitto/config/mosquitto.conf` file is correctly configured for mosquitto.
4. Run the following command to start the services:

   ```sh
   docker-compose up -d
   ```

## Mosquitto Docker Compose Example with Auth Plugin

This example demonstrates how to set up the TagoIO MQTT Relay with an Eclipse Mosquitto broker using Docker Compose and the mosquitto-go-auth plugin. The plugin enables client authentication using TagoIO device tokens, providing secure access control by validating client credentials against TagoIO's authentication service.

### Configuration
- **File**: `docker-compose.yml`

