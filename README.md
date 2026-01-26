<h1 align="center">Codex Profiles</h1>

<p align="center">Manage multiple Codex CLI profiles and switch between them instantly.</p>

<p align="center">
  <img src="https://img.shields.io/github/stars/midhunmonachan/codex-profiles" alt="Stars" />
  <img src="https://img.shields.io/github/v/release/midhunmonachan/codex-profiles" alt="Release" />
  <img src="https://img.shields.io/github/issues/midhunmonachan/codex-profiles" alt="Issues" />
  <img src="https://img.shields.io/github/license/midhunmonachan/codex-profiles" alt="License" />
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

> [!IMPORTANT]
> Requires [Codex CLI](https://developers.openai.com/codex/cli/).

> [!WARNING]
> Codex CLI is not included with the ChatGPT Free plan.

> [!TIP]
> Looking for a Teams promo? [See details](https://www.reddit.com/r/ChatGPTPromptGenius/comments/1lo7v0u/chatgpt_team_for_1_first_month_up_to_5_users/)

### npm (recommended)

```bash
npm install -g codex-profiles
```

### bun

```bash
bun install -g codex-profiles
```

The npm/bun installers pull a small JS launcher plus a platform-specific binary
package (for example, `codex-profiles-linux-x64`).

### Homebrew (macOS)

```bash
brew install --cask codex-profiles
```

### Shell script (recommended for manual install)

Automatically detects your OS/architecture, downloads the correct binary, verifies checksums:

```bash
curl -fsSL https://raw.githubusercontent.com/midhunmonachan/codex-profiles/main/install.sh | bash
```

Or download and inspect first:

```bash
curl -fsSL https://raw.githubusercontent.com/midhunmonachan/codex-profiles/main/install.sh -o install.sh
chmod +x install.sh
./install.sh --help
```

### GitHub releases (manual)

1. Download the appropriate asset for your OS/arch from the [latest release](https://github.com/midhunmonachan/codex-profiles/releases/latest).
2. Extract and move the binary into your PATH.

Example (Linux x64):

```bash
VERSION=0.1.0
TARGET=x86_64-unknown-linux-gnu
curl -L -o codex-profiles.tar.gz \
  "https://github.com/midhunmonachan/codex-profiles/releases/download/v${VERSION}/codex-profiles-${TARGET}.tar.gz"
tar -xzf codex-profiles.tar.gz
install -m 755 codex-profiles ~/.local/bin/codex-profiles
```

### From source

```bash
cargo install --path .
```

Or from git:

```bash
cargo install --git https://github.com/midhunmonachan/codex-profiles --locked
```

## Uninstall

> [!IMPORTANT]
> If you installed the legacy `cx` script, remove it and use this version instead, as it will no longer be supported:
>
> ```bash
> rm ~/.local/bin/cx
> ```
>
> If you installed with a custom command name (`mycmd`), remove that name instead:
>
> ```bash
> rm ~/.local/bin/mycmd
> ```

### npm

```bash
npm uninstall -g codex-profiles
```

### bun

```bash
bun uninstall -g codex-profiles
```

### Homebrew (macOS)

```bash
brew uninstall --cask codex-profiles
```

### Manual install

```bash
rm ~/.local/bin/codex-profiles
```

## Usage

> [!TIP]
> Commands are interactive unless you pass `--label`.

| Command | Description |
| --- | --- |
| `codex-profiles save [--label <name>]` | Save the current `auth.json` as a profile, optionally labeled. |
| `codex-profiles load [--label <name>]` | Load a profile from the picker without re-login (or by label). |
| `codex-profiles list` | List profiles ordered by last used. |
| `codex-profiles status [--all] [--label <name>]` | Show usage for the current profile, all profiles, or a specific label. |
| `codex-profiles delete [--yes] [--label <name>]` | Delete profiles from the picker (or by label). |

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
> | `usage.tsv` | Last-used timestamps. |
> | `labels.json` | Optional labels. |
> | `version.json` | Update check cache. |

## FAQ

### Is my auth file uploaded anywhere?

No. Everything stays on your machine. This tool only copies files locally.

### What is a “profile” in this tool?

A profile is a saved copy of your `~/.codex/auth.json`. Each profile represents
one Codex login.

### How do I save and switch between accounts?

Log in with Codex CLI, then run `codex-profiles save --label <name>`. To switch
later, run `codex-profiles load --label <name>`.

### What happens if I run load without saving?

You will be prompted to save the current profile, continue without saving, or
cancel.

### Can I keep personal and work accounts separate?

Yes. Save each account with a label (for example, `personal` and `work`) and
switch with the label.

### How can I verify my installation?

After installing, verify it works:

```bash
# Check version
codex-profiles --help

# Verify Codex CLI is detected
codex-profiles list
# Should show: "No profiles saved yet" (not an error about missing Codex CLI)
```

If you see "Codex CLI not found", install it from https://developers.openai.com/codex/cli/.

