# patwoz-notify

Free to use notification service via web push notifications.

## Setup

```sh
cargo install sqlx-cli
sqlx database create
sqlx migrate run
```

## VAPID

```sh
bun x web-push generate-vapid-keys --json
# insert it into .env
```

## Development

```sh
# backend
cargo run

# frontend
cd web/
bun install
bun run dev
tailscale serve 5173
```
