#!/bin/bash
# Ensure a platform is provided
if [ -z "$1" ]; then
  echo "Error: No distribution provided."
  exit 1
fi

# Ensure a version is provided
if [ -z "$2" ]; then
  echo "Error: No platform provided."
  exit 1
fi

# Ensure a version is provided
if [ -z "$3" ]; then
  echo "Error: No version provided."
  exit 1
fi

DISTRIBUTION=$1
PLATFORM=$2
FULL_VERSION=$3

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

echo "Building for $DISTRIBUTION on $PLATFORM"

# Construct the tags
if [ "$DISTRIBUTION" == "debian" ]; then
  TAGS="--tag tagoio/relay \
    --tag tagoio/relay:${DISTRIBUTION} \
    --tag tagoio/relay:${MAJOR}.${MINOR} \
    --tag tagoio/relay:${MAJOR}.${MINOR}.${PATCH} \
    --tag tagoio/relay:bookworm \
    --tag tagoio/relay:${MAJOR}.${MINOR}-bookworm \
    --tag tagoio/relay:${MAJOR}.${MINOR}.${PATCH}-bookworm"
elif [ "$DISTRIBUTION" == "alpine" ]; then
  TAGS="--tag tagoio/relay:${DISTRIBUTION} \
    --tag tagoio/relay:${MAJOR}.${MINOR}-${DISTRIBUTION} \
    --tag tagoio/relay:${MAJOR}.${MINOR}.${PATCH}-${DISTRIBUTION}"
else
  echo "Error: Unknown distribution."
  exit 1
fi

# Display the tags
echo "Tags to be used: $TAGS"

cd build
docker buildx build --push --build-arg TAGORELAY_VERSION=${FULL_VERSION} \
  --file Dockerfile.${DISTRIBUTION} \
  --platform ${PLATFORM} \
  $TAGS \
  .
