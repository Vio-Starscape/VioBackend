FROM rust:1.76 as builder

RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/vio_api* || true
RUN cargo build --release

FROM rust:1.76-slim

COPY --from=builder /app/target/release/vio_api /usr/local/bin/vio_api

CMD ["vio_api"]
