FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM alpine:3.23 AS runtime

RUN adduser -D -u 10001 remoteconfig

WORKDIR /app

COPY --from=builder /app/target/release/remote-config /usr/local/bin/remote-config

USER remoteconfig

EXPOSE 6767

ENTRYPOINT ["/usr/local/bin/remote-config"]
CMD ["--config", "config/docker.toml"]
