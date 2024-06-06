FROM rust:slim-bookworm as build
ARG TAGOIO_SOURCE_FOLDER="/tago-io"

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
COPY --from=build ${TAGOIO_SOURCE_FOLDER}/target/release/tagoio-mqtt-relay .
COPY --from=build ${TAGOIO_SOURCE_FOLDER}/config.toml config.toml

ENTRYPOINT ["/tago-io/tagoio-mqtt-relay"]
