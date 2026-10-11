FROM rust@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0

COPY --chmod=0555 yetlet-redis-qualification /usr/local/bin/yetlet-redis-qualification

USER 10001:10001
ENTRYPOINT ["/usr/local/bin/yetlet-redis-qualification"]
