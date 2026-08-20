<div align="center">
    <img src="./src/assets/logo.png" width="175px" alt="please">
<h1>Please</h1>
</div>

<h3 align="center">An AI-native git CLI — you never type raw <code>git</code> commands again.</h3>

<p align="center">
  <a href="https://please.hylith.com">Documentation & Installation</a>
</p>

---

<p align="center">
 <a href="https://github.com/HylithLabs/please/actions/workflows/ci.yml">
  <img src="https://img.shields.io/github/actions/workflow/status/HylithLabs/please/ci.yml?branch=main&style=flat-square" alt="CI">
 </a>
 <a href="https://github.com/HylithLabs/please/blob/main/LICENSE">
  <img src="https://img.shields.io/badge/License-Apache%202.0-brightgreen.svg?style=flat-square" alt="Apache 2.0 License">
 </a>
 <a href="#contributing">
  <img src="https://img.shields.io/badge/PRs-Welcome-brightgreen.svg?style=flat-square" alt="PRs Welcome">
 </a>
</p>

---

<p align="center">
  <img src="./src/assets/demo.gif" alt="Please Demo">
</p>

## Table of Contents

* [Introduction](#introduction)
* [Install](#install)
* [Setup](#setup)
* [Design Philosophy](#design-philosophy)
* [Contributing](#contributing)
* [Roadmap](#roadmap)
* [Security](#security)
* [License](#license)

## Introduction

`please` is an AI-native interface for git. Instead of memorizing and typing raw `git` (and `gh`) commands, you run `please` commands, and an AI agent — your choice of Anthropic Claude, Google Gemini, or OpenAI ChatGPT — handles staging, commit messages, branch management, and pushing on your behalf.

Git remains the engine underneath; `please` is the interface. Every operation still runs as real git under the hood — nothing is reinvented, and your history stays fully compatible with any other git tool or collaborator.

Run `please` with no arguments (or `please help`) at any time for a full, grouped command reference. Typos and filler words don't block a command either — `please swich to master` still runs `please switch master`. A typo'd command word is recognized if it's one edit away from a real command and isn't also close to some *other* command; small connector words like "to"/"into"/"the"/"a"/"an" are dropped before the real argument is passed through. Anything looser than that is deliberately left alone and handed to the AI agent instead, so a request like "clean up my messy code" asks the AI to look at your code rather than silently running `please cleanup` and deleting branches.

Beyond the built-in commands, `please "<what you want to do>"` and `please chat` let you describe what you want in plain language — the AI figures out how to do it using `git`, `gh`, and `please` itself, with destructive actions always stopping to ask for confirmation first.

## Install

Full documentation and installation instructions also live at [please.hylith.com](https://please.hylith.com).

macOS / Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/HylithLabs/please/releases/latest/download/please-installer.sh | sh
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/HylithLabs/please/releases/latest/download/please-installer.ps1 | iex"
```

Homebrew:

```bash
brew install HylithLabs/please/please
```

Or build it yourself with `cargo install --path .` if you'd rather not run a prebuilt binary — note that `please update` only works for the prebuilt installs above, since it has no source tree to rebuild from.

## Setup

Run once per machine:

```bash
please setup
```

You'll pick a provider — **Anthropic** (Claude), **Google** (Gemini), or **OpenAI** (ChatGPT) — and paste an API key. `please` checks the key against the provider right away with a real, cheap call, so a typo gets caught immediately instead of failing confusingly on your first `please commit`. It then auto-picks the cheapest model the key has access to, though you can decline the pick and type a specific model id yourself. This is saved globally to `~/.please/config` and applies to every project on the machine.

You can save keys for more than one provider, switch which is active at any time, and change a saved provider's model later without re-entering its key. Re-run `please setup` and it shows what's already configured, then lets you add, switch, update, or remove providers.

On first use in a repo, `please` also generates a short project description cached at `.git/PLEASE.MD`, giving the AI context about the codebase.

## Design Philosophy

- **No hand-written git.** Every git operation a developer needs is reachable through a `please` command.
- **Explicit overrides, not blanket blocks.** Guardrails (sensitive files, junk directories) can always be bypassed by staging the file yourself first — the tool assumes intent over blocking outright.
- **Bounded AI latency.** The model's own response time isn't something we control, but everything around it is: requests are timeout-bounded, progress is printed so the CLI never looks frozen, and the model is auto-selected once at setup rather than re-selected on every call.
- **Destructive operations require confirmation.** Anything that can discard work explains the consequences up front and requires explicit confirmation.

## Contributing

To contribute to `please`, please refer to our [CONTRIBUTING.md](CONTRIBUTING.md) document. It covers how the project is laid out, how to add a new command, and what's expected before opening a pull request. If you need further information, please [open an issue](https://github.com/HylithLabs/please/issues/new).

Please also see our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

## Roadmap

See `PROJECT.MD` for the full vision, including the planned **Auto Commit** feature (AI commits automatically as you reach meaningful checkpoints, no command needed) and a configuration GUI.

## Security

If you discover a security vulnerability, please do not open a public issue — see [SECURITY.md](SECURITY.md) for how to report it responsibly.

## License

`please` is published under the [Apache 2.0 license](LICENSE).
