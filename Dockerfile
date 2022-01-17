FROM rust:1.57.0 as planner
WORKDIR /app
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:1.57.0 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.57.0 AS builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM rust:1.57.0 AS runtime
RUN apt-get update && apt-get install sqlite3
WORKDIR /app
EXPOSE 8000
COPY --from=builder /app/target/release/twhn_api /usr/local/bin/twhn_api
COPY --from=builder /app/migrations /app/migrations
CMD ["twhn_api"]
