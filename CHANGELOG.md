# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-27

### Added
- Save/load Codex CLI auth profiles with optional labels
- Interactive picker for profile selection
- List profiles ordered by last used
- Delete profiles with confirmation
- Display usage details for current, all, or specific profiles
- OAuth token refresh on profile load
- Automatic update checking (24-hour interval)
- Smart installer with checksum verification and auto-platform detection
- Cross-platform install.sh support (Linux, macOS, Windows)
- Download progress indicators in installer
- Multi-install support (npm, bun, cargo, homebrew, manual)
- File locking for concurrent-safe operations
- Support for both OAuth tokens and API keys
- Terminal styling with `NO_COLOR` and `FORCE_COLOR` support
- `--plain` flag to disable styling
- Profile storage in `~/.codex/profiles/`
- Contribution guidelines with PR process and code standards
- Pre-commit and pre-push hooks
- Comprehensive test suite with 150 tests
- Cross-compilation for 5 platforms (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)

[Unreleased]: https://github.com/midhunmonachan/codex-profiles/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/midhunmonachan/codex-profiles/releases/tag/v0.1.0
