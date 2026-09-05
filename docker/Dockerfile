FROM debian:bookworm-slim
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y --no-install-recommends \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY klar /usr/local/bin/klar
COPY klar-entry.sh /usr/local/bin/klar-entry.sh
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY ui /usr/share/klar/ui
RUN chmod +x /usr/local/bin/klar /usr/local/bin/klar-entry.sh
EXPOSE 10500 10520
ENTRYPOINT ["/usr/local/bin/klar-entry.sh"]
