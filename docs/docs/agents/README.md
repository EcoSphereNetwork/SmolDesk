# Agent Based Development

SmolDesk uses automated agents like **Codex** to maintain the project. Agents perform analysis, run tests and create pull requests.

## Available Agents
- **Codex** – general repository automation and refactoring.
- **OpenHands** – documentation parser and linter.
- **TestRunner** – executes test suites and reports coverage.

Each agent has a dedicated entry in `.codex.json` with default commands.
Agents collaborate by creating issues and pull requests for each development phase.
Agents follow the workflow described in `AGENTS.md`.

See [agent-types.md](./agent-types.md) for a list of agent categories and
[agent-life-cycle.md](./agent-life-cycle.md) for the workflow. The pull request
process is detailed in [pull-request-agent.md](./pull-request-agent.md) with
additional rules in [merge-strategies.md](./merge-strategies.md). For API usage
see [github-api-access.md](./github-api-access.md).
