# Please Git

## Vision

**Please Git** is an AI-native Git distribution that extends the standard Git experience with an intelligent AI agent.

Instead of developers manually staging changes, writing commit messages, and managing repetitive Git workflows, they interact with Git through simple natural-language commands while the AI understands the repository and performs the required Git operations.

The goal is not to replace Git, but to build an AI-powered Git distribution in the same philosophy as Microsoft's Git distribution with Scalar—extending Git while preserving its core functionality.

---

# Installation

The user installs **Please Git** using the installer provided for their operating system.

Examples:

* Windows (`.exe`)
* Linux (`.deb`)
* macOS (`.pkg`)

After installation, the user has a Git distribution that includes the standard Git functionality together with the AI capabilities.

---

# Configuration

The project includes a graphical interface where users configure the software.

The interface allows users to manage:

* LLM provider
* API key
* Model selection
* Software settings

Once configured, developers continue working primarily from the terminal.

---

# AI Commands

The primary command is:

```bash
please push
```

When executed, the AI agent will:

* Connect to the Git repository.
* Read the current changes.
* Analyze the Git diff.
* Understand the repository context.
* Understand what the developer changed.
* Generate an appropriate commit message.
* Commit the changes.
* Push the commit to the remote repository.

The workflow is fully automated without requiring the user to manually perform each Git step.

---

# AI Agent

The AI is not a commit message generator.

It acts as an intelligent Git agent.

It understands the repository, understands the current work, and performs Git operations on behalf of the developer.

The objective is to remove repetitive Git tasks while maintaining a fast terminal workflow.

---

# Native Architecture

Please Git is designed as a native Git distribution.

Git remains the version control engine.

The AI layer extends Git with intelligent capabilities while preserving compatibility with existing repositories and Git workflows.

The project combines:

* Git
* AI agent
* Native terminal commands
* Configuration GUI

into a single developer tool.

---

# Future Feature — Auto Commit

A future capability of Please Git is **Auto Commit**.

The AI continuously monitors changes in the codebase.

When it determines that the developer has completed a meaningful unit of work, it automatically creates a commit for those changes without requiring the developer to manually run a commit command.

The AI manages commit creation as development progresses.

---

# Goal

Please Git aims to transform Git into an AI-native developer experience.

Developers continue using the terminal as usual, but instead of manually managing Git operations, they simply instruct the AI, allowing it to understand the project and execute the workflow automatically.