FROM rust:bookworm as sqlx
RUN cargo install sqlx-cli

FROM rust:bookworm as builder
WORKDIR /app

COPY --from=sqlx /usr/local/cargo/bin/sqlx /usr/local/cargo/bin/sqlx

ARG TARGET
ENV TARGET=$TARGET

RUN rustup update && rustup target add ${TARGET}

# build first just with the dependencies
# for caching and speed up the build 
RUN cargo init
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --target ${TARGET} --release

COPY . .
# otherwise cargo will not rebuild
RUN touch src/main.rs

# sqlx validates the sql queries at compile time
# against the database
# so this is required to build the application
ENV DATABASE_URL=sqlite://data.db
RUN sqlx database create\
  && sqlx migrate run

RUN cargo build --target ${TARGET} --release --bin patwoz-notify

FROM oven/bun:1.0.20 as web
WORKDIR /app
COPY . .
WORKDIR /app/web
ENV NODE_ENV=production
RUN bun install --frozen-lockfile
RUN bun run build

FROM ubuntu:latest as runtime
WORKDIR /app

ARG TARGET
ENV TARGET=$TARGET

COPY --from=web /app/web/dist /app/web/dist
COPY --from=builder /app/target/${TARGET}/release/patwoz-notify /app/patwoz-notify

RUN apt-get update\
  && apt-get install -y ca-certificates\
  && update-ca-certificates

ENTRYPOINT ["/app/patwoz-notify"]
