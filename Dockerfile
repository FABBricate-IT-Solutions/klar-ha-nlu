# Source build → static musl binary → scratch (non-root, labels, healthcheck).
FROM node:24-bookworm-slim AS ui
WORKDIR /ui
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web ./
# KlarBrand imports ../../../docs/klar-mark.png from web/src/components → /docs/...
COPY docs/klar-mark.png /docs/klar-mark.png
RUN npm run build

FROM rust:1.100-bookworm AS build
WORKDIR /src
RUN rustup target add x86_64-unknown-linux-musl \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        musl-tools python3 \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY custom_components/klar_nlu/infra_needles.txt custom_components/klar_nlu/infra_needles.txt
COPY scripts/third-party-notices.py scripts/third-party-notices.py
COPY web/package.json web/package-lock.json ./web/
RUN cargo build --release --locked --target x86_64-unknown-linux-musl \
    && python3 scripts/third-party-notices.py > THIRD_PARTY

FROM alpine:3.22 AS dirs
RUN mkdir -p /out/data /out/config \
    && chown -R 65532:65532 /out/data /out/config

FROM scratch
COPY --from=build /src/target/x86_64-unknown-linux-musl/release/klar /usr/local/bin/klar
COPY --from=build /src/THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY --from=ui /ui/dist /usr/share/klar/ui
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY --chown=65532:65532 --from=dirs /out/data /data
COPY --chown=65532:65532 --from=dirs /out/config /config

USER 65532:65532

EXPOSE 10500 10520

LABEL org.opencontainers.image.title="Klar NLU" \
      org.opencontainers.image.description="Deterministic NLU for Home Assistant" \
      org.opencontainers.image.source="https://github.com/FABBricate-IT-Solutions/klar-ha-nlu" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.vendor="FABBricate IT Solutions"

# R-19 / V-13: Supervisor and Docker both probe the public health document.
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD ["/usr/local/bin/klar", "healthcheck", "--http", "127.0.0.1:10520"]

ENTRYPOINT ["/usr/local/bin/klar"]
CMD ["--http", "0.0.0.0:10520", "--wyoming", "0.0.0.0:10500", "--config-dir", "/config", "--data-dir", "/data"]
