# Use the official Rust image from the Docker Hub
FROM rust:1-slim-bookworm

# Install LLVM 15, clang, git, and other essentials
RUN apt-get update && apt-get install -y \
    llvm-15-dev \
    libclang-15-dev \
    clang-15 \
    pkg-config \
    libssl-dev \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Set up environment variables to point LLVM-sys to the correct installation
# On Linux, llvm-config-15 is used to determine paths for llvm-sys
ENV LLVM_SYS_150_PREFIX=/usr/lib/llvm-15

# Create and set the working directory
WORKDIR /usr/src/v-rust

# Copy the local project into the container
COPY . .

# Run the tests as the default command
# We use --release because building V tests can be slow otherwise
CMD ["cargo", "test", "--test", "official_suite", "--release"]
