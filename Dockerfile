FROM rust:slim-bookworm as build
ARG TAGOIO_SOURCE_FOLDER="/tago-io"
ARG TAGORELAY_VERSION
ARG CARGO_SERVER_SSL_CA
ARG CARGO_SERVER_SSL_CERT
ARG CARGO_SERVER_SSL_KEY

# Decode the base64 encoded environment variables
RUN export CARGO_SERVER_SSL_CA=$(echo "${CARGO_SERVER_SSL_CA}" | base64 -d)
RUN export CARGO_SERVER_SSL_CERT=$(echo "${CARGO_SERVER_SSL_CERT}" | base64 -d)
RUN export CARGO_SERVER_SSL_KEY=$(echo "${CARGO_SERVER_SSL_KEY}" | base64 -d)

RUN apt update
RUN apt install -y protobuf-compiler libssl-dev gcc pkg-config build-essential

RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}
ADD . ${TAGOIO_SOURCE_FOLDER}

RUN cargo build --release

FROM debian:bookworm-slim
ARG TAGOIO_SOURCE_FOLDER="/tago-io"

RUN apt update
RUN apt install -y build-essential netcat-traditional ca-certificates
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}
COPY --from=build ${TAGOIO_SOURCE_FOLDER}/target/release/tagoio-relay .

RUN /tago-io/tagoio-relay init

ENTRYPOINT ["/tago-io/tagoio-relay"]
CMD ["start"]
