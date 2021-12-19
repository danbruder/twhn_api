FROM rust:1 as builder
WORKDIR /app
COPY . .
RUN cargo install --path .


FROM debian:buster-slim as runner
COPY --from=builder /usr/local/cargo/bin/twhn_api /usr/local/bin/twhn_api
EXPOSE 8000
CMD ["twhn_api"]
