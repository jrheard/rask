## Builder stage
FROM rust:1.53 as builder

# Built-dep-caching approach via https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/
WORKDIR /usr/src/rask
COPY ./Cargo.toml ./Cargo.toml

RUN USER=root cargo new --bin rask_api
COPY ./rask_api/Cargo.toml ./rask_api/Cargo.toml

RUN cargo build --release --bin rask_api
RUN rm ./rask_api/src/*.rs

ADD . ./

RUN rm ./target/release/deps/rask_api*
RUN cargo build --release --bin rask_api

## Final stage
FROM debian:buster-slim
#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/rask
COPY --from=builder /usr/src/rask/target/release/rask_api ./rask_api

RUN chown rask:rask ./rask_api
USER rask

# TODO env vars?? or are they provided via docker-compose?
CMD ["rask_api"]