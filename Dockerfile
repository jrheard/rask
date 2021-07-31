# See https://github.com/LukeMathWalker/cargo-chef#without-the-pre-built-image for context
FROM rust:1.54 as planner
WORKDIR /usr/rask
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.54 as cacher
WORKDIR /usr/rask
RUN cargo install cargo-chef
COPY --from=planner /usr/rask/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.54 as builder
WORKDIR /usr/rask
COPY . .
COPY --from=cacher /usr/rask/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
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
