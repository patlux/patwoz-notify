# patwoz-notify

Free to use real-time notification service via web push notifications.

Main purpose is for me to learn rust.

## Development

```sh
# backend (http://localhost:3000)
cargo run

# frontend (http://localhost:5173)
cd web/
bun run dev
```

**iOS, Android**

To test it locally on your phone while developing, you need a valid ssl certificate to be able to use web push notifications.

I'm using [tailscale serve](https://tailscale.com/kb/1242/tailscale-serve/) for this. But you can also use something like [Cloudflare Tunnel](https://www.cloudflare.com/products/tunnel/).

```sh
tailscale serve 5173
```

Now open https://`<hostname>`.`<tailscale-domain>`.ts.net

## Setup Development

**Database**

```sh
cargo install sqlx-cli
sqlx database create
sqlx migrate run
```

**VAPID**

```sh
bun x web-push generate-vapid-keys --json
# insert the private and public key into .env
```

**Frontend**

```sh
cd web/
bun install
```

## Migrations

See [sqlx-cli](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)

