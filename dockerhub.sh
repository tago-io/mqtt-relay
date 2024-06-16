#!/bin/bash

FULL_VERSION=$1

SPLIT=(${FULL_VERSION//./ })
MAJOR=${SPLIT[0]}
MINOR=${SPLIT[1]}
PATCH=${SPLIT[2]}

# Alpine
# docker buildx build --push --build-arg TAGORELAY_VERSION=${FULL_VERSION} \
#   --platform linux/arm64/v8,linux/amd64 \
#   --tag tagoio/tagorelay:alpine \
#   --tag tagoio/tagorelay:${MAJOR}.${MINOR}-alpine \
#   --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH}-alpine .

# Debian
docker buildx build --push --build-arg TAGORELAY_VERSION=${FULL_VERSION} \
  --platform linux/arm/v7,linux/arm64/v8,linux/amd64 \
  --tag tagoio/tagorelay \
  --tag tagoio/tagorelay:debian \
  --tag tagoio/tagorelay:bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}-bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH}-bullseye \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR} \
  --tag tagoio/tagorelay:${MAJOR}.${MINOR}.${PATCH} .
