# Changelog

All notable changes to Y Lang will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Y Lang implementation
- Compiler binary (`yc`)
- Formatter binary (`yfmt`)
- Language server binary (`yls`)

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0] - TBD

### Added
- Expression-centric language design
- Function declarations and lambda expressions
- Control flow with if/else expressions and while loops
- Struct declarations with field access
- Type inference and checking
- LLVM-based code generation
- Grammar definition using rust-sitter
- Comprehensive error reporting with spans
- Cross-platform compilation support

---

## Release Process

Releases are automated via GitHub Actions when tags are pushed:

1. **Regular Release**: `git tag v1.0.0 && git push origin v1.0.0`
2. **Pre-release**: `git tag v1.0.0-RC1 && git push origin v1.0.0-RC1`

The CI will automatically:
- Update version numbers in `Cargo.toml` files
- Build cross-platform binaries
- Create GitHub release with changelog
- Upload binary assets for all supported platforms

### Supported Platforms
- Linux x86_64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64