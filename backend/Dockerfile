# Build
FROM rust:1.75-slim as build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
COPY . .
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim
RUN useradd -m appuser
WORKDIR /app
COPY --from=build /app/target/release/pulsesense-backend /app/pulsesense-backend
EXPOSE 8080
USER appuser
CMD ["/app/pulsesense-backend"]
