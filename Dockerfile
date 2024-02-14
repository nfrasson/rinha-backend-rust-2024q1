FROM rust:1-slim-buster as builder
RUN USER=root cargo new --bin rinha
WORKDIR /rinha
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN mkdir src/bin
COPY ./src ./src
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /rinha/target/release/rinha /usr/local/bin/rinha
CMD ["rinha"]
