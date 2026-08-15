FROM rust:1.85-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY web ./web
COPY scripts/third-party-notices.py scripts/third-party-notices.py
RUN apt-get update && apt-get install -y --no-install-recommends python3 \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release --locked \
    && python3 scripts/third-party-notices.py > THIRD_PARTY

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/klar /usr/local/bin/klar
COPY --from=build /src/THIRD_PARTY /usr/share/doc/klar/THIRD_PARTY
COPY LICENSE /usr/share/doc/klar/LICENSE
COPY scripts/klar-entry.sh /usr/local/bin/klar-entry.sh
RUN chmod +x /usr/local/bin/klar-entry.sh
EXPOSE 10500 10520
ENTRYPOINT ["/usr/local/bin/klar-entry.sh"]
