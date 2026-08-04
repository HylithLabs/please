# Contributing to please

Thanks for taking the time to contribute. This document covers how the
project is laid out and how to get a change merged.

## Getting set up

You need a recent stable Rust toolchain (the project targets the 2024
edition) and `git`.

```bash
git clone https://github.com/HylithLabs/please.git
cd please
cargo build
```

Run it locally with `cargo run -- <command>`, e.g. `cargo run -- help`.

## Before opening a pull request

CI runs formatting, linting, and tests on every push and PR — run the same
checks locally first so nothing surprises you:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --all-features
```

`cargo fmt --all` (without `--check`) will fix formatting for you.

## Project layout

- `src/main.rs` — thin entry point, hands off to `dispatch.rs`.
- `src/dispatch.rs` — routes `please`'s argv to a subcommand, including
  fuzzy-matching typo'd command names.
- `src/commands/*.rs` — one file per `please` subcommand (`commit.rs`,
  `push.rs`, `squash.rs`, and so on). This is almost always where a new
  command or a change to an existing one belongs.
- `src/git.rs` — every raw `git` subprocess call the rest of the codebase
  needs. If a command needs a new git operation, add it here rather than
  shelling out to `git` directly from `src/commands/`.
- `src/llm/` — the AI provider layer. `mod.rs` holds provider-neutral
  dispatch logic; `anthropic.rs`, `gemini.rs`, and `openai.rs` are thin
  adapters that each implement the same functions for their own provider.
- `src/config.rs` — reads and writes `~/.please/config`.
- `src/ui.rs` — terminal styling helpers (colors, step/success/warning/error
  lines) used by commands for user-facing output.

## Adding a new command

Following the shape of an existing command (e.g. `src/commands/squash.rs`)
is the fastest way to get this right. At minimum, a new command needs:

1. `src/commands/<name>.rs` with a `pub fn run(args: &[String])`.
2. A `pub mod <name>;` line in `src/commands/mod.rs`.
3. An entry in the `COMMANDS` table in `src/dispatch.rs`.
4. A line in `src/commands/help.rs`'s output.
5. If it's something the AI agent should be able to run on your behalf, add
   it to the subcommand list in `run_please`'s tool description in
   `src/commands/agent.rs`.

Destructive commands (anything that can lose work) should follow the
existing confirmation pattern: explain the consequences, then require typing
`yes` before proceeding — see `src/commands/discard.rs` or
`src/commands/purge.rs` for examples.

## Commit messages

This project loosely follows [Conventional Commits](https://www.conventionalcommits.org/)
style (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, ...) — not enforced,
but appreciated.

## Reporting bugs and requesting features

Open a GitHub issue. For security vulnerabilities, see
[SECURITY.md](SECURITY.md) instead of filing a public issue.
