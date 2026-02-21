# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - Unreleased

### Added

- Usage sorting and spinner animation in `status --all` output

### Fixed

- Profile deduplication now uses a composite identity key (email + plan) to prevent false duplicates
- Update cache is preserved across profile lifecycle operations (save/load/delete)
- Profile lifecycle flows were hardened to reduce partial-state edge cases during save/load/delete
- Installer trap variable scoping bug that caused cleanup to fail on some shells
- Hardened installer and release scripts for edge-case reliability
- Release version bump helper now updates npm optional platform package versions and installer default version

### Changed

- Simplified profile state machine and status rendering logic
- Profile ordering simplified in `status --all` output
- "Unknown last-used" badge is now hidden instead of shown as a placeholder

### Removed

- Legacy `cx` shorthand script (use `codex-profiles` directly)

### Internal

- CI: removed unused `sccache` from workflow jobs
- CI: fixed deprecated `rust-cache` parameters and `save-if` syntax

### Documentation

- README: reorganized badges, added a tests badge, and fixed license badge color
- README: added Cargo installation instructions

## [0.1.0] - 2026-01-28

### Added

**Core Features**
- Save and load Codex CLI authentication profiles with optional labels
- Interactive profile picker with search and navigation
- List all profiles ordered by last used timestamp
- Delete profiles with confirmation prompts
- Display usage statistics (requests, costs) for profiles
- Automatic OAuth token refresh when loading expired profiles
- Support for both OAuth tokens and API keys

**CLI Experience**
- Terminal styling with color support (respects `NO_COLOR` and `FORCE_COLOR`)
- `--plain` flag to disable all styling and use raw output
- Clear error messages with actionable suggestions
- Command examples in `--help` output

**Installation**
- Smart installer script with automatic OS/architecture detection
- Checksum verification for secure downloads
- Download progress indicators with TTY detection
- Cross-platform support (Linux, macOS, Windows)
- Multiple installation methods: npm, Bun, Cargo, manual script
- Automated update checking with 24-hour interval

**Technical**
- File locking for safe concurrent operations
- Profile storage in `~/.codex/profiles/`
- Atomic file writes to prevent corruption
- 150 tests covering core functionality
- Pre-commit hooks for code quality
- Binary releases for 5 platforms (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)

[Unreleased]: https://github.com/midhunmonachan/codex-profiles/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/midhunmonachan/codex-profiles/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/midhunmonachan/codex-profiles/releases/tag/v0.1.0
