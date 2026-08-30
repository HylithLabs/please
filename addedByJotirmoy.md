# Features Added by Jotirmoy

## Repository health and review

```terminal
please doctor
```

Checks Git availability and version, repository state, detached HEAD, remotes, merge conflicts, uncommitted changes, and AI provider configuration. It only reports safe fixes.

```terminal
please review
```

Sends the current tracked, staged, unstaged, and untracked changes to the configured AI provider. The assistant summarizes the changes, identifies bugs or suspicious modifications, and recommends tests.

## Conflict resolution and recovery

```terminal
please resolve
```

Inspects conflicted files, explains both sides, proposes resolutions, applies approved changes, and runs validation through the AI assistant.

```terminal
please undo --preview
```

Shows which commit will be undone and explains what will happen before changing history.

```terminal
please recover
```

Lists recent reflog entries and creates a recovery branch without changing the current branch.

```terminal
please recover changes
```

Restores the safety copy created before discarded changes are removed.

```terminal
please discard
```

Creates a recoverable safety copy before deleting uncommitted work.

## Daily workflow improvements

```terminal
please status
```

Displays a concise summary of changed files and the branch's ahead/behind state.

```terminal
please commit
```

Detects common test setups such as Cargo, npm, pytest, and Makefile projects. Test results are included in commit confirmations.

```terminal
please start "add dark mode"
```

Creates a purpose-based feature branch, such as `feature/add-dark-mode`, and records the branch purpose locally.

```terminal
please cleanup --preview
```

Shows merged branches that can be cleaned up. Branch deletion requires approval.

## Stacked changes and releases

```terminal
please split
```

Uses AI assistance to separate a messy working tree into logical commits.

```terminal
please reorder
```

Uses AI assistance to safely reorder recent commits after explaining the proposed change.

```terminal
please combine
```

Uses AI assistance to combine related commits with confirmation before rewriting history.

```terminal
please release
```

Analyzes commits since the latest release and drafts release notes with a suggested version. Tagging and pushing require approval.

## Project context and conversations

```terminal
.please/instructions.md
```

Provides a user-editable place for project conventions, commit style, test commands, and sensitive paths.

```terminal
please context
```

Displays the cached project context and project instructions.

```terminal
please context --refresh
```

Regenerates the cached project context after major repository changes.

```terminal
please chat
```

Runs the interactive AI conversation and saves useful conversation history locally.

```terminal
please chat --last
```

Displays the most recently saved conversation.

## Guided next steps and repository-aware push checks

After a command finishes, `please` can display a green `Next:` suggestion based
on the current repository state. For example, it may suggest `please commit`
when changes are present, `please push` when local commits are ready to share,
or `please status` after switching branches or syncing.

The AI chat features can also be used outside a Git repository. Run
`please chat` or send a natural-language request without running `please init`
first. Git-specific actions still explain that a repository is required.

Before `please push` continues, it checks for common problems such as a
detached HEAD, missing remotes, merge conflicts, or an unconfigured AI
provider. When one is detected, it asks whether to run `please doctor` first.
Answer `y` to run the health check and stop before pushing, or `n` to continue
with the normal commit-and-push flow.

## GitHub publishing and changing remotes

```terminal
please github <github-origin-url>
```

Initializes Git in a new project, switches to `main`, creates a `first commit`,
configures the GitHub repository as `origin`, and pushes it. If `origin` is
already configured, the URL can be omitted with `please github`.

```terminal
please change origin <url>
```

Changes the repository's `origin` remote without pushing any data.

```terminal
please change origin and push <url>
```

Changes `origin` and pushes all local branches and tags to the new remote,
preserving the repository's existing commits and history. These operations
ask for confirmation before changing the remote or pushing.
