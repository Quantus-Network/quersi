FROM rust:1.97-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static perl make

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

RUN cargo build --release --locked

FROM alpine:3.23 AS runtime

RUN useradd --system --create-home --uid 10001 remoteconfig \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/remote-config /usr/local/bin/remote-config
COPY configs ./configs
COPY config ./config

USER remoteconfig

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/remote-config"]
CMD ["--config", "config/default.toml"]
