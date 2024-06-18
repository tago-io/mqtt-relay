# TagoIO MQTT Relay

Welcome to the TagoIO MQTT Relay! This software bridges your MQTT Broker and the TagoIO platform, allowing seamless integration and data flow. It's a fast, open-source, and scalable solution written in Rust.

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

The TagoIO MQTT Relay connects to your MQTT Broker on predefined topics and redirects the information to TagoIO Devices. It uses TagoIO Integration Network and Connector, alongside an Authorization Key from your TagoIO Profile.

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

1. **Download the Binary**: Get the `tagoio-relay` binary from the [GitHub releases page](./releases/latest).
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
   - Navigate to Integrations in your TagoIO Profile and create a new Network.
   - Enable Serial and write a Payload Parser:
     ```js
     if (Array.isArray(payload)) {
       const payload_received = payload.find(x => x.variable === "payload");
       serial = payload_received?.metadata.topic.split("/").pop();
     }
     ```
2. **Generate Network Token**: Generate and save the Network Token.
3. **Create a Connector**: Create a Connector for your Network.
4. **Generate Authorization**: Navigate to Devices > Authorizations in TagoIO and generate an authorization token.
5. **Create a Device**: Create a Device with a Serial to use later on the Broker.
6. **Set Up an MQTT Broker**: Create or use a public MQTT Broker.
7. **Gather Broker Details**: Obtain the Address, Port, and Credentials of your MQTT Broker.

### Running the Relay

1. **Initialize Config File**:
   ```sh
   ./tagoio-relay init
   ```
2. **Edit Config File**: Modify the generated `config.toml` as described in the [Configuration File and Environment Variables](#configuration-file-and-environment-variables).

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

- **tagoio/relay:<version>**: Main image for general use.
- **tagoio/relay:bullseye**: Based on Debian 11.

## CLI Commands

The CLI has two main commands: `init` and `start`.

### `init`

Generates the `config.toml` file required for setting up the Relay.

```sh
tagoio-relay init [--config-path /path/to/config]
```

### `start`

Starts the MQTT Relay service.

```sh
tagoio-relay start [--verbose info,mqtt] [--config-path /path/to/config.toml]
```

## Configuration File and Environment Variables

To configure the TagoIO MQTT Relay, you can either use environment variables or edit the `config.toml` file directly. Below are the available configuration parameters:

### Environment Variables

```sh
export TAGOIO__RELAY__NETWORK_TOKEN="Your-Network-Token"
export TAGOIO__RELAY__AUTHORIZATION_TOKEN="Your-Authorization-Token"
export TAGOIO__RELAY__TAGOIO_URL="https://api.tago.io"
export TAGOIO__RELAY__DOWNLINK_PORT="3001"
export TAGOIO__RELAY__MQTT__CLIENT_ID="tagoio-relay"
export TAGOIO__RELAY__MQTT__TLS_ENABLED="false"
export TAGOIO__RELAY__MQTT__ADDRESS="localhost"
export TAGOIO__RELAY__MQTT__PORT="1883"
export TAGOIO__RELAY__MQTT__SUBSCRIBE="/tago/# /topic/+"
export TAGOIO__RELAY__MQTT__USERNAME="my-username"
export TAGOIO__RELAY__MQTT__PASSWORD="my-password"
export TAGOIO__RELAY__MQTT__BROKER_TLS_CA=""
export TAGOIO__RELAY__MQTT__BROKER_TLS_CERT=""
export TAGOIO__RELAY__MQTT__BROKER_TLS_KEY=""
```

### `config.toml`

The `config.toml` file contains the Relay parameters. Here is a reference:

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

## License

The TagoIO MQTT Relay is licensed under the Apache License. See the [LICENSE](./LICENSE) file for more details.

---

Thank you for using TagoIO MQTT Relay! If you have any questions or need further assistance, feel free to reach out via [GitHub Issues](https://github.com/tago-io/mqtt-relay/issues) or our [community forum](https://community.tago.io). 🚀