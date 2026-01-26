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
> Requires [Codex CLI](https://developers.openai.com/codex/cli/) (with ChatGPT subscription or OpenAI API key).

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

### Homebrew

```bash
brew install --cask codex-profiles
```

### Manual Install

Automatically detects your OS/architecture, downloads the correct binary, verifies checksums:

```bash
curl -fsSL https://raw.githubusercontent.com/midhunmonachan/codex-profiles/main/install.sh | bash
```

## Uninstall

> [!WARNING]
> Legacy script support is ending. Remove `cx` and use this version instead.
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

### NPM

```bash
npm uninstall -g codex-profiles
```

### Bun

```bash
bun uninstall -g codex-profiles
```

### Homebrew

```bash
brew uninstall --cask codex-profiles
```

### Manual Uninstall

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
> | `profiles.json` | Profile metadata (labels, last-used, active). |
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
> codex-profiles --help
>
> # Verify Codex CLI is detected
> codex-profiles list
> # Should show: "No profiles saved yet" (not an error about missing Codex CLI)
> ```
>
> If you see "Codex CLI not found", install it from [here](https://developers.openai.com/codex/cli/).
</details>
