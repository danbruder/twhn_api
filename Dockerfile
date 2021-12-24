# Backend ---------------
FROM rust:1.57.0 as backend-planner
WORKDIR /app
RUN cargo install cargo-chef 
COPY backend .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.57.0 as backend-cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=backend-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Backend Builder. Using the same image to save download times, although
# the base rust image probably shares layers with the node version
FROM rust:1.57.0 AS backend-builder
WORKDIR /app
COPY . .
COPY --from=backend-cacher /app/target target
COPY --from=backend-cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

# Runtime stage
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates openssl libpq-dev \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/target/release/twhn_api twhn_api
EXPOSE 8000
CMD ["twhn_api"]
