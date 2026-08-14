FROM rust:1.85-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY web ./web
RUN cargo build --release --locked || cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/klar /usr/local/bin/klar
EXPOSE 10500 10520
ENTRYPOINT ["/usr/local/bin/klar"]
CMD ["--config-dir", "/config"]
