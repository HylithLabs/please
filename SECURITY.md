# Security Policy

## Supported Versions

`please` is pre-1.0 and releases roll forward — only the latest published
release is supported. Please make sure you're on the
[latest version](https://github.com/HylithLabs/please/releases/latest)
(`please update`, or reinstall) before reporting an issue.

## Reporting a Vulnerability

Please do not open a public issue for security vulnerabilities.

Instead, use GitHub's private vulnerability reporting for this repository:

1. Go to the [Security tab](https://github.com/HylithLabs/please/security).
2. Click **Report a vulnerability**.

This opens a private advisory visible only to you and the maintainers, so
the issue can be discussed and fixed before it's public.

If it isn't a sensitive issue — e.g. a hardening suggestion with no
exploitable impact on its own — a regular GitHub issue is fine instead.

## Scope

Things worth reporting privately: anything that could leak an API key
stored by `please setup`, unintended remote code execution, or a way for a
repository's contents to make `please` run something the developer didn't
ask for.

Things that are expected behavior, not vulnerabilities: `please`'s AI
agent mode running `git`/`gh` commands on your behalf — that's the intended
feature, gated by the confirmation prompts described in the
[README](README.md#please-what-you-want-to-do).
