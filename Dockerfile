FROM rust:1.82 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY README.md ./README.md
COPY docs ./docs

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN useradd -m app

COPY --from=builder /app/target/release/messaging /usr/local/bin/messaging
COPY --from=builder /app/migrations /app/migrations

USER app
WORKDIR /app

ENV PORT=8080 \
    HOST=0.0.0.0 \
    SCHEME=http

EXPOSE 8080

CMD ["/usr/local/bin/messaging"]

