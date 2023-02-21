ARG RUSTVERSION=1.67
ARG DEBIANVERSION=bullseye

FROM --platform=$BUILDPLATFORM rust:${RUSTVERSION}-${DEBIANVERSION} AS builder

ARG TARGETARCH
ARG TARGETOS

# Install the target toolchain
RUN arch=$(echo ${TARGETARCH} | sed "s/arm64/aarch64/g" | sed "s/amd64/x86_64/g") \
  && vendor=unknown \
  && os=$(echo ${TARGETOS} | tr '[:upper:]' '[:lower:]') \
  && abi=gnu \
  && target="$arch-$vendor-$os-$abi" \
  && echo "$target" > /tmp/y-lang-rust-target-toolchain \
  && rustup target add "$target"

# Copy the sources
WORKDIR /opt/y-lang
COPY src src
COPY Cargo.toml Cargo.lock .

# Build the compiler
RUN target="$(cat /tmp/y-lang-rust-target-toolchain)" \
  && cargo build --release --target "$target" \
  && mkdir -p bin \
  && cp target/"$target"/release/why bin

ARG DEBIANVERSION

FROM debian:${DEBIANVERSION}-slim

# Install runtime dependencies
RUN apt-get update -y && apt-get install -y nasm

# Copy the compiler
COPY --from=builder /opt/y-lang/bin/why /usr/local/bin/why

ENTRYPOINT ["/usr/local/bin/why"]
