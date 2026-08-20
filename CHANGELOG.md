# Changelog

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
