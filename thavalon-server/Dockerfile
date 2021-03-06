# syntax=docker/dockerfile:experimental
FROM rust:1.46 as builder

COPY . /app
WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo install --path .

FROM debian:buster-slim

RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/thavalon-server /usr/local/bin/thavalon-server

CMD ["/usr/local/bin/thavalon-server"]