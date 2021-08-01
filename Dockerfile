FROM rust:1.54 as builder
WORKDIR /usr/rask
COPY . .
RUN cargo build --release --bin rask_api

FROM debian:buster-slim as runtime
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/rask
COPY --from=builder /usr/rask/target/release/rask_api ./rask_api

RUN groupadd rask && useradd -g rask rask
RUN chown rask:rask ./rask_api
USER rask

WORKDIR /usr/rask
CMD ["./rask_api"]
