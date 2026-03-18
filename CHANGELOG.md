# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `--json` output for mutating commands: `save`, `load`, `delete`, `label set`, `label clear`, `label rename`, `export`, and `import`
- `doctor --json` and `doctor --fix` for scriptable diagnostics and repair
- profile label management commands: `label set`, `label clear`, and `label rename`
- `status` selectors and JSON modes: `--id`, `--label`, and `--json`
- `load --force`, `export`, `import`, exact `--id` selectors for `load` / `delete`, and `list --json` / `--show-id`

### Changed

- mutating commands now emit a uniform JSON response shape when `--json` is passed
- `status` now refreshes tokens on usage `401` responses, syncs refreshed credentials back to the saved profile when the current profile is saved, and formats auth/usage HTTP errors in Codex-CLI style
- regular remote error output now uses aligned multiline blocks while JSON output preserves raw backend detail
- top-level `--help` output now uses a shorter example list and highlights `--json` support as a common option
- errors always exit non-zero and write to stderr only; stdout is never polluted with partial JSON on error
- status output marker and related doctor messaging now use "active profile" wording
- `status --all --json` now returns a single `profiles` array that includes API-key and errored profiles

### Removed

- `anyhow` dependency; `updates.rs` now uses `Result<T, String>` consistently
- `status --all --show-errors` flag and hidden-profile summary/count modes

### Internal

- auth refresh now refuses to rewrite drifted on-disk auth state
- remote output sanitization and legacy-schema detection were hardened
- `updates.rs` error handling is now consistently `Result<T, String>` / `.map_err(|e| e.to_string())`
- integration and regression coverage expanded for mutating JSON output, remote error formatting, and status/profile edge cases

## [0.2.0] - 2026-02-21

### Added

- `status --all --show-errors` to include errored profiles in the output
- Hidden-profile summaries in `status --all` (API profiles and errored profiles)

### Fixed

- Profile deduplication now uses a composite identity key (`principal_id` + `workspace_or_org_id` + `plan_type`) to prevent false matches
- Profile lifecycle operations now preserve update cache and handle save/load/delete edge cases more safely
- Installer trap variable scoping bug that could break cleanup on some shells
- Release bump flow now syncs npm optional package versions and installer default `VERSION` correctly
- CI cache config updated to remove deprecated `rust-cache` parameters and invalid `save-if` usage

### Changed

- Simplified profile state machine and status rendering logic
- Profile ordering changed from last-used timestamp sorting to deterministic profile-id ordering
- `list` and `status` no longer perform implicit current-profile sync side effects
- "Unknown last-used" badge is now hidden instead of shown as a placeholder

### Removed

- Last-used timestamp tracking in `profiles.json` profile metadata
- Legacy `cx` shorthand script (use `codex-profiles` directly)

### Internal

- Installer and release scripts hardened for edge-case reliability
- CI: removed unused `sccache` from workflow jobs
- Added repository checksum file for previous release (`checksums/v0.1.0.txt`)

### Documentation

- README: reorganized badges, added a tests badge, and fixed license badge color
- README: added Cargo installation instructions
- CONTRIBUTING: documented release-tag bump behavior for npm optional packages and installer defaults

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
