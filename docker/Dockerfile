FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY klar /usr/local/bin/klar
RUN chmod +x /usr/local/bin/klar
EXPOSE 10500 10520
ENTRYPOINT ["/usr/local/bin/klar"]
CMD ["--config-dir", "/config"]
