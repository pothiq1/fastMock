# Stage 1: Build the application using the Rust image with musl
FROM rust:latest as builder

# Install musl target for Rust and musl-tools for musl-gcc
RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y musl-tools pkg-config

# Set environment variables for openssl-sys
ENV OPENSSL_STATIC=1
ENV OPENSSL_VENDORED=1
ENV PKG_CONFIG_ALLOW_CROSS=1

# Set the working directory inside the container
WORKDIR /app

# Copy Cargo.toml and Cargo.lock first to cache dependencies
COPY fastMock/Cargo.toml fastMock/Cargo.lock ./

# Create a dummy src directory to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release --target x86_64-unknown-linux-musl

# Remove the dummy src directory and copy the actual source code
RUN rm -rf src
COPY fastMock/src ./src

# Copy the static files from root level
COPY fastMock/static ./static

# Build the application in release mode with musl target
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Create the runtime image using scratch
FROM scratch

# Set the working directory
WORKDIR /app

# Copy the statically linked binary from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/OMock ./mock-api-manager

# Copy the static files directory to the final image
COPY --from=builder /app/static ./static

# Expose the application's port
EXPOSE 8080

# Set the entrypoint command
ENTRYPOINT ["./mock-api-manager"]