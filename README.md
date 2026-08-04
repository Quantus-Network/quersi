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
# GET http://127.0.0.1:6767/api/configs/wallet
```

## Hot reload

Every ~250ms the service re-reads the JSON file:

- Valid update → memory updated
- Invalid JSON → **last known good kept** (warn log)

Startup **fails hard** if the initial file is missing or invalid.

## Configuration

See [`config/example.toml`](config/example.toml) (local) and [`config/docker.toml`](config/docker.toml) (containers).

Environment overrides use the `REMOTE_CONFIG` prefix and `__` separator, e.g.:

```bash
REMOTE_CONFIG__SERVER__PORT=9090
REMOTE_CONFIG__REMOTE_CONFIGS__WALLET_CONFIGS_FILE=/path/to/flags.json
```

## Docker

The image ships the binary only. Mount host-managed TOML and JSON at runtime (same pattern IAC should use).

```bash
docker build -t remote-config .
docker run --rm -p 6767:6767 \
  -v "$(pwd)/wallet_config.json:/app/wallet_config.json:ro" \
  -v "$(pwd)/config/docker.toml:/app/config/docker.toml:ro" \
  remote-config
```

- Default command: `--config config/docker.toml` (expects that path to be mounted; listens on `0.0.0.0:6767`)
- Mount `wallet_config.json` at `/app/wallet_config.json` (path from the TOML)
- Mount configs **read-only**; the image runs as uid `10001`
- If host files are mode `0640`, run the container as the file owner (IAC pattern: set `user:` to the admin UID) so the process can read them

## Development

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
