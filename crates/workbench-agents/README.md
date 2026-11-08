# workbench-agents

Agent adapter layer for the Deck AI Agent Workbench. Defines the `AgentAdapter` trait and ships a deterministic `MockAgent` for UI development and testing.

## Contents

| Module | Description |
|--------|-------------|
| `adapter` | `AgentAdapter` — the trait every backend must implement |
| `mock` | `MockAgent` — scripted, in-process adapter with no external dependencies |

## `AgentAdapter`

The synchronous MVP trait. All agent integrations implement this interface so the UI can work with any backend without change:

```rust
pub trait AgentAdapter: Send + Sync {
    fn adapter_id(&self) -> &str;
    fn display_name(&self) -> &str;

    /// Run a prompt and return the complete ordered event log.
    fn run_prompt(&self, session_id: Uuid, prompt: &str) -> Vec<WorkbenchEvent>;

    /// Resolve a pending ApprovalRequested event.
    fn resolve_approval(&self, session_id: Uuid, request_id: Uuid, approved: bool)
        -> Vec<WorkbenchEvent>;
}
```

> **Note:** The synchronous signature is intentional for Milestone 2. Async adapters will be introduced for the ACP integration in Milestone 3.

## `MockAgent`

A fully deterministic adapter that returns scripted event sequences based on prompt keywords:

| Prompt contains | Tool used |
|-----------------|-----------|
| `test` | `cargo test` |
| `build` | `cargo build` |
| anything else | `read_file` |

Every `run_prompt` call produces:
1. `SessionStarted`
2. Streaming `MessageDelta` chunks
3. `MessageCompleted`
4. `ToolStarted` → `ToolUpdated` → `ToolCompleted`
5. `FilesChanged`
6. `ApprovalRequested`

`resolve_approval(approved: true)` appends tool execution events and `PromptCompleted`.  
`resolve_approval(approved: false)` returns only `ApprovalResolved` and `PromptCompleted`.

## Adding a new adapter

1. Create a struct in a new module under `src/`.
2. Implement `AgentAdapter`.
3. Export it from `lib.rs`.
4. Register it in `workbench-app/src/app.rs`.

## Tests

```bash
cargo test -p workbench-agents
```

13 tests covering event shapes, ordering, keyword dispatch, approval flows, and adapter metadata.
