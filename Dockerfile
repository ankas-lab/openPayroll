# Use a Rust Docker image as the base image
FROM rust:1.69

# Clone the repository
COPY . ./

WORKDIR /src

# Update Rust and install cargo contract
RUN rustup update \
    && rustup component add rust-src \
    && cargo install --force --locked cargo-contract

# Run the tests
CMD [ "cargo", "test"]
