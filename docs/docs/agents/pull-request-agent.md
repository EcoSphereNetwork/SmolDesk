---
title: Pull Request Agent
description: Codex
---
# Pull Request Agent

This agent scans open pull requests using the GitHub CLI and applies the rules defined in `.codex.json`.

- Skip drafts and PRs marked with `do not merge`.
- Merge clean PRs via squash using their title as the commit message.
- Attempt to rebase conflicting branches; create an issue if the conflict cannot be resolved automatically.
