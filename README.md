<h1 align="center">Codex CLI Profiles</h1>

<p align="center">Manage multiple Codex auth profiles by saving and restoring <code>~/.codex/auth.json</code>.</p>

<p align="center">
  <img src="https://img.shields.io/github/last-commit/midhunmonachan/codex-cli-profiles" alt="Last commit" />
  <img src="https://img.shields.io/github/issues/midhunmonachan/codex-cli-profiles" alt="Issues" />
</p>

---

## Requirements

- `bash`, `date`, `jq`, `node`, and `codex` (Codex CLI)
- Tested on Ubuntu 24.04 with Codex CLI 0.79.0
- Expected to work on other Linux distributions, but unverified

## Install

```sh
./install.sh
```

Optional custom command name:

```sh
./install.sh --name mycmd
```

## Quickstart

1. Save the current account as a profile.
2. Switch back to it later, or list everything saved so far.

```sh
cx save
cx list
cx load <account_id>
```

## Usage

| Command | Description |
| --- | --- |
| `cx save` | Save the current `auth.json` as a profile. |
| `cx load [id]` | Load a profile; no ID picks the least recently used. |
| `cx list` | List profiles ordered by last used. |
| `cx current` | Print the active `account_id`. |

## How it works

- Profiles are stored as `~/.codex/profiles/{account_id}.json`.
- Last-used timestamps are tracked in `~/.codex/profiles/usage.tsv`.
- `load` with no ID auto-selects the least recently used profile.

## Uninstall

Remove the symlink from `~/.local/bin`:

```sh
rm ~/.local/bin/cx
```

If you installed with a custom name, remove that name instead:

```sh
rm ~/.local/bin/mycmd
```

## Examples

<details>
<summary>Rotate between two accounts</summary>

```sh
cx save
cx load 1234567890abcdef
cx save
cx load abcdef1234567890
```
</details>

<details>
<summary>Restore the least recently used account</summary>

```sh
cx load
```
</details>

## Troubleshooting

- Missing `jq`: install it with your package manager.
- `date -d` not found: install GNU coreutils or use a Linux environment.
- "Error: ~/.local/bin/cx exists and is not cx symlink": remove or rename the existing file.
