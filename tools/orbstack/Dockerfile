# AIOS Development Environment for OrbStack
# This creates a development environment, not the actual OS

FROM ubuntu:22.04

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    unzip \
    qemu-system-x86 \
    qemu-utils \
    nasm \
    xorriso \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Node.js/Bun
RUN curl -fsSL https://bun.sh/install | bash
ENV PATH="/root/.bun/bin:${PATH}"

# Set working directory
WORKDIR /workspace

# Copy source code
COPY . /workspace

# Build kernel
WORKDIR /workspace/kernel
RUN cargo build --features alloc --release --target x86_64-aios.json || echo "Kernel build may need target setup"

# Build userland services
WORKDIR /workspace
RUN bun install
RUN bun run build || echo "Services build may need configuration"

WORKDIR /workspace

CMD ["/bin/bash"]

