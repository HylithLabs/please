<h1 align="center">
    <a href="https://please.hylith.com"><img src="./src/assets/logo.png" width="175px" alt="please"></a>
</h1>

<h3 align="center">An AI native git CLI, built so you never have to type a raw git command again.</h3>

---

## Table of Contents

* [Overview](#overview)
* [Policy, rules and guidelines](#policy-rules-and-guidelines)
* [Project layout](#project-layout)
  * [Command structure](#command-structure)
  * [The AI provider layer](#the-ai-provider-layer)
* [Adding a new AI provider](#adding-a-new-ai-provider)
* [Creating a new command for please](#creating-a-new-command-for-please)
  * [Fork the please repository](#fork-the-please-repository)
  * [Make changes to your fork](#make-changes-to-your-fork)
  * [Follow the existing command conventions](#follow-the-existing-command-conventions)
  * [Destructive commands](#destructive-commands)
  * [Making a command AI agent aware](#making-a-command-ai-agent-aware)
  * [Create a pull request](#create-a-pull-request)
  * [Fixing any CI errors](#fixing-any-ci-errors)
  * [Reviewing and merging](#reviewing-and-merging)
* [Reporting bugs and requesting features](#reporting-bugs-and-requesting-features)
* [Reporting security vulnerabilities](#reporting-security-vulnerabilities)

## Overview

[`HylithLabs/please`](https://github.com/HylithLabs/please) is the source repository for please, an AI native git CLI written in Rust. Please wraps everyday git and gh workflows in plain language commands, and hands anything outside that set to an AI agent, Anthropic Claude, Google Gemini, or OpenAI ChatGPT, whichever the user has configured, which then acts on the user's behalf through git, gh, and please itself.

Please relies on community contributions to grow the set of commands it supports and to keep the AI agent's behavior safe and predictable. With every command sharing the same confirmation patterns and the same underlying git layer, it is important that new contributions follow the conventions already established in the codebase.

This document covers how the project is laid out, how to get your development environment running, and how to get a change merged.

## Policy, rules and guidelines

Please welcomes contributions of any size, from a documentation fix to a new command.

Every new command must be reachable without the contributor writing raw git anywhere outside `src/git.rs`. This keeps the project's core promise intact: a user of please should never need to fall back to git themselves.

Any command capable of discarding work must ask for explicit confirmation before running, following the pattern already used throughout the codebase.

New AI provider integrations are welcome, provided they implement the full set of functions the existing providers already implement, so the rest of the codebase does not need to special case a particular provider.

## Project layout

You need a recent stable Rust toolchain, targeting the 2024 edition, and git.

```bash
git clone https://github.com/HylithLabs/please.git
cd please
cargo build
```

Run it locally with `cargo run -- <command>`, for example `cargo run -- help`. If you want to exercise the AI backed commands locally, run `cargo run -- setup` first and configure a provider key the same way an end user would. This is saved to `~/.please/config` and is not part of the repository.

### Command structure

* `src/main.rs` is the thin entry point, and hands off to `dispatch.rs`.
* `src/dispatch.rs` routes please's argv to a subcommand, including fuzzy matching typo'd command names.
* `src/commands/*.rs` holds one file per please subcommand (`commit.rs`, `push.rs`, `squash.rs`, and so on). This is almost always where a new command or a change to an existing one belongs.
* `src/git.rs` holds every raw git subprocess call the rest of the codebase needs. If a command needs a new git operation, add it here rather than shelling out to git directly from `src/commands/`.
* `src/config.rs` reads and writes `~/.please/config`.
* `src/context.rs` builds and caches the project description at `.git/PLEASE.MD` that gives the AI context about the codebase.
* `src/sensitive.rs` detects sensitive files, such as `.env` and credentials, that guardrails skip during staging.
* `src/gitignore.rs` handles auto adding untracked build and dependency directories to `.gitignore`.
* `src/update_checker.rs` checks for and applies new releases.
* `src/ui.rs` holds terminal styling helpers used by commands for user facing output.

### The AI provider layer

`src/llm/` is the AI provider layer. `mod.rs` holds provider neutral dispatch logic, while `anthropic.rs`, `gemini.rs`, and `openai.rs` are thin adapters that each implement the same functions for their own provider.

## Adding a new AI provider

The tool calling conversation is represented in provider neutral terms inside `src/llm/mod.rs`. Each provider only needs a thin adapter, matching the shape of `anthropic.rs`, `gemini.rs`, or `openai.rs`, that translates that neutral representation to and from its own wire format: model listing, key validation, message formatting, and tool call parsing. Match an existing adapter's function signatures rather than introducing new ones, so `mod.rs` does not need special casing per provider.

## Creating a new command for please

Changes to please happen through the following process.

### Fork the please repository

Fork [`HylithLabs/please`](https://github.com/HylithLabs/please) to your own GitHub account:

[https://github.com/HylithLabs/please/fork](https://github.com/HylithLabs/please/fork)

Select your GitHub account as the destination and wait for the forking process to complete. Once done, you will be able to access your fork from your own GitHub account at `https://github.com/your-github-username/please`.

### Make changes to your fork

Create a new branch off `main` with a sensible name, for example `add stash drop confirmation`, and make your changes there. Please avoid committing directly to `main` in your fork, since this makes it harder to keep your fork in sync with upstream while a pull request is open.

### Follow the existing command conventions

At minimum, a new command needs the following:

1. `src/commands/<name>.rs` with a `pub fn run(args: &[String])`.
2. A `pub mod <name>;` line in `src/commands/mod.rs`.
3. An entry in the `COMMANDS` table in `src/dispatch.rs`.
4. A line in `src/commands/help.rs`'s output.

Following the shape of an existing command, such as `src/commands/squash.rs`, is the fastest way to get this right.

### Destructive commands

Any command that can lose work, whether that is discarding uncommitted changes, rewriting history, deleting a branch, or force pushing, must follow the existing confirmation pattern. Explain the consequences in plain language, show exactly what will be lost or changed, then require the user to type `yes` before proceeding. See `src/commands/discard.rs` or `src/commands/purge.rs` for reference implementations. Please discuss any flag that would skip this prompt with a maintainer before adding one.

### Making a command AI agent aware

If a command should be something the AI agent can run on the user's behalf, add it to the subcommand list in `run_please`'s tool description in `src/commands/agent.rs`. Two safety nets apply automatically to agent driven commands and should not be bypassed. Anything destructive run through raw git or gh by the agent stops and asks the user to confirm before running. Any please subcommand that would normally prompt for confirmation itself cancels the same way for the agent as it would for any other non interactive caller, and the agent relays that back to the user rather than pretending it succeeded.

### Create a pull request

Run the checks CI will also run, so nothing surprises you:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

`cargo fmt --all`, without `--check`, will fix formatting for you. Fix any clippy warnings rather than suppressing them with `#[allow(...)]`, unless there is a genuine reason the lint does not apply, in which case leave a short comment explaining why.

This project loosely follows [Conventional Commits](https://www.conventionalcommits.org/) style (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, and so on) for commit messages. This is not enforced, but appreciated.

Push your branch and open a pull request against `HylithLabs/please`'s `main` branch. Fill out the description with what changed and why. If it resolves an open issue, add `Resolves #<issue>` so GitHub links the two together. If you are planning a larger change, such as a new command category or a new AI provider, please open an issue to discuss the approach first.

### Fixing any CI errors

Every pull request runs the same `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo build` checks described above through GitHub Actions. If CI fails, open the failing job's log from the pull request's Checks tab to see which step and which file it is complaining about, fix it locally, and push again. There is no need to open a new pull request.

If a check fails in a way that seems unrelated to your change, such as a flaky test or an environment issue, mention it in the pull request rather than trying to work around it, and a maintainer will help sort it out.

### Reviewing and merging

Once CI is passing, a maintainer will review your pull request. Expect feedback on whether the change fits the project's design philosophy described in the [README](README.md#design-philosophy), whether a new destructive command follows the existing confirmation pattern, and whether the change has adequate test coverage.

Once a maintainer is happy with your pull request, they will merge it. You can then delete your fork, or your branch if you plan to contribute again.

Thank you for contributing to please.

## Reporting bugs and requesting features

Open a [GitHub issue](https://github.com/HylithLabs/please/issues/new). Include your operating system, how you installed please (installer script, Homebrew, or `cargo install`), the exact command you ran, and what you expected versus what happened.

## Reporting security vulnerabilities

Please do not open a public issue for a security vulnerability. See [SECURITY.md](SECURITY.md) for how to report one responsibly.
