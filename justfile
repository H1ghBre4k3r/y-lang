alias b := build
alias br := build-release
alias t := test

build:
    cargo build

build-release:
    cargo build --release

test:
    cargo test --workspace

bins:
    cargo build --bins

bins-release:
    cargo build --bins --release

watch:
    cargo watch -x "build --bins"

install:
    cargo install --path .

test-unit:
    cargo test --workspace --lib

test-integration:
    cargo test --workspace --tests

fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace --all-targets -- -D warnings

doc:
    cargo doc --workspace --no-deps

clean:
    cargo clean
