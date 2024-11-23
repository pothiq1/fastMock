# Stage 1: Build the application using the Rust slim image
FROM rust:slim-buster as builder

# Install necessary build tools and OpenSSL development libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy src directory to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release && rm -rf target/release/build

# Remove the dummy src directory and copy the actual source code
RUN rm -rf src
COPY src ./src

# Copy the static files
COPY static ./static

# Build the application in release mode
RUN cargo build --release && \
    strip target/release/OMock && \
    rm -rf target/release/{build,deps,examples,incremental} && \
    rm -rf /usr/local/cargo/registry /usr/local/cargo/git

# Stage 2: Create the runtime image using Debian slim for minimal size
FROM debian:buster-slim

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/OMock ./mock-api-manager

# Copy the static files directory to the runtime image
COPY --from=builder /app/static ./static

# Expose the application's port
EXPOSE 8080

# Set the entrypoint command
ENTRYPOINT ["./mock-api-manager"]