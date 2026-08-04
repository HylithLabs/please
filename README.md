# Please Git

An AI-native git CLI. You never type raw `git` commands — you run `please` commands, and an AI agent (Anthropic Claude, Google Gemini, or OpenAI ChatGPT — your choice) handles staging, commit messages, and pushing on your behalf.

Git remains the engine underneath; `please` is the interface.

Run `please` with no arguments (or `please help`) any time for a full command reference, grouped by what you're trying to do.

Typos and filler words don't block a command either — `please swich to master` still runs `please switch master`. A typo'd command word is recognized if it's one edit away from a real command (a wrong letter, a missing one, an extra one, or two adjacent letters swapped) and isn't also close to some *other* command, and small connector words like "to"/"into"/"the"/"a"/"an" are dropped before the real argument is passed through. Anything looser than that is deliberately left alone and handed to the AI agent instead — "clean up my messy code" should ask the AI to look at your code, not silently run `please cleanup` and start deleting branches.

## Install

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

Or build it yourself with `cargo install --path .` if you'd rather not run a prebuilt binary — note that `please update` (below) only works for the prebuilt installs above, since it has no source tree to rebuild from.

## Setup

Run once per machine:

```bash
please setup
```

You'll pick a provider: **Anthropic** (Claude), **Google** (Gemini), or **OpenAI** (ChatGPT), and paste an API key. `please` checks the key against the provider right away (a real, cheap call: listing available models), so a typo gets caught immediately instead of failing confusingly on your first `please commit`; if it fails, you're offered another try without restarting setup. It then auto-picks the cheapest model the key has access to, so there's no need to know model names or pricing tiers, though you can decline the pick and type a specific model id yourself if you'd rather choose. This is saved globally to `~/.please/config` and applies to every project on the machine.

You can save keys for more than one provider (adding a second one never overwrites the first), switch which is active any time, and change a saved provider's model later without re-entering its key. Re-run `please setup` and it shows what's already configured, then lets you add or update a provider, switch the active one, change a provider's model, or remove a saved key:

```
Providers you've set up:
  * Anthropic (model: claude-haiku-4-5, key ending in ...ab12)
    Google (model: models/gemini-flash-lite-latest, key ending in ...hnxg)
  (* = active: this is what `please` uses right now)

What would you like to do?
  1) Add or update a provider
  2) Switch the active provider
  3) Change a provider's model
  4) Remove a saved provider
  5) Nothing, just checking
```

On first use in a repo, `please` also generates a short project description cached at `.git/PLEASE.MD`, giving the AI context about the codebase.

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

### `please stash` / `please stash list` / `please stash pop` / `please stash drop`

No AI involved. `please stash` saves everything in the working tree, tracked and untracked, and clears it, so you can switch context and come back later — replaces `git stash push -u`. `please stash list` shows what's saved. `please stash pop` restores the most recent one; if it conflicts, it lists the conflicting files and tells you to resolve them and `please commit`, or `please discard` to cancel the restore, which cleanly cancels it without losing the stash. `please stash drop` deletes the most recent one outright, showing what it is and requiring `yes` first since it can't be recovered afterward.

### `please squash` / `please squash <n>` / `please squash <branch-or-commit>`

Combines a run of commits into one, AI-written message and all, the other boring, error-prone task nobody enjoys: `git rebase -i`, editing a todo file by hand, resolving whatever it trips on. `please squash` does it with a single `git reset --soft` back to a starting point instead, so there's no rebase to go wrong.

No argument squashes everything ahead of the branch's upstream (or the repo's default branch, if it hasn't been pushed yet) — the usual "clean up before merging" case. A number squashes the last `<n>` commits; a branch or commit name squashes back to it directly (it has to actually be an ancestor of HEAD, so this can't jump the branch somewhere unrelated). Either way, it shows every commit about to be combined and requires typing `yes` first; if the branch has already been pushed, that same `yes` also covers force-pushing the result back with `--force-with-lease`, right after, so it fails safely instead of clobbering anything if someone else pushed to it meanwhile.

```
please squash
please squash 3
please squash main
```

### `please purge <path>`

Permanently removes a file or folder from your entire git history, not just the working tree, the boring, error-prone task of scrubbing a leaked secret or an accidentally committed file out of every commit that ever touched it. Uses `git filter-repo` if it's installed (git's own recommended tool for this), falling back to the built-in `git filter-branch` otherwise, then cleans up the now-unreachable objects so the old content is actually gone, not just unreferenced.

This rewrites commit hashes for the file's entire history and everything after it, so it shows you exactly what that means and requires typing `yes` first. If you have an `origin` remote, a single `yes` also force-pushes the rewritten history there right after, since a leaked secret is still live on the remote until that happens; you're told upfront that's part of the plan, and every collaborator needs to re-clone or `please sync exactly` afterward, since their local history has now diverged permanently.

```
please purge secrets.env
please purge config/credentials
```

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
- Any `please` subcommand that would normally ask for confirmation itself (`please discard`, `please sync exactly`, `please revert`, `please stash drop`, creating a branch via `please switch`) can't be rubber-stamped by the agent — it cancels itself exactly as it would for any non-interactive caller, and the agent relays that back to you rather than pretending it succeeded, so you can run it yourself and confirm it directly.

Built to add new providers without touching the agent loop: the tool-calling conversation is represented in provider-neutral terms internally, and only a small adapter per provider (Gemini, Claude, ChatGPT) translates that to and from its own wire format.

### `please chat`

The same agent, kept alive across multiple messages instead of one and done. Use `please "..."` for a single request; switch to `please chat` when you want to go back and forth — ask a follow-up, correct something, or build up a multi-step task piece by piece, without repeating context each time:

```
$ please chat
> what changed since the last commit?
...
> undo that last one, actually
...
> good, now make a branch for the fix
...
> exit
```

Type `exit`/`quit` or press Ctrl+D to leave. Same tools, same safety nets as `please "..."` — the only difference is the conversation itself persists in memory for the life of the session (nothing is written to disk), so later messages can refer back to what was said or done earlier.

### `please alias <name>` / `please alias` / `please alias remove <name>`

Give `please` a shorter name, like `pls` or `plz`, so casual daily use is less to type. `please alias pls` creates a real symlink next to the `please` binary itself, so `pls` becomes a genuine command, not a shell alias: it works in every shell, in scripts, and doesn't need a new terminal or a sourced rc file to take effect. `please alias` on its own lists what you've set up; `please alias remove <name>` takes one back. If the name you pick already exists as a different program elsewhere on your `PATH`, it warns before shadowing it and asks you to confirm; if it collides with an unrelated file right next to `please` itself, it refuses outright rather than overwriting something that isn't its own.

### `please update`

Updates `please` itself to the latest release in place, no reinstalling by hand. Only works if you installed it via the shell/PowerShell installer or Homebrew above — those leave behind a record of how they installed it that this reads; a `cargo install --path .` build has no such record, so it's told to rebuild from source instead of failing mysteriously.

## Design notes

- **No hand-written git.** Every git operation a developer needs is reachable through a `please` command.
- **Explicit overrides, not blanket blocks.** Guardrails (sensitive files, junk directories) can always be bypassed by staging the file yourself first — the tool assumes intent over blocking outright.
- **Bounded AI latency.** The model's own response time isn't something we control, but everything around it is: requests are timeout-bounded, progress is printed so the CLI never looks frozen, and the model is auto-selected once at setup (self-healing on failure) rather than re-selected on every call.
- **Destructive operations require confirmation.** Anything that can discard work (`please sync exactly`) explains the consequences up front and requires explicit confirmation.

## Roadmap

See `PROJECT.MD` for the full vision, including the planned **Auto Commit** feature (AI commits automatically as you reach meaningful checkpoints, no command needed) and a configuration GUI.
