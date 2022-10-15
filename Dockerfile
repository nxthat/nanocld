# Create Builder image
FROM rust:1.64.0-alpine3.16 AS builder

# Install required dependencies
RUN apk add openssl
RUN apk add libpq-dev
RUN apk add gcc
RUN apk add g++
RUN apk add make

# Create a non root user
RUN adduser --disabled-password nanocld
USER nanocld
# Copy dependency files
COPY --chown=nanocld ./Cargo.* /home/nanocld/
# Copy Source
COPY --chown=nanocld ./src /home/nanocld/src/
# Copy Sql migrations
COPY --chown=nanocld ./migrations /home/nanocld/migrations/
# Build
WORKDIR /home/nanocld
ENV RUSTFLAGS="-C target-feature=-crt-static"
ENV OPENSSL_STATIC=yes
RUN cargo build --release

# Create Production image
FROM alpine:3.16

RUN apk add openssl
RUN apk add libpq-dev
RUN apk add gcc g++

COPY --from=builder /home/nanocld/target/release/nanocld /bin/nanocld

RUN mkdir /run/nanocl
RUN chmod 777 /run/nanocl

ENTRYPOINT ["/bin/nanocld"]
