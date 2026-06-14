# Stage 1: builder
# Rust 1.75 matches MSRV and Ubuntu 24.04 apt toolchain.
FROM rust:1.75-slim AS builder

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build --release --bin repodesk

# Stage 2: runtime
# Alpine for minimal attack surface and image size.
FROM alpine:3.19

# git is required for GitCli at runtime
RUN apk add --no-cache git

# Create non-root user
RUN addgroup -S repodesk && adduser -S repodesk -G repodesk

WORKDIR /workspace

# Copy binary from builder
COPY --from=builder /build/target/release/repodesk /usr/local/bin/repodesk

# Copy entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

USER repodesk

# Repository is mounted into /workspace at runtime:
#   podman run -it -v /path/to/repo:/workspace repodesk
VOLUME ["/workspace"]

ENTRYPOINT ["/entrypoint.sh"]
