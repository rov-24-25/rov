FROM rust:latest

RUN apt update && apt full-upgrade -y

RUN rustup target add aarch64-unknown-linux-gnu
RUN apt install gcc-aarch64-linux-gnu