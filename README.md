<h1 align="center">Codex Profiles</h1>

<p align="center">Seamlessly switch between multiple Codex accounts</p>

<p align="center">
  <a href="https://github.com/zorbeytorunoglu/codex-profiles-ui/actions/workflows/tests.yml"><img src="https://img.shields.io/github/actions/workflow/status/zorbeytorunoglu/codex-profiles-ui/tests.yml?branch=main&label=tests" alt="Tests" /></a>
  <a href="https://github.com/zorbeytorunoglu/codex-profiles-ui/releases"><img src="https://img.shields.io/github/v/release/zorbeytorunoglu/codex-profiles-ui" alt="Release" /></a>
  <a href="https://github.com/zorbeytorunoglu/codex-profiles-ui/stargazers"><img src="https://img.shields.io/github/stars/zorbeytorunoglu/codex-profiles-ui?style=flat" alt="Stars" /></a>
  <a href="https://github.com/zorbeytorunoglu/codex-profiles-ui/blob/main/LICENSE"><img src="https://img.shields.io/github/license/zorbeytorunoglu/codex-profiles-ui?color=blue" alt="License" /></a>
</p>

<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#install">Install</a> •
  <a href="#usage">Usage</a> •
  <a href="#more-docs">More Docs</a> •
  <a href="#faq">FAQ</a>
</p>

---

## Overview

Codex Profiles lets you save and switch easily between multiple Codex accounts without repeated `codex login`

## Install

<table width="100%">
  <colgroup>
    <col style="width: 25%;" />
    <col style="width: 75%;" />
  </colgroup>
  <thead>
    <tr>
      <th align="left">Method</th>
      <th align="left" style="white-space: nowrap;">Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>npm</td>
      <td style="white-space: nowrap;"><code>npm install -g @zorbeytorunoglu/codex-profiles</code></td>
    </tr>
    <tr>
      <td>Bun</td>
      <td style="white-space: nowrap;"><code>bun install -g @zorbeytorunoglu/codex-profiles</code></td>
    </tr>
  </tbody>
</table>

### Manual install

```bash
curl -fsSL https://raw.githubusercontent.com/zorbeytorunoglu/codex-profiles-ui/main/install.sh | bash
```

<details>
<summary>Advanced install option (build from source)</summary>

```bash
cargo install --locked codex-profiles
```

Requires Rust 1.94+

</details>

## Quick Start

```bash
codex-profiles save --label work
codex-profiles list
codex-profiles dashboard
codex-profiles load --label work --force
```

## Usage

> [!NOTE]
> Codex Profiles data is stored under `~/.codex/profiles/` on your machine

### Command Reference

<table width="100%">
  <thead>
    <tr>
      <th align="left" width="44%">Command</th>
      <th align="left">Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td width="43%"><code>codex-profiles save</code><br/><code>[--label &lt;name&gt;]</code></td>
      <td>Save current <code>auth.json</code><br/>Optional label</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles load</code><br/><code>(--label &lt;name&gt; | --id &lt;profile-id&gt;)</code><br/><code>[--force]</code></td>
      <td>Load a saved profile<br/>Choose a target profile and force when needed</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles list</code><br/><code>[--show-id] [--json]</code></td>
      <td>List profiles<br/>Supports id and JSON views</td>
    </tr>
    <tr>
      <td width="44%"><code>codex-profiles export</code><br/><code>[--label &lt;name&gt;]</code><br/><code>[--id &lt;profile-id&gt; (repeatable)]</code></td>
      <td>Export to a JSON bundle<br/>Default: all profiles, or a selected subset</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles import</code><br/><code>--input &lt;file&gt;</code></td>
      <td>Import from a JSON bundle</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles doctor</code><br/><code>[--fix] [--json]</code></td>
      <td>Run diagnostics and optionally apply safe repairs</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles label set</code><br/><code>(--label &lt;name&gt; | --id &lt;profile-id&gt;)</code><br/><code>--to &lt;label&gt;</code></td>
      <td>Set or replace a label<br/>Target one profile</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles label clear</code><br/><code>(--label &lt;name&gt; | --id &lt;profile-id&gt;)</code></td>
      <td>Clear a label<br/>Target one profile</td>
    </tr>
    <tr>
      <td width="43%"><code>codex-profiles label rename</code><br/><code>--label &lt;label&gt; --to &lt;label&gt;</code></td>
      <td>Rename an existing label</td>
    </tr>
    <tr>
      <td width="44%"><code>codex-profiles status</code><br/><code>[--label &lt;name&gt; | --id &lt;profile-id&gt;]</code><br/><code>[--all] [--json]</code></td>
      <td>Show usage for active, selected, or all targets<br/>Human-readable or JSON output</td>
    </tr>
    <tr>
      <td width="44%"><code>codex-profiles dashboard</code><br/><code>[--interval-secs &lt;seconds&gt;]</code></td>
      <td>Open a live TUI dashboard<br/>Refreshes profile status and lets you load the selected profile</td>
    </tr>
    <tr>
      <td width="44%"><code>codex-profiles delete</code><br/><code>[--label &lt;name&gt; | --id &lt;profile-id&gt; (repeatable)]</code><br/><code>[--yes]</code></td>
      <td>Delete by label or id<br/>Supports bulk delete and non-interactive mode</td>
    </tr>
  </tbody>
</table>

### Notes

- `load` and `delete` are interactive unless you pass `--label` or `--id`
- `dashboard` uses an interactive terminal UI with automatic refresh and manual load controls
- Export bundles contain secrets

## More Docs

- [Release verification guide](https://github.com/zorbeytorunoglu/codex-profiles-ui/blob/main/docs/verification.md)
- [Contribution guide](https://github.com/zorbeytorunoglu/codex-profiles-ui/blob/main/CONTRIBUTING.md)

## FAQ

<details>
<summary>How do I uninstall?</summary>

> - npm: `npm uninstall -g @zorbeytorunoglu/codex-profiles`
> - Bun: `bun uninstall -g @zorbeytorunoglu/codex-profiles`
> - Cargo: `cargo uninstall codex-profiles`
> - Manual: `rm ~/.local/bin/codex-profiles`
</details>

<details>
<summary>Is my auth file uploaded anywhere?</summary>

> No. Everything stays on your machine. This tool only copies files locally
</details>

<details>
<summary>What is a “profile” in this tool?</summary>

> A profile is a saved copy of your `~/.codex/auth.json`. Each profile represents
> one Codex account
</details>

<details>
<summary>What happens if I run load without saving?</summary>

> You will be prompted to save the active profile, continue without saving, or
> cancel
</details>

<details>
<summary>Can I keep personal and work accounts separate?</summary>

> Yes. Save each account with a label (for example, `personal` and `work`) and
> switch with the label
</details>

<details>
<summary>How can I verify my installation?</summary>

> ```bash
> codex-profiles --version
> codex-profiles list
> ```
</details>

<details>
<summary>Where are release verification steps documented?</summary>

> In [docs/verification.md](https://github.com/zorbeytorunoglu/codex-profiles-ui/blob/main/docs/verification.md)
</details>
