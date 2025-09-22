# Changelog

All notable changes to Y Lang will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Deprecated

### Removed

### Fixed

### Security

## [0.1.0] - TBD

### Added
- Initial Y Lang implementation with expression-centric language design
- Compiler binary (`yc`) for compiling Y Lang source code
- Formatter binary (`yfmt`) for code formatting and pretty-printing
- Language server binary (`yls`) for IDE integration and tooling support
- Function declarations and lambda expressions with first-class support
- Control flow with if/else expressions and while loops
- Struct declarations with field access and manipulation
- Type inference and comprehensive type checking system
- LLVM-based code generation for optimized native code output
- Grammar definition using rust-sitter for robust parsing
- Comprehensive error reporting with span information and colored output
- Cross-platform compilation support (Linux, macOS, Windows)
- Automated release workflow with version management
- Cross-platform binary distribution

---

## 🤖 Automated Release Process

Releases are fully automated via GitHub Actions. Simply create and push a tag to trigger the release workflow:

### Creating Releases

**Regular Release:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

**Pre-Release (Release Candidate):**
```bash
git tag v1.0.0-RC1
git push origin v1.0.0-RC1
```

### What Happens Automatically

The release workflow performs the following steps:

1. **📝 Version Management**
   - Extracts version from tag (`v1.0.0` → `1.0.0`)
   - Updates `Cargo.toml` and `crates/why_lib/Cargo.toml` with new version
   - Validates version consistency across all files

2. **📚 Changelog Updates**
   - Generates changelog entries from git commit history
   - Updates this `CHANGELOG.md` file automatically
   - Moves content from "Unreleased" to the new version section
   - Resets "Unreleased" section for future development

3. **🏷️ Git Management**
   - Commits version and changelog changes
   - Amends the tag to point to the updated commit
   - Ensures git history remains clean and consistent

4. **🔨 Cross-Platform Builds**
   - Builds binaries for Linux x86_64, macOS (Intel & ARM64), and Windows x86_64
   - Handles LLVM dependencies automatically
   - Optimizes binaries for release distribution

5. **🚀 Release Creation**
   - Creates GitHub release with comprehensive release notes
   - Uploads binary assets for all supported platforms
   - Includes installation instructions and documentation links
   - Generates proper pre-release markers for RC versions

6. **✅ Validation**
   - Validates all version updates completed successfully
   - Confirms changelog was updated correctly
   - Verifies git state and tag consistency
   - Ensures release assets are properly generated

### Supported Platforms

| Platform | Architecture | Binary Format |
|----------|--------------|---------------|
| Linux | x86_64 | `.tar.gz` |
| macOS | x86_64 (Intel) | `.tar.gz` |
| macOS | ARM64 (Apple Silicon) | `.tar.gz` |
| Windows | x86_64 | `.zip` |

### Included Binaries

Each release includes three optimized binaries:
- **`yc`** - Y Lang compiler for compiling `.why` source files
- **`yfmt`** - Y Lang formatter for code formatting and pretty-printing
- **`yls`** - Y Lang language server for IDE integration and tooling

### Manual Changelog Entries

While the system auto-generates changelog entries from commits, you can manually add entries to the "Unreleased" section before creating a release. The automation will preserve and move these entries to the appropriate version section.

**Note:** The automation is designed to be robust and includes comprehensive error handling, validation, and rollback capabilities to ensure reliable releases.