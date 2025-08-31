# Use the official Rust image as the base image
FROM rust:1.85-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy full project (respecting .dockerignore)
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage for the runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false kubeatlas

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/kubeatlas-backend /app/kubeatlas-backend

# Change ownership to the non-root user and make executable
RUN chown kubeatlas:kubeatlas /app/kubeatlas-backend && chmod +x /app/kubeatlas-backend

# Switch to the non-root user (temporarily commented for debugging)
# USER kubeatlas

# Expose the port
EXPOSE 3001

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_ADDRESS=0.0.0.0:3001

# Run the application
CMD ["/app/kubeatlas-backend"]
