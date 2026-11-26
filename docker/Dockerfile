FROM debian:bookworm-slim
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y --no-install-recommends \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates "openssl=3.0.22-1~deb12u2" "libssl3=3.0.22-1~deb12u2" \
    && rm -rf /var/lib/apt/lists/*
COPY klar /usr/local/bin/klar
COPY klar-entry.sh /usr/local/bin/klar-entry.sh
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY ui /usr/share/klar/ui
RUN chmod +x /usr/local/bin/klar /usr/local/bin/klar-entry.sh
EXPOSE 10500 10520
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD bash -c 'exec 3<>/dev/tcp/127.0.0.1/10520; printf "GET /api/v2/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n" >&3; grep -q "\"ok\":true" <&3'
ENTRYPOINT ["/usr/local/bin/klar-entry.sh"]
