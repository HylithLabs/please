# Changelog

## Version 2.1.0

* Removed the `Next:` step hint that printed after almost every command. In its place `please` now shows an occasional colorful `Tip:` one-liner, roughly one run in five, pointing at a feature you might not have reached for yet
* Added `please run set "<command>"` and `please run reset`. These record how to start a project by hand with no AI call, so you (or `please chat`) can tell `please run` the real steps once and have every later run replay them
* `please run set` also takes `--check "<probe>"` to guard the run behind a health check (for example `docker info`) and `--term` to mark it as a long-running process
* `please run` now launches long-running processes such as dev servers and file watchers in a new terminal window, leaving your current shell free. Pass `--here` to force it inline or `--new-terminal` to force a window
* `please chat` can now set up `please run` for you: explain how the project starts and the agent saves it with `please run set`
* Reworked the update checker so it behaves like the rest of `please`: no background timer, no scheduled job, nothing running while `please` is idle. It checks only when you run a command, shows the result from cache instantly, then refreshes in a fully detached background process for the next run. It stays quiet under `NO_UPDATE_NOTIFIER`, `CI`, and piped output, and `please update ignore` now sticks across refreshes

## Version 2.0.0

* Added `please github [url]` — initializes Git, switches to `main`, creates the first commit, sets `origin`, and pushes in one step
* Added `please change origin <url>` and `please change origin and push <url>` — move a repository to another remote, optionally republishing all branches and tags
* Added `please doctor` — diagnoses Git, repository, and AI provider health without changing anything
* Added `please review` — sends the current working-tree diff to the AI for a bug/risk summary and suggested tests before you commit
* Added `please run` / `please run reset` — reads a small set of manifest files (README, package.json, Cargo.toml, Dockerfile, and similar), asks the AI how to start the project, confirms once, then remembers the decision outside the working tree so every run after that replays it with no AI call
* Added `please recover` / `please recover changes` — brings back a recent commit or a safety stash created by `discard`
* Added `please context [--refresh]` — inspect or regenerate the AI's cached project context
* Added `please start "purpose"` — creates a purpose-named feature branch, such as `feature/add-dark-mode`
* Added `please resolve` — explains and helps resolve merge conflicts with AI
* Added `please split`, `please reorder`, `please combine`, `please release` — AI-assisted commit history organizing and release-note drafting
* `please` no longer refuses outright outside a Git repository — it now asks whether to `git init` right there, so a plain folder does not feel broken
* Fixed `please review` crashing with `fatal: bad revision 'HEAD'` on a repository with no commits yet, by diffing staged and unstaged changes separately instead of against `HEAD`
* Fixed `please review`'s untracked-file diff always using the Windows null device, breaking it on macOS and Linux
* Fixed `please doctor` and `please status` crashing on a repository with no commits yet
* Fixed `please commit` crashing on a repository's first-ever commit
* Fixed a secret-leak vulnerability in `please github`'s first-commit path, where `.env` and other sensitive files could be committed and pushed despite an on-screen "skipped" warning, because the underlying unstage step silently failed on a repository with no commits yet
* Fixed `please commit` always running the full project test suite, even with feedback mode off and nobody to see the result — it now only runs when `--feedback` is on
* `please` no longer creates a `.please` folder in your working tree — internal state now lives inside `.git/please/` (or a per-project cache under your home directory when there is no repository yet); `.please/instructions.md` is the one exception, since it is meant to be committed and shared with your team

## Version 0.1.9

* Added `please init` — runs `git init` if this directory isn't already a git repo, then eagerly generates `please.md`, the AI-written project description
* Added `please clone <url>` — a plain, literal passthrough to `git clone`, no AI involved
* Every other command now checks for a git repo up front and points at `please init` or `please clone` instead of leaking git's raw "fatal: not a git repository" error

## Version 0.1.8

* Fixed `please update` always failing on Homebrew installs with "Can't check for updates: unable to load receipt" — Homebrew never writes the cargo-dist install receipt the updater looks for, so `please update` now detects a Homebrew install and runs `brew upgrade please` automatically instead

## Version 0.1.7

* Overhauled the AI agent's permission system: every mutating `git`/`gh`/`please` action now asks for confirmation by default, replacing the old destructive-flag blocklist with a read-only allowlist so nothing risky slips through unflagged
* Commands that can only be confirmed interactively (discard, purge, revert, squash, sync exactly, stash drop, switching to a new branch) are now refused up front by the agent instead of silently failing after a wasted round trip
* Fixed `please chat` reasoning from a stale, turn-one snapshot of branch/upstream/working-tree state for the whole session — repo state is now refreshed before every message
* The agent's reasoning is now printed before it runs a tool, so confirmation prompts aren't answered blind
* Fixed `please update` showing a redundant "update available" notice right after updating
* Fixed release notes never showing after `please update`: the lookup guessed a `v{version}` tag that doesn't match every release (0.1.6 is tagged plain `0.1.6`), so it 404'd silently — now fetches by "latest" instead

## Version 0.1.6

* Added global `--feedback` / `-f` flag for interactive confirmation before destructive or major commands (like commit, push, and agent operations)
* Implemented `please config` interactive menu to persistently toggle the feedback loop


## Version 0.1.5

* Implemented background update checking mechanism for zero latency
* Added automatic fetching and rendering of release notes after please update
* Fixed clippy reference slicing errors in dispatch routing
* Changed AI output to render in rich Markdown using termimad
* Created custom Please Dark color scheme using owo colors
* Added native offline manual pages through please man command
* Intercepted standard help flags to render native manual seamlessly
