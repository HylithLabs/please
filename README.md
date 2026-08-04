# Please Git

An AI-native git CLI. You never type raw `git` commands — you run `please` commands, and an AI agent (currently Google Gemini) handles staging, commit messages, and pushing on your behalf.

Git remains the engine underneath; `please` is the interface.

## Install

```bash
cargo install --path .
```

## Setup

Run once per machine:

```bash
please setup
```

You'll pick an LLM provider and paste an API key (Google AI Studio for Gemini). This is saved globally to `~/.please/config` and applies to every project on the machine. On first use in a repo, `please` also generates a short project description cached at `.git/PLEASE.MD`, giving the AI context about the codebase.

## Commands

### `please commit`

Stages your changes, sends the diff to the AI, and lets it split the work into one or more logically coherent commits — each with its own files and message. You never run `git add` or write a commit message yourself.

Guardrails run automatically before staging:
- **Sensitive files** (`.env`, credentials, private keys, etc.) are skipped unless you've already staged them yourself — that's treated as explicit intent to include them.
- **Build/dependency directories** (`node_modules`, `dist`, `target`, `.venv`, etc.) that aren't already tracked or ignored are added to `.gitignore` automatically, so beginners without one don't accidentally commit junk.

### `please push`

Runs `please commit`, then pushes the current branch to `origin` (setting the upstream on first push).

### `please status`

A plain-language view of where you stand: current branch, how far ahead/behind the remote you are, and what's changed — no staged/unstaged jargon.

### `please branch [name]`

No name: lists local branches. With a name: creates and switches to a new branch.

### `please switch <name>`

Switches to an existing branch. If the branch doesn't exist, offers to create and switch to it.

### `please sync`

Fetches and merges the current branch's upstream in (like `git pull`). Reports "up to date" or the merge result; on a real conflict, shows git's conflict output and points you to `please commit` once you've resolved it.

### `please sync exactly`

Makes the local branch match its remote exactly — discards local commits and uncommitted changes not on the remote (untracked files are left alone). Destructive, so it shows exactly what will be lost and requires typing `yes` to proceed.

### `please undo` / `please redo`

Undoes the last commit — replaces `git reset --soft HEAD~1` — leaving its changes back in your working tree so you can fix them and try again. `please redo` brings it back, as long as history hasn't moved on since (a new commit invalidates it).

### `please move-commit <branch>`

Fixes "committed to the wrong branch": moves your last commit onto a new branch and switches you to it — replaces `git branch new && git reset --hard HEAD~1 && git checkout new`. Refuses if you have uncommitted changes, so nothing else gets swept up in the move.

### `please discard`

Throws away all uncommitted changes, tracked and untracked — replaces `git checkout -- . && git clean -fd`. Shows exactly what will be lost and requires typing `yes` to proceed.

### `please restore <path>`

Brings back a file that was deleted in a past commit — replaces hunting through `git log --diff-filter=D` for the deleting commit and `git checkout <sha>^ -- <path>`.

### `please branch delete <name>`

Deletes a branch locally and on `origin` in one step — replaces `git branch -d name && git push origin --delete name`. Refuses to delete the branch you're currently on.

### `please rename <new-name>`

Renames the current branch, including on the remote if it's been pushed — replaces `git branch -m old new && git push origin -u new && git push origin --delete old`.

### `please cleanup`

Deletes local branches already merged into the repo's main branch — replaces `git branch --merged main | grep -v main | xargs git branch -d`. Only ever removes branches git already considers safe to delete (`git branch -d`, not `-D`).

### `please log`

A readable commit graph — replaces remembering `git log --oneline --graph --decorate`.

### `please revert`

Interactive, no AI involved. Lists your recent commits with a serial number and hash next to each; you pick one either way, and it's reverted — replaces hunting down a SHA with `git log` and running `git revert <sha>` yourself. Won't run into a wall of uncommitted changes either: if your tree is dirty it tells you up front to `please commit` or `please discard` first, and if the revert itself conflicts, it lists the conflicting files and tells you to resolve them and `please commit`, or `please discard` to cancel — never just a raw git error.

### `please "<what you want to do>"`

Agent mode: anything you type that isn't one of the commands above is treated as a plain-language request. The AI figures out how to do it and acts on your behalf — it never edits files directly, only ever acting through `git`, `gh`, or another `please` subcommand, calling itself recursively where that helps (e.g. "commit and clean up merged branches" might run `please commit` then `please cleanup`).

```
please "clean up branches that are already merged into main"
please "what changed in the last 3 commits?"
please "I broke something, undo my last commit"
please "open a PR for this branch"
```

Two safety nets, not one:
- Anything destructive run via raw `git`/`gh` (force-push, `reset --hard`, `branch -D`, deleting things) stops and asks you to confirm before running.
- Any `please` subcommand that would normally ask for confirmation itself (`please discard`, `please sync exactly`, `please revert`, creating a branch via `please switch`) can't be rubber-stamped by the agent — it cancels itself exactly as it would for any non-interactive caller, and the agent relays that back to you rather than pretending it succeeded, so you can run it yourself and confirm it directly.

Built to add new providers without touching the agent loop: the tool-calling conversation is represented in provider-neutral terms internally, and only a small adapter (currently just Gemini's) translates that to and from a given provider's wire format.

## Design notes

- **No hand-written git.** Every git operation a developer needs is reachable through a `please` command.
- **Explicit overrides, not blanket blocks.** Guardrails (sensitive files, junk directories) can always be bypassed by staging the file yourself first — the tool assumes intent over blocking outright.
- **Bounded AI latency.** Gemini's own response time isn't something we control, but everything around it is: requests are timeout-bounded, progress is printed so the CLI never looks frozen, and the model is auto-selected once at setup (self-healing on failure) rather than re-selected on every call.
- **Destructive operations require confirmation.** Anything that can discard work (`please sync exactly`) explains the consequences up front and requires explicit confirmation.

## Roadmap

See `PROJECT.MD` for the full vision, including the planned **Auto Commit** feature (AI commits automatically as you reach meaningful checkpoints, no command needed) and a configuration GUI.
