alias b := build
alias br := build-release
alias t := test

build:
    cargo build

build-release:
    cargo build --release

test:
    cargo test --workspace
