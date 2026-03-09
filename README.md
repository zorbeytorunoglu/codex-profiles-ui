<h1 align="center">Codex Profiles</h1>

<p align="center">Manage multiple Codex CLI profiles and switch between them instantly.</p>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/midhunmonachan/codex-profiles/tests.yml?branch=main&label=tests" alt="Tests" />
  <img src="https://img.shields.io/github/v/release/midhunmonachan/codex-profiles" alt="Release" />
  <img src="https://img.shields.io/github/stars/midhunmonachan/codex-profiles?style=flat" alt="Stars" />
  <img src="https://img.shields.io/github/license/midhunmonachan/codex-profiles?color=blue" alt="License" />
</p>

<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#install">Install</a> •
  <a href="#uninstall">Uninstall</a> •
  <a href="#usage">Usage</a> •
  <a href="#faq">FAQ</a>
</p>

---

## Overview

Codex Profiles helps you manage multiple Codex CLI logins on a single machine.
It saves the current login and lets you switch in seconds, making it ideal for
personal and team accounts across multiple organizations.

## Install

> [!NOTE]
> `npm`, `bun`, and the manual installer use prebuilt native binaries. `cargo install --locked codex-profiles` builds locally from source if you prefer not to run prebuilt binaries.

> [!TIP]
> Looking for a Teams promo? [See details](https://www.reddit.com/r/ChatGPTPromptGenius/comments/1lo7v0u/chatgpt_team_for_1_first_month_up_to_5_users/)

### NPM

```bash
npm install -g codex-profiles
```

### Bun

```bash
bun install -g codex-profiles
```

### Cargo

```bash
cargo install --locked codex-profiles
```

### Manual Install

Automatically detects your OS/architecture, downloads the correct binary, and verifies checksums when verification tooling is available:

```bash
curl -fsSL https://raw.githubusercontent.com/midhunmonachan/codex-profiles/main/install.sh | bash
```

## Uninstall

### NPM

```bash
npm uninstall -g codex-profiles
```

### Bun

```bash
bun uninstall -g codex-profiles
```

### Cargo

```bash
cargo uninstall codex-profiles
```

### Manual Uninstall

```bash
rm ~/.local/bin/codex-profiles
```

## Usage

> [!TIP]
> `load` and `delete` are interactive unless you pass `--label` or `--id`.
> If the current profile is not saved, `load` also prompts before switching unless you pass `--force`.

| Command | Description |
| --- | --- |
| `codex-profiles save [options]` | Save the current `auth.json` as a profile. Use `--label <name>` to label it. |
| `codex-profiles load [options]` | Load a profile without re-login. Use `--label <name>`, `--id <profile-id>`, and `--force` as needed. |
| `codex-profiles list [options]` | List saved profiles. Use `--show-id` or `--json` for alternate output. |
| `codex-profiles export [options]` | Export saved profiles to a single JSON bundle. |
| `codex-profiles import [options]` | Import saved profiles from a JSON bundle. |
| `codex-profiles doctor [--fix] [--json]` | Run local diagnostics and optionally apply safe metadata repairs. |
| `codex-profiles label set [options]` | Add or replace a label on a saved profile. |
| `codex-profiles label clear [options]` | Remove a label from a saved profile. |
| `codex-profiles label rename [options]` | Rename an existing label without using set/clear manually. |
| `codex-profiles default [set\|clear\|show]` | Manage the default saved profile used by non-interactive load. |
| `codex-profiles status [options]` | Show usage for the current profile, a selected saved profile (`--label`/`--id`), or all saved profiles with `--all`. Use `--json` for structured output. |
| `codex-profiles delete [options]` | Delete profiles. Use `--label <name>`, repeat `--id <profile-id>`, and `--yes` as needed. |

Label examples: `codex-profiles label set --id <profile-id> --to work`, `codex-profiles label clear --label work`.

Default examples: `codex-profiles default set --label work`, `codex-profiles default show`, `codex-profiles default clear`.

When `load` runs without a selector in a non-interactive session, it uses the saved default profile if one is set.

`status --json` returns the current profile object (or `null`); `status --label/--id --json` returns the selected saved profile object (or `null` when no saved profiles exist); `status --all --json` returns `profiles` plus hidden-profile counts.

`export --output <file>` exports all saved profiles by default. Use `--label` or repeated `--id` to export a smaller set.

Export bundles contain secrets. Store them securely. `import` fails on id, label, or default-profile conflicts instead of overwriting existing state.

When the exported set includes the current default profile, export/import preserves that default selection too.

`doctor --fix` repairs safe profile-storage metadata only (missing storage files, stale index/default refs, and a rebuild of `profiles.json` from saved profile files when needed). It does not delete invalid saved profile files in this first pass; when it rebuilds a broken `profiles.json`, it writes `profiles.json.bak` (or `profiles.json.bak.N` if a backup already exists), and labels/defaults stored only in the broken index may need to be reconfigured.

> [!WARNING]
> Deleting a profile does not log you out. It only removes the saved profile file.

Quick example:

```console
$ codex-profiles save --label team
Saved profile mail@company.com (Team)

$ codex-profiles load --label team
Loaded profile mail@company.com (Team)
```

> [!NOTE]
> Files are stored under `~/.codex/profiles/`:
>
> | File | Purpose |
> | --- | --- |
> | `{email-plan}.json` | Saved profiles. |
> | `profiles.json` | Profile metadata (labels and identity fields for saved profiles). |
> | `update.json` | Cached updater state (latest checked version metadata). |
> | `profiles.lock` | Lock file for safe updates. |

## FAQ

<details>
<summary>Is my auth file uploaded anywhere?</summary>

> No. Everything stays on your machine. This tool only copies files locally.
</details>

<details>
<summary>What is a “profile” in this tool?</summary>

> A profile is a saved copy of your `~/.codex/auth.json`. Each profile represents
> one Codex login.
</details>

<details>
<summary>How do I save and switch between accounts?</summary>

> Log in with Codex CLI, then run `codex-profiles save --label <name>`. To switch
> later, run `codex-profiles load --label <name>`.
</details>

<details>
<summary>What happens if I run load without saving?</summary>

> You will be prompted to save the current profile, continue without saving, or
> cancel.
</details>

<details>
<summary>Can I keep personal and work accounts separate?</summary>

> Yes. Save each account with a label (for example, `personal` and `work`) and
> switch with the label.
</details>

<details>
<summary>How can I verify my installation?</summary>

> After installing, verify it works:
>
> ```bash
> # Check version
> codex-profiles --version
>
> # Verify the command runs
> codex-profiles list
> # Should show: "No saved profiles." if you have not saved any profiles yet
> ```
</details>

<details>
<summary>How do I verify a release?</summary>

> See [docs/verification.md](docs/verification.md) for release manifests, checksums,
> and GitHub attestation verification steps.
</details>

<details>
<summary>Can I contribute to this project?</summary>

> Yes! Contributions are welcome. For non-trivial changes (new features, significant
> refactors), please open an [issue](https://github.com/midhunmonachan/codex-profiles/issues)
> or [discussion](https://github.com/midhunmonachan/codex-profiles/discussions) first
> to discuss your idea and avoid wasted effort.
>
> For minor changes (bug fixes, typos, docs), feel free to submit a PR directly.
>
> See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.
</details>
