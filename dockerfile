# Compile the application with the official Rust Docker image
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Use a smaller base for the final image
FROM debian:trixie-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Volume for default SQLite database storage (overridden by compose file if that's used)
VOLUME /data

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/discord-bot /app/discord-bot
COPY --from=builder /app/fortune.json /app/

# Set environment variable defaults. Don't forget to override this with your own bot token
ENV DISCORD_BOT_TOKEN=""
ENV DB_PATH="/data/bot.sqlite"
ENV OLLAMA_URL=""
ENV OLLAMA_MODEL=""
ENV AI_FORTUNETELLER_PERSONA=""
ENV AI_FORTUNETELLER_THINKING=0
ENV AI_CHAT_PERSONA=""
ENV AI_CHAT_THINKING=0

# Create an entrypoint script
RUN echo '#!/bin/sh\n\
exec ./discord-bot \
--db-path "$DB_PATH" \
--bot-token "$DISCORD_BOT_TOKEN" \
--fortune-cooldown $FORTUNE_COOLDOWN \
"$@"' > /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
