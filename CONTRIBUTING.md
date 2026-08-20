<div align="center">
    <img src="./src/assets/logo.png" width="120px" alt="please">
<h1>Contributing to Please</h1>
</div>

<h3 align="center">An AI-native git CLI — you never type raw <code>git</code> commands again.</h3>

---

## Table of Contents

* [Overview](#overview)
* [Code of Conduct](#code-of-conduct)
* [Getting set up](#getting-set-up)
* [Project layout](#project-layout)
* [Adding a new command](#adding-a-new-command)
  * [Destructive commands](#destructive-commands)
  * [Making a command AI-agent aware](#making-a-command-ai-agent-aware)
* [Adding a new AI provider](#adding-a-new-ai-provider)
* [Before opening a pull request](#before-opening-a-pull-request)
* [Commit messages](#commit-messages)
* [Creating a pull request](#creating-a-pull-request)
* [Fixing any CI errors](#fixing-any-ci-errors)
* [Reviewing and merging](#reviewing-and-merging)
* [Reporting bugs and requesting features](#reporting-bugs-and-requesting-features)
* [Reporting security vulnerabilities](#reporting-security-vulnerabilities)
* [License](#license)

## Overview

[`HylithLabs/please`](https://github.com/HylithLabs/please) is the source repository for `please`, an AI-native git CLI written in Rust. `please` wraps everyday git (and `gh`) workflows in plain-language commands, and hands anything outside that set off to an AI agent — Anthropic Claude, Google Gemini, or OpenAI ChatGPT, whichever the user has configured — that acts on the user's behalf through `git`, `gh`, and `please` itself.

This document covers how the project is laid out, how to get your development environment set up, and how to get a change merged. Please read it before opening a pull request — it'll save you and the maintainers a round trip.

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you're expected to uphold it. Please report unacceptable behavior as described there.

## Getting set up

You need a recent stable Rust toolchain (the project targets the 2024 edition) and `git`.

```bash
git clone https://github.com/HylithLabs/please.git
cd please
cargo build
```

Run it locally with `cargo run -- <command>`, e.g.:

```bash
cargo run -- help
cargo run -- commit
```

If you want to exercise the AI-backed commands (`please commit`, `please "..."`, `please chat`, etc.) locally, run `cargo run -- setup` first and configure a provider key the same way an end user would — this is saved to `~/.please/config` and is not part of the repository.

## Project layout

- `src/main.rs` — thin entry point, hands off to `dispatch.rs`.
- `src/dispatch.rs` — routes `please`'s argv to a subcommand, including fuzzy-matching typo'd command names.
- `src/commands/*.rs` — one file per `please` subcommand (`commit.rs`, `push.rs`, `squash.rs`, and so on). This is almost always where a new command or a change to an existing one belongs.
- `src/git.rs` — every raw `git` subprocess call the rest of the codebase needs. If a command needs a new git operation, add it here rather than shelling out to `git` directly from `src/commands/`.
- `src/llm/` — the AI provider layer. `mod.rs` holds provider-neutral dispatch logic; `anthropic.rs`, `gemini.rs`, and `openai.rs` are thin adapters that each implement the same functions for their own provider.
- `src/config.rs` — reads and writes `~/.please/config`.
- `src/context.rs` — builds and caches the project description at `.git/PLEASE.MD` that gives the AI context about the codebase.
- `src/sensitive.rs` — detection for sensitive files (`.env`, credentials, private keys) that guardrails skip during staging.
- `src/gitignore.rs` — logic for auto-adding untracked build/dependency directories to `.gitignore`.
- `src/update_checker.rs` — checks for and applies new releases (`please update`).
- `src/ui.rs` — terminal styling helpers (colors, step/success/warning/error lines) used by commands for user-facing output.
- `src/assets/` — static assets bundled with the repo (README images, help text).
- `man/` — man page source.

## Adding a new command

Following the shape of an existing command (e.g. `src/commands/squash.rs`) is the fastest way to get this right. At minimum, a new command needs:

1. `src/commands/<name>.rs` with a `pub fn run(args: &[String])`.
2. A `pub mod <name>;` line in `src/commands/mod.rs`.
3. An entry in the `COMMANDS` table in `src/dispatch.rs`.
4. A line in `src/commands/help.rs`'s output.
5. If it's something the AI agent should be able to run on your behalf, see [Making a command AI-agent aware](#making-a-command-ai-agent-aware) below.

Keep git operations themselves in `src/git.rs`, not inline in the command file — this keeps every raw git call auditable in one place, which matters for a tool whose entire pitch is "you never type raw git yourself."

### Destructive commands

Any command that can lose work (discard uncommitted changes, rewrite history, delete a branch, force-push) must follow the existing confirmation pattern: explain the consequences in plain language, show exactly what will be lost or changed, then require the user to type `yes` before proceeding. See `src/commands/discard.rs` or `src/commands/purge.rs` for reference implementations. Never add a `--force`/`--yes` flag that skips this prompt for a destructive command without discussing it with a maintainer first.

### Making a command AI-agent aware

If it's something the AI agent should be able to run on the user's behalf, add it to the subcommand list in `run_please`'s tool description in `src/commands/agent.rs`. Two safety nets apply automatically and shouldn't be bypassed:

- Anything destructive run via raw `git`/`gh` by the agent stops and asks the user to confirm before running.
- Any `please` subcommand that would normally prompt for confirmation itself cancels the same way for the agent as it would for any other non-interactive caller — the agent relays that back to the user rather than pretending it succeeded.

## Adding a new AI provider

The tool-calling conversation is represented in provider-neutral terms inside `src/llm/mod.rs`; each provider only needs a thin adapter (see `anthropic.rs`, `gemini.rs`, or `openai.rs` for the shape) that translates that neutral representation to and from its own wire format — model listing, key validation, message formatting, and tool-call parsing. Match an existing adapter's function signatures rather than introducing new ones, so `mod.rs` doesn't need special-casing per provider.

## Before opening a pull request

CI runs formatting, linting, and tests on every push and PR — run the same checks locally first so nothing surprises you:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

`cargo fmt --all` (without `--check`) will fix formatting for you. Fix any `clippy` warnings rather than suppressing them with `#[allow(...)]` unless there's a genuine reason the lint doesn't apply — if so, leave a short comment explaining why.

Add or update tests alongside any behavior change. A new command doesn't need exhaustive coverage, but its confirmation flow (if destructive) and its core success/failure paths should be tested.

## Commit messages

This project loosely follows [Conventional Commits](https://www.conventionalcommits.org/) style (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, ...) — not enforced, but appreciated. Keep the subject line short and focused on *why* the change was made where that isn't obvious from the diff alone.

## Creating a pull request

Changes to `please` happen with the following process:

1. Fork the [`HylithLabs/please`](https://github.com/HylithLabs/please) repository to your own GitHub account.
2. Create a new branch off `main` with a sensible name (e.g. `add-stash-drop-confirmation`) — please don't commit directly to `main` in your fork.
3. Make your changes, following the [project layout](#project-layout) and conventions above.
4. Run the checks in [Before opening a pull request](#before-opening-a-pull-request) locally.
5. Push your branch and open a pull request against `HylithLabs/please`'s `main` branch.
6. Fill out the PR description with what changed and why. If it resolves an open issue, add `Resolves #<issue>` so GitHub links the two.
7. Delete your branch (or your whole fork, if you don't plan to contribute again) once the PR is merged.

If you're planning a larger change (a new command category, a new AI provider, a change to the confirmation/guardrail model), please open an issue to discuss the approach first — it's much easier to course-correct before code is written than after.

## Fixing any CI errors

Every pull request runs the same `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo build` checks described above via GitHub Actions. If CI fails, open the failing job's log from the PR's "Checks" tab to see exactly which step and which file it's complaining about, fix it locally, and push again — there's no need to open a new PR.

If a check fails in a way that seems unrelated to your change (a flaky test, an environment issue), mention it in the PR rather than trying to work around it, and a maintainer will help sort it out.

## Reviewing and merging

Once CI is passing, a maintainer will review your pull request. Expect feedback on:

- Whether the change fits the [design philosophy](README.md#design-philosophy) in the README — no hand-written git, explicit overrides over blanket blocks, bounded AI latency, and confirmation for anything destructive.
- Whether a new destructive command follows the existing confirmation pattern.
- Test coverage for the behavior being added or changed.

Once a maintainer is happy with your PR, they'll merge it. You can then delete your fork (or branch, if you plan to contribute again).

Thank you for contributing to `please`!

## Reporting bugs and requesting features

Open a [GitHub issue](https://github.com/HylithLabs/please/issues/new). Include your OS, how you installed `please` (installer script, Homebrew, or `cargo install`), the exact command you ran, and what you expected versus what happened.

## Reporting security vulnerabilities

Please do **not** open a public issue for a security vulnerability. See [SECURITY.md](SECURITY.md) for how to report it responsibly.

## License

By contributing to `please`, you agree that your contributions will be licensed under its [Apache 2.0 license](LICENSE).
