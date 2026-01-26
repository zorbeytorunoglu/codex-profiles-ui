# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-26

### Added
- Initial release
- Save current Codex CLI auth profile with optional labels
- Load saved profiles with interactive picker or by label
- List all saved profiles ordered by last used
- Delete profiles with confirmation or by label
- Display usage details for current profile, all profiles, or by label
- OAuth token refresh on profile load
- Automatic update checking (24-hour interval)
- Multi-install support (npm, bun, cargo, homebrew, manual)
- File locking for concurrent-safe usage tracking
- Atomic file operations for profile and auth data
- Support for both OAuth tokens and API keys
- Terminal styling with `NO_COLOR` and `FORCE_COLOR` support
- `--plain` flag to disable all styling
- Profile storage in `~/.codex/profiles/`
- Profile metadata (labels, last-used, update cache) stored in `profiles.json`
- Pre-commit and pre-push hooks for code quality
- 100% test coverage with 150+ tests
- Multi-platform CI (Linux, macOS, Windows)
- Cross-compilation for 5 target platforms

[Unreleased]: https://github.com/midhunmonachan/codex-profiles/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/midhunmonachan/codex-profiles/releases/tag/v0.1.0
