<br/>
<p align="center">
  <img src="https://assets.tago.io/tagoio/tagoio.png" width="250px" alt="TagoIO"></img>
</p>

# TagoIO | MQTT Relay

This software bridges your MQTT Broker and the TagoIO platform, allowing seamless integration and data flow. It's a fast, open-source, and scalable solution written in Rust.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
  - [Running the Relay](#running-the-relay)
- [Docker Setup](#docker-setup)
  - [Image Variants](#image-variants)
- [CLI Commands](#cli-commands)
  - [`init`](#init)
  - [`start`](#start)
- [Configuration File and Environment Variables](#configuration-file-and-environment-variables)
- [License](#license)

## Introduction

The TagoIO MQTT Relay connects to your MQTT Broker on predefined topics and redirects the information to TagoIO Devices. It uses TagoIO Integration Network and Connector, alongside an [Authorization Key](https://admin.tago.io/devices/authorization) from your TagoIO Profile.

## Features

- **Written in Rust**: Fast and reliable performance.
- **Open Source**: Available on GitHub for community contributions.
- **Scalable**: Easily handles increasing data loads.
- **Docker Support**: Simplifies deployment and scaling.

## Getting Started

### Prerequisites

Before you begin, ensure you have:

- A TagoIO account.
- Access to an MQTT Broker (e.g., HiveMQ, EMQX).

### Installation

1. **Download the Binary**: Get the `tagoio-relay` binary from the [GitHub releases page](https://github.com/tago-io/mqtt-relay/releases/latest).
2. **Set Permission Rights**: Give the binary executable permissions:
   ```sh
   chmod +x tagoio-relay
   ```
3. **Resolve Malicious Software Alert (macOS)**: Follow [Apple Support](https://support.apple.com) instructions if you encounter a malicious software alert.
4. **Verify Installation**: Check if the `tagoio-relay` is working properly:
   ```sh
   ./tagoio-relay --help
   ```

### Configuration

1. **Create a Network in TagoIO**:
   - Navigate to [Integrations](https://admin.tago.io/integrations/network) in your TagoIO [Profile](https://help.tago.io/portal/en/kb/articles/198-profiles) and create a new Network.
   - Enter the [Middleware Endpoint](#middleware-endpoint-optional) for your Relay (Optional, allows publishing from TagoIO to the Relay).
   - Enable Serial and write a Payload Parser:
     ```js
     // The Payload Parser runs every time a message is received from the Broker
     if (Array.isArray(payload)) {
        // Relay sends the "payload" variable by default
       const payload_received = payload.find(x => x.variable === "payload");
       // Extract the serial from the last element of the topic (e.g., /device/SERIAL)
       serial = payload_received?.metadata.topic.split("/").pop();
     }
     ```

2. **Generate Network Token**: Generate and save the Network Token.
3. **Create a Connector**: Create a Connector for your Network.
4. **Generate Authorization**: Navigate to [Devices > Authorizations](https://admin.tago.io/devices/authorization) in TagoIO and generate an authorization token.
5. **Create a Device**: Create a Device with a Serial to use later on the Broker.
6. **Set Up an MQTT Broker**: Create or use a public MQTT Broker.
7. **Gather Broker Details**: Obtain the Address, Port, and Credentials of your MQTT Broker.

### Running the Relay

1. **Initialize Config File**:
   ```sh
   ./tagoio-relay init
   ```
2. **Edit Config File**: Modify the generated `.tagoio-mqtt-relay.toml` as described in the [Configuration File and Environment Variables](#configuration-file-and-environment-variables).

3. **Start the Relay**:
   ```sh
   ./tagoio-relay start
   ```
4. **Publish Messages to Broker**: Publish messages to the Broker on your chosen topics and see them forwarded to your TagoIO device.

## Docker Setup

To run the TagoIO MQTT Relay using Docker, use the following command:

```sh
docker run -p 3001:3001 -it --rm --name my-test tagoio/relay start --no-daemon
```

Specific docker documentation can be found on the [Docker Hub page](https://hub.docker.com/r/tagoio/relay).

### Image Variants

- **tagoio/relay:latest**: Main image for general use.
- **tagoio/relay:<version>**: Specific version of the image.
- **tagoio/relay:bookworm**: Based on Debian Bookworm.
- **tagoio/relay:alpine**: Based on Alpine Linux.
- **tagoio/relay:<version>-<distribution>**: Specific version of the image for a specific distribution.

## CLI Commands

The CLI has two main commands: `init` and `start`.

### `init`

Generates the `.tagoio-mqtt-relay.toml` file required for setting up the Relay.

```sh
tagoio-relay init [--config-path /path/to/config]
```

### `start`

Starts the MQTT Relay service.

```sh
tagoio-relay start [--verbose info,error,mqtt,network] [--config-path /path/to/.tagoio-mqtt-relay.toml]
```

## Configuration File and Environment Variables

To configure the TagoIO MQTT Relay, you can either use environment variables or edit the `.tagoio-mqtt-relay.toml` file directly. Below are the available configuration parameters:

### `.tagoio-mqtt-relay.toml`

The `.tagoio-mqtt-relay.toml` file generated by the `init` command contains the Relay parameters. By default the file is located at `~/.config/.tagoio-mqtt-relay.toml` (Mac/Linux) or `/root/.config/.tagoio-mqtt-relay.toml` (Linux/Docker).

Here is a reference:
```toml
[relay]
network_token="Your-Network-Token"
authorization_token="Your-Authorization-Token"
tagoio_url="https://api.tago.io"

# The Relay will listen on this port for incoming messages from TagoIO 
downlink_port="3001"

[relay.mqtt]
client_id="tagoio-relay"
tls_enabled=false
address="localhost"
port=1883
subscribe=["/tago/#", "/topic/+"]
username="my-username"
password="my-password"

# TLS Certificates for the MQTT Broker (optional)
# broker_tls_ca=""
# broker_tls_cert=""
# broker_tls_key=""
```
### Environment Variables
The environment variables can be set directly in the shell, and they will override the values provided in the `.tagoio-mqtt-relay.toml` file. Use it as alternative in case you don't want to use or edit the configuration file.

```sh
export TAGOIO__RELAY__NETWORK_TOKEN="Your-Network-Token"
export TAGOIO__RELAY__AUTHORIZATION_TOKEN="Your-Authorization-Token"
export TAGOIO__RELAY__TAGOIO_URL="https://api.tago.io"
export TAGOIO__RELAY__DOWNLINK_PORT="3001"

# MQTT Client Settings
export TAGOIO__RELAY__MQTT__CLIENT_ID="tagoio-relay"
export TAGOIO__RELAY__MQTT__TLS_ENABLED="false"
export TAGOIO__RELAY__MQTT__ADDRESS="localhost"
export TAGOIO__RELAY__MQTT__PORT="1883"
export TAGOIO__RELAY__MQTT__USERNAME="my-username"
export TAGOIO__RELAY__MQTT__PASSWORD="my-password"
export TAGOIO__RELAY__MQTT__BROKER_TLS_CA=""
export TAGOIO__RELAY__MQTT__BROKER_TLS_CERT=""
export TAGOIO__RELAY__MQTT__BROKER_TLS_KEY=""

# Subscribe to multiple topics
export TAGOIO__RELAY__MQTT__SUBSCRIBE=["/device/#"] 

# Change the path to the configuration file
export TAGOIO__RELAY__CONFIG_PATH="/root/.config/.tagoio-mqtt-relay.toml"
```

### Middleware Endpoint (Optional)
The Middleware Endpoint allows the TagoIO MQTT Relay to receive messages from TagoIO through a secure TLS connection. This feature is optional but can be very useful for advanced integrations.

The Relay comes with pre-set TLS certificates configured during build time, so you don't need to set them up manually.

#### Setting Up the Middleware Endpoint

1. **Public HTTPs Endpoint:**:
   Ensure that you have a public HTTPs endpoint that TagoiO can reach.
2. **Configure the Downlink Port:**
  The default port for the Middleware Endpoint is 3001, but you can change this by setting the downlink_port in your (configuration file)[#configuration-file-and-environment-variables].
Repl
3. **Local Testing**:
  For local testing, you can use tools like ngrok or tailscale to expose your local server to the internet securely. Ensure to run the relay with the `--unsafe-mode` flag.

  ```sh
  tagoio-relay start --unsafe-mode
  ```

  **Using Ngrok:**
  ```bash
  ngrok http https://localhost:3000
  ```

  **Using Tailscale:**
  ```bash
  tailscale funnel 3000
  ```
4. **Network Middleware Endpoint:**
   To enable the Middleware Endpoint, you need to set the field `Middleware Endpoint` in your Network at TagoIO to the generated URL (e.g., https://abcd1234.ngrok.io) as your Middleware Endpoint in TagoIO.

## License

The TagoIO MQTT Relay is licensed under the Apache License. See the [LICENSE](./LICENSE) file for more details.

---

Thank you for using TagoIO MQTT Relay! If you have any questions or need further assistance, feel free to reach out via [GitHub Issues](https://github.com/tago-io/mqtt-relay/issues) or our [community forum](https://community.tago.io). 🚀
