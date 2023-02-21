ARG RUSTVERSION=1.67
ARG DEBIANVERSION=bullseye

FROM rust:${RUSTVERSION}-${DEBIANVERSION} AS builder

# Copy the sources
WORKDIR /opt/y-lang
COPY src src
COPY Cargo.toml Cargo.lock .

# Build the compiler
RUN cargo build --release

FROM debian:${DEBIANVERSION}-slim

# Install runtime dependencies
RUN apt-get update -y && apt-get install -y nasm

# Copy the compiler
COPY --from=builder /opt/y-lang/target/release/why /usr/local/bin/why

ENTRYPOINT ["/usr/local/bin/why"]
