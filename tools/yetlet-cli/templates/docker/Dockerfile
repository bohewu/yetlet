# syntax=docker/dockerfile:1.7
# All three contexts MUST be sanitized and hash-verified before invoking BuildKit.
FROM --platform=linux/amd64 rust:1.99.0-bookworm AS builder
WORKDIR /work/yetlet
COPY --from=yetlet-src /Cargo.toml ./Cargo.toml
COPY --from=yetlet-src /crates/ ./crates/
WORKDIR /work/app
COPY --from=app-src /Cargo.lock ./Cargo.lock
COPY --from=app-src /src/main.rs ./src/main.rs
COPY cargo-overlay/Cargo.toml ./Cargo.toml
# --locked preserves the supplied lock; registry downloads may require network.
RUN cargo metadata --locked --format-version 1 > /dev/null
RUN cargo build --locked --release --bin @NAME@

FROM --platform=linux/amd64 debian:bookworm-slim AS runtime
COPY --from=builder /work/app/target/release/@NAME@ /usr/local/bin/yetlet-app
USER 65532:65532
ENV YETLET_HOST=0.0.0.0 YETLET_PORT=8080
EXPOSE 8080
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/yetlet-app"]
