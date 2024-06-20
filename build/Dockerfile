# Install runtime dependencies
FROM debian:bookworm-slim
ARG TAGOIO_SOURCE_FOLDER="/tago-io"

RUN apt update
RUN apt install -y openssl build-essential netcat-traditional ca-certificates
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}

COPY /tagoio-relay .

RUN /tago-io/tagoio-relay init

EXPOSE 3001/tcp

ENTRYPOINT ["/tago-io/tagoio-relay"]
CMD ["start"]
