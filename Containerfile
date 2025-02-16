# Build stage - cargo-chef
FROM rust:1.81-alpine3.20 AS chef
RUN apk add --no-cache musl-dev
WORKDIR /app
RUN cargo install cargo-chef
COPY . .

# Compute recipe
FROM chef AS planner
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
FROM rust:1.81-alpine3.20 AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY --from=cacher /app/target target
COPY ./ /app
RUN cargo build --release
RUN strip target/release/qr-248-no

# Prod stage
FROM alpine:3.20
RUN apk add --no-cache libgcc
COPY --from=builder /app/target/release/qr-248-no /
ENTRYPOINT ["/qr-248-no"]
