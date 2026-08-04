# remote-config

Minimal Axum service that serves wallet feature flags over HTTP.

This is a standalone extract of the wallet remote-config feature from `task-master`. **task-master is unchanged** and continues to serve its own copy until clients cut over.

## Design

Feature flags are intentionally **public to read**. There is **no write/admin HTTP API** — updates are done by replacing the JSON file on disk (hot-reloaded).

Signing config files does not help against a compromised host: anyone who can replace the service binary (or its config) can serve arbitrary responses anyway. Trust is host/deploy integrity, not a signature on the JSON.

```text
Operator/CI --> config.json --> VPS mount
                              |
                    remote-config (load / hot-reload)
                              |
                    GET /api/configs/wallet
```

## API

| Method | Path | Auth | Response |
|--------|------|------|----------|
| `GET` | `/api/configs/wallet` | none | `{ "data": { ...flags } }` |
| `GET` | `/health` | none | `200` if a config is loaded |

Example payload:

```json
{
  "data": {
    "enableTestButtons": false,
    "enableKeystoneHardwareWallet": false,
    "enableHighSecurity": true,
    "enableRemoteNotifications": true,
    "enableSwap": true
  }
}
```

Payload is opaque JSON: add or change keys by updating the file — no service redeploy required. Only invalid JSON is rejected.

## Quick start

```bash
cargo run -- --config config/default.toml
# GET http://127.0.0.1:8080/api/configs/wallet
```

## Hot reload

Every ~250ms the service re-reads the JSON file:

- Valid update → memory updated
- Invalid JSON → **last known good kept** (warn log)

Startup **fails hard** if the initial file is missing or invalid.

## Configuration

See [`config/example.toml`](config/example.toml).

Environment overrides use the `REMOTE_CONFIG` prefix and `__` separator, e.g.:

```bash
REMOTE_CONFIG__SERVER__PORT=9090
REMOTE_CONFIG__REMOTE_CONFIGS__WALLET_CONFIGS_FILE=/path/to/flags.json
```

## Docker

```bash
docker build -t remote-config .
docker run --rm -p 8080:8080 \
  -v "$(pwd)/configs:/app/configs:ro" \
  -v "$(pwd)/config/default.toml:/app/config/default.toml:ro" \
  remote-config
```

The image runs as a non-root user. Mount configs **read-only**.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
