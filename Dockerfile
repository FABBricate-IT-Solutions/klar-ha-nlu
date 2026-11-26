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
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY custom_components/klar_nlu/infra_needles.txt custom_components/klar_nlu/infra_needles.txt
COPY scripts/third-party-notices.py scripts/third-party-notices.py
RUN apt-get update && apt-get install -y --no-install-recommends python3 \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release --locked \
    && python3 scripts/third-party-notices.py > THIRD_PARTY

FROM debian:bookworm-slim
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y --no-install-recommends \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates "openssl=3.0.22-1~deb12u2" "libssl3=3.0.22-1~deb12u2" \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/klar /usr/local/bin/klar
COPY --from=build /src/THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY --from=ui /ui/dist /usr/share/klar/ui
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY scripts/klar-entry.sh /usr/local/bin/klar-entry.sh
RUN chmod +x /usr/local/bin/klar-entry.sh
EXPOSE 10500 10520
# R-19: Supervisor and Docker both probe the public health document.
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD bash -c 'exec 3<>/dev/tcp/127.0.0.1/10520; printf "GET /api/v2/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n" >&3; grep -q "\"ok\":true" <&3'
ENTRYPOINT ["/usr/local/bin/klar-entry.sh"]
