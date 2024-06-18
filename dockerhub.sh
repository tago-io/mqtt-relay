#!/bin/bash
# Ensure a version is provided
if [ -z "$1" ]; then
  echo "Error: No version provided."
  exit 1
fi

FULL_VERSION=$1

SPLIT=(${FULL_VERSION//./ })
MAJOR=${SPLIT[0]}
MINOR=${SPLIT[1]:-0} # set to 0 if no minor version is provided
PATCH=${SPLIT[2]:-0} # set to 0 if no patch version is provided

# Validate MAJOR, MINOR, and PATCH to ensure they are numeric
if ! [[ "$MAJOR" =~ ^[0-9]+$ ]]; then
  echo "Error: MAJOR version is not a number."
  exit 1
fi

if ! [[ "$MINOR" =~ ^[0-9]+$ ]]; then
  echo "Error: MINOR version is not a number."
  exit 1
fi

if ! [[ "$PATCH" =~ ^[0-9]+$ ]]; then
  echo "Error: PATCH version is not a number."
  exit 1
fi

# Ensure CARGO_SERVER_SSL_CA, CARGO_SERVER_SSL_CERT, and CARGO_SERVER_SSL_KEY are set
if [ -z "$CARGO_SERVER_SSL_CA" ] || [ -z "$CARGO_SERVER_SSL_CERT" ] || [ -z "$CARGO_SERVER_SSL_KEY" ]; then
  echo "Error: SSL environment variables are not set."
  exit 1
fi

CARGO_SERVER_SSL_CA_BASE64=$(echo "$CARGO_SERVER_SSL_CA" | base64 -w 0)
CARGO_SERVER_SSL_CERT_BASE64=$(echo "$CARGO_SERVER_SSL_CERT" | base64 -w 0)
CARGO_SERVER_SSL_KEY_BASE64=$(echo "$CARGO_SERVER_SSL_KEY" | base64 -w 0)

# Alpine
# docker buildx build --push --build-arg TAGORELAY_VERSION=${FULL_VERSION} \
#   --platform linux/arm64/v8,linux/amd64 \
#   --tag tagoio/tagorelay:alpine \
#   --tag tagoio/tagorelay:${MAJOR}.${MINOR}-alpine \
#   --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH}-alpine .

# Debian
docker buildx build --push --build-arg TAGORELAY_VERSION=${FULL_VERSION} \
  --build-arg CARGO_SERVER_SSL_CA=${CARGO_SERVER_SSL_CA_BASE64} \
  --build-arg CARGO_SERVER_SSL_CERT=${CARGO_SERVER_SSL_CERT_BASE64} \
  --build-arg CARGO_SERVER_SSL_KEY=${CARGO_SERVER_SSL_KEY_BASE64} \
  --platform linux/arm/v7,linux/arm64/v8,linux/amd64 \
  --tag tagoio/tagorelay \
  --tag tagoio/tagorelay:debian \
  --tag tagoio/tagorelay:bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}-bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH}-bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR} \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH} \
  .
