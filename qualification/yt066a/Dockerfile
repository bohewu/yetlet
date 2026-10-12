FROM rust@sha256:b36c246742b4d323472588f789601be34eb4af053255e2af53969078ee8dcec7

COPY --chmod=0555 yetlet-auth-notes /usr/local/bin/yetlet-auth-notes

USER 10001:10001
ENTRYPOINT ["/usr/local/bin/yetlet-auth-notes"]
