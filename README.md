# remote-config

Minimal Axum service that serves wallet feature flags over HTTP.

## Design

Feature flags are intentionally **public to read**. There is **no write/admin HTTP API** — updates are done by replacing the JSON file on disk (hot-reloaded).

Signing config files does not help against a compromised host: anyone who can replace the service binary (or its config) can serve arbitrary responses anyway. Trust is host/deploy integrity, not a signature on the JSON.

```text
Operator/CI --> wallet_config.json --> VPS mount
                                    |
                          remote-config (load / hot-reload)
                                    |
                          GET /api/configs/wallet
```

## API

| Method | Path | Auth | Response |
|--------|------|------|----------|
| `GET` | `/api/configs/wallet` | none | `{ "data": { ...flags } }` |
| `GET` | `/health` | none | `{ "healthy": true, "service": "RemoteConfig", "version": "…" }` |

Example wallet payload:

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

CORS is off by default (`cors_allowed_origins = []`). Set origins in TOML when browser clients need cross-origin access.

## Quick start

`wallet_config.json` is gitignored (host-managed). Copy the example first:

```bash
cp wallet_config.example.json wallet_config.json
cargo run -- --config config/default.toml
# GET http://127.0.0.1:6767/api/configs/wallet
# GET http://127.0.0.1:6767/health
```

## Hot reload

Every ~250ms the service re-reads the JSON file:

- Valid update → memory updated
- Invalid JSON → **last known good kept** (warn log)

Startup **fails hard** if the initial file is missing or invalid.

## Configuration

| File | Purpose |
|------|---------|
| [`config/default.toml`](config/default.toml) | Local defaults (`127.0.0.1:6767`) |
| [`config/docker.toml.example`](config/docker.toml.example) | Container template (`0.0.0.0:6767`) |

Paths in TOML that are relative are resolved against the TOML file’s directory.

`config/docker.toml` and `wallet_config.json` are gitignored — create them on the host (or in CI) and bind-mount them. For containers:

```bash
cp config/docker.toml.example config/docker.toml
```

Environment overrides use the `REMOTE_CONFIG` prefix and `__` separator, e.g.:

```bash
REMOTE_CONFIG__SERVER__PORT=9090
REMOTE_CONFIG__REMOTE_CONFIGS__WALLET_CONFIGS_FILE=/path/to/flags.json
```

## Docker

The image ships the binary only. Mount host-managed TOML and JSON at runtime (same pattern IAC should use).

```bash
cp config/docker.toml.example config/docker.toml
cp wallet_config.example.json wallet_config.json

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
