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
