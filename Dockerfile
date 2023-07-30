# Use a Rust Docker image as the base image
FROM rust:1.69

WORKDIR /src

# Update Rust and install cargo contract
RUN rustup component add rust-src \
    && cargo install --force --version 3.0.1 cargo-contract
