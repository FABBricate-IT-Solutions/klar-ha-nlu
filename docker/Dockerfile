# Pre-built musl binary + UI. Final stage is scratch (no OS, non-root).
# Build context is the `docker/` directory after CI copies klar, UI, notices.
FROM alpine:3.22 AS dirs
RUN mkdir -p /out/data /out/config \
    && chown -R 65532:65532 /out/data /out/config

FROM scratch

COPY klar /usr/local/bin/klar
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY ui /usr/share/klar/ui
COPY --chown=65532:65532 --from=dirs /out/data /data
COPY --chown=65532:65532 --from=dirs /out/config /config

# Distroless/nonroot uid; volume mounts supply /config and /data at runtime.
USER 65532:65532

EXPOSE 10500 10520

LABEL org.opencontainers.image.title="Klar NLU" \
      org.opencontainers.image.description="Deterministic NLU for Home Assistant" \
      org.opencontainers.image.source="https://github.com/FABBricate-IT-Solutions/klar-ha-nlu" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.vendor="FABBricate IT Solutions"

# No shell in scratch — probe through the binary (V-13).
HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
  CMD ["/usr/local/bin/klar", "healthcheck", "--http", "127.0.0.1:10520"]

ENTRYPOINT ["/usr/local/bin/klar"]
CMD ["--http", "0.0.0.0:10520", "--wyoming", "0.0.0.0:10500", "--config-dir", "/config", "--data-dir", "/data"]
