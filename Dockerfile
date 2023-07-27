# Use a Rust Docker image as the base image
FROM rust:1.71

WORKDIR /src

# Update Rust and install cargo contract
RUN rustup component add rust-src \
    && cargo install --force --version 3.1.0 cargo-contract

# Run the tests
CMD [ "cargo", "test"]
