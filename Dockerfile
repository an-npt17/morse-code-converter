# syntax=docker/dockerfile:1

# Use arm64 Ubuntu base
FROM --platform=linux/arm64 ubuntu:22.04 as build

RUN apt-get update && apt-get install -y \
  curl build-essential pkg-config libssl-dev git ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a new user (optional, avoid root)
RUN useradd -ms /bin/bash builder
USER builder
WORKDIR /home/builder/app

COPY --chown=builder . .
