# Use the official Rust image for building
FROM rust:latest AS builder

# Set working directory
WORKDIR /app

# Clone the repository (replace with actual repository URL)
RUN git clone https://github.com/ExperimentalCyborg/discord-bot.git .

# Build the application in release mode
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates
RUN rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Create volume for persistent database storage
VOLUME /data

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/discord-bot /app/discord-bot

# Set environment variables
ENV DISCORD_BOT_TOKEN=""
ENV DB_PATH="/data/bot.sqlite"

# Create an entrypoint script
RUN echo '#!/bin/sh\n\
exec ./discord-bot --db-path "$DB_PATH" --bot-token "$DISCORD_BOT_TOKEN" "$@"' > /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
