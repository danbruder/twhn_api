FROM rust:1.58.1-buster as chef
WORKDIR /app
RUN cargo install cargo-chef 

FROM chef as planner
WORKDIR /app
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS runtime
RUN  apt-get update -y \
    &&  apt-get install awscli sqlite3 -y \
    &&  apt-get autoremove -y \
    &&  apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/twhn_api /usr/local/bin/twhn_api
COPY --from=builder /app/migrations /app/migrations
EXPOSE 8000
CMD ["twhn_api"]
