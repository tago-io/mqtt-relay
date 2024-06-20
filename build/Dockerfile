# Install runtime dependencies
FROM debian:bookworm-slim
ARG TAGOIO_SOURCE_FOLDER="/tago-io"

RUN apt update
RUN apt install -y openssl build-essential netcat-traditional ca-certificates --no-install-recommends
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

RUN mkdir -p ${TAGOIO_SOURCE_FOLDER}
WORKDIR ${TAGOIO_SOURCE_FOLDER}

COPY /tagoio-relay .

# Add execute permissions to the tagoio-relay file
RUN chmod +x /tago-io/tagoio-relay

RUN /tago-io/tagoio-relay init

EXPOSE 3001/tcp

ENTRYPOINT ["/tago-io/tagoio-relay"]
CMD ["start"]
