FROM rust:1.79-bookworm as builder
WORKDIR /usr/src/app

COPY . .
COPY --from=d3fk/tailwindcss:stable /tailwindcss /usr/local/bin/tailwindcss
ENV SQLX_OFFLINE=true
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y procps ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/et-migrate /usr/local/bin/et-migrate
COPY --from=builder /usr/local/cargo/bin/et /usr/local/bin/et

ENTRYPOINT ["/usr/local/bin/et"]
