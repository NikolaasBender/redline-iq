FROM rust:latest as builder
WORKDIR /usr/src/app

# Install dependencies if needed
# RUN apt-get update && apt-get install -y libpq-dev

# Copy over the source
COPY . .

# Expose port and run
EXPOSE 8080
CMD ["cargo", "run", "--release"]
