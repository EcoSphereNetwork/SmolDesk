# Agent Development Guide

This guide explains how to extend SmolDesk using LLM driven agents.

## Principles
- Follow the process defined in `AGENTS.md`.
- Keep changes small and testable.
- Document new commands and scripts.

## Adding a New Agent
1. Create a description under `docs/docs/agents/`.
2. Provide a script or entrypoint if the agent requires one.
3. Update `.codex.json` with default actions.
4. Reference new agent files in AGENTS.md so all agents follow the same lifecycle.

## CI-fähige Agenten-Abläufe
Agents trigger tests and merges via the GitHub CLI. Configure authentication with
`gh auth login` and use `gh pr list` to find open requests. Automated merges
follow the policies in `.codex.json` and post comments on success or failure.
