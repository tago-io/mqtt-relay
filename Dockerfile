FROM rust:slim-bookworm as build
ARG TAGOIO_SOURCE_FOLDER="/tago-io"
ARG TAGORELAY_VERSION
ARG CARGO_SERVER_SSL_CA
ARG CARGO_SERVER_SSL_CERT
ARG CARGO_SERVER_SSL_KEY

# Decode the base64 encoded environment variables and validate that they are set
RUN export CARGO_SERVER_SSL_CA=$(echo "${CARGO_SERVER_SSL_CA}" | base64 -d) && \
    export CARGO_SERVER_SSL_CERT=$(echo "${CARGO_SERVER_SSL_CERT}" | base64 -d) && \
    export CARGO_SERVER_SSL_KEY=$(echo "${CARGO_SERVER_SSL_KEY}" | base64 -d) && \
    /bin/bash -c 'if [ -z "$CARGO_SERVER_SSL_CA" ]; then echo "Error: CARGO_SERVER_SSL_CA is not set"; exit 1; fi && \
    if [ -z "$CARGO_SERVER_SSL_CERT" ]; then echo "Error: CARGO_SERVER_SSL_CERT is not set"; exit 1; fi && \
    if [ -z "$CARGO_SERVER_SSL_KEY" ]; then echo "Error: CARGO_SERVER_SSL_KEY is not set"; exit 1; fi'

# Install dependencies
RUN apt update
RUN apt install -y protobuf-compiler libssl-dev gcc musl-dev pkg-config build-essential libc6-dev-arm64-cross cmake
# TODO: For armv7l, need to use custom version for openssl. We may need to make a separated Dockerfile for armv7l
# RUN apt install -y protobuf-compiler libssl-dev gcc musl-dev pkg-config build-essential libc6-dev-arm64-cross cmake clang llvm-dev

# Set up the build environment
RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}
ADD . ${TAGOIO_SOURCE_FOLDER}

RUN touch .env

# Determine the target based on the platform
RUN set -e; \
    TARGET=$(uname -m); \
    if [ "$TARGET" = "x86_64" ]; then \
        TARGET="x86_64-unknown-linux-gnu"; \
    elif [ "$TARGET" = "aarch64" ]; then \
        TARGET="aarch64-unknown-linux-gnu"; \
    elif [ "$TARGET" = "armv7l" ]; then \
        TARGET="armv7-unknown-linux-gnueabihf" \
        export LIBCLANG_PATH=$(llvm-config --libdir); \
    else \
        echo "Unsupported architecture: $TARGET"; exit 1; \
    fi; \
    rustup target add $TARGET && \
    cargo build --release --target $TARGET

# Unset the SSL environment variables
RUN unset CARGO_SERVER_SSL_CA CARGO_SERVER_SSL_CERT CARGO_SERVER_SSL_KEY

# Install runtime dependencies
FROM debian:bookworm-slim
ARG TAGOIO_SOURCE_FOLDER="/tago-io"

RUN apt update
RUN apt install -y openssl build-essential netcat-traditional ca-certificates
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}

# Copy the target directory from the build stage
COPY --from=build ${TAGOIO_SOURCE_FOLDER}/target/*/release/tagoio-relay .

RUN /tago-io/tagoio-relay init

ENTRYPOINT ["/tago-io/tagoio-relay"]
CMD ["start"]
