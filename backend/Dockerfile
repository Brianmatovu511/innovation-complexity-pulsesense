# syntax=docker/dockerfile:1

############################
# Build stage
############################
FROM rust:1.83-slim AS build
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY tests ./tests

RUN cargo build --release --bin pulsesense-backend
RUN cargo build --release --bin simulator

############################
# Runtime stage
############################
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install curl for compose healthcheck, plus certs + openssl runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 10001 appuser

COPY --from=build /app/target/release/pulsesense-backend /usr/local/bin/pulsesense-backend
COPY --from=build /app/target/release/simulator /usr/local/bin/simulator

EXPOSE 8080
USER appuser

# IMPORTANT: remove Dockerfile HEALTHCHECK (it used bash which isn't installed)
CMD ["/usr/local/bin/pulsesense-backend"]
