# Use the official Rust image as the base
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image
FROM debian:buster-slim

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/OMock /OMock

# Expose the application port
EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/OMock"]