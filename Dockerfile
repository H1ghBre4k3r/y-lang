ARG RUSTVERSION=1.67
ARG DEBIANVERSION=bullseye

FROM --platform=$BUILDPLATFORM rust:${RUSTVERSION}-${DEBIANVERSION} AS builder

ARG BUILDARCH
ARG TARGETARCH
ARG TARGETOS

# Store info about target triplet
RUN arch=$(echo ${TARGETARCH} | sed 's/arm64/aarch64/g' | sed 's/amd64/x86_64/g') \
  && vendor=unknown \
  && os="$(echo ${TARGETOS} | tr '[:upper:]' '[:lower:]')" \
  && abi=gnu \
  && target="$arch-$vendor-$os-$abi" \
  && echo "$arch" > /tmp/y-lang-rust-target-arch \
  && echo "$os" > /tmp/y-lang-rust-target-os \
  && echo "$abi" > /tmp/y-lang-rust-target-abi \
  && echo "$target" > /tmp/y-lang-rust-target

# Install the cross buildtools if needed (this most importantly contains the proper linker!)
RUN if [ ${BUILDARCH} != ${TARGETARCH} ]; then \
  apt-get update -y && apt-get install -y crossbuild-essential-$TARGETARCH; \
fi

# Install the target toolchain
RUN target="$(cat /tmp/y-lang-rust-target)" \
  && rustup target add "$target"

# Copy the sources
WORKDIR /opt/y-lang
COPY src src
COPY Cargo.toml Cargo.lock .

# Build the `why` compiler (and make sure that cargo knows about the correct linker)
RUN arch="$(cat /tmp/y-lang-rust-target-arch)" \
  && os="$(cat /tmp/y-lang-rust-target-os)" \
  && abi="$(cat /tmp/y-lang-rust-target-abi)" \
  && target="$(cat /tmp/y-lang-rust-target)" \
  && target_linker="$arch-$os-$abi-gcc" \
  && cargo build --release --target "$target" --config "target.$target.linker=\"$target_linker\"" \
  && mkdir -p bin \
  && cp target/"$target"/release/why bin

ARG DEBIANVERSION

FROM debian:${DEBIANVERSION}-slim

# Install runtime dependencies
RUN apt-get update -y && apt-get install -y nasm

# Copy the compiler
COPY --from=builder /opt/y-lang/bin/why /usr/local/bin/why

ENTRYPOINT ["/usr/local/bin/why"]
