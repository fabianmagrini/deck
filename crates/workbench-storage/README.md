# workbench-storage

SQLite persistence layer for the Deck AI Agent Workbench. **Stubbed — planned for Milestone 4.**

## Planned scope

This crate will provide durable storage for workspaces, agent sessions, event logs, and UI preferences using SQLite (via `rusqlite` or `sqlx`) in the macOS application-support directory.

Proposed tables:

| Table | Contents |
|-------|----------|
| `workspaces` | Registered workspace directories and metadata |
| `agent_configs` | Registered agent adapter configurations |
| `sessions` | Session records with status and timestamps |
| `events` | Full ordered event log per session (JSON payload) |
| `settings` | Key/value UI and application preferences |

Migrations will be versioned and applied at startup.

## Current status

The crate compiles as an empty stub. All state is held in memory within `workbench-app` for Milestones 0–2.
