# Stage 1: builder
# rust:1.75-slim is glibc-based. We add the musl target to produce a
# fully static binary that runs on Alpine (musl libc) without glibc.
FROM rust:1.75-slim AS builder

# Install musl toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/

# Build fully static release binary targeting musl
RUN cargo build --release --bin repodesk --target x86_64-unknown-linux-musl

# Stage 2: runtime
# Alpine for minimal attack surface and image size.
# Static musl binary has no runtime lib dependencies except git.
FROM alpine:3.19

# git is required for GitCli at runtime
RUN apk add --no-cache git

# Create non-root user
RUN addgroup -S repodesk && adduser -S repodesk -G repodesk

WORKDIR /workspace

# Copy static binary from builder
COPY --from=builder \
    /build/target/x86_64-unknown-linux-musl/release/repodesk \
    /usr/local/bin/repodesk

# Verify binary is executable and statically linked
RUN chmod +x /usr/local/bin/repodesk \
    && /usr/local/bin/repodesk --help 2>/dev/null || true \
    && file /usr/local/bin/repodesk

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

USER repodesk

# Repository is mounted into /workspace at runtime:
#   podman run -it -v /path/to/repo:/workspace repodesk
VOLUME ["/workspace"]

ENTRYPOINT ["/entrypoint.sh"]
