FROM rust:1.53 as planner
WORKDIR /usr/rask
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.53 as cacher
WORKDIR /usr/rask
RUN cargo install cargo-chef
COPY --from=planner /usr/rask/recipe.json recipe.json
# --release takes forever to compile so we just stick with a debug build for now
RUN cargo chef cook --recipe-path recipe.json

FROM rust:1.53 as builder
WORKDIR /usr/rask
COPY . .
COPY --from=cacher /usr/rask/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --bin rask_api

FROM debian:buster-slim as runtime
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/rask
COPY --from=builder /usr/rask/target/debug/rask_api ./rask_api

RUN groupadd rask && useradd -g rask rask
RUN chown rask:rask ./rask_api
USER rask

# TODO env vars?? or are they provided via docker-compose?
WORKDIR /usr/rask
CMD ["./rask_api"]
