# workbench-core

Core domain types for the Deck AI Agent Workbench. This crate is the shared foundation for all other crates in the workspace and has **no dependency on GPUI** or any UI framework.

## Contents

| Module | Description |
|--------|-------------|
| `models` | `Workspace`, `AgentSession`, `SessionStatus`, `AgentConfig` |
| `events` | `WorkbenchEvent`, `WorkbenchEventKind` — the ordered event log |
| `errors` | `WorkbenchError` — top-level error enum |

## Key types

### `SessionStatus`

An enum that models the full session lifecycle with an enforced transition table:

```
Created → Initializing → Ready → Running → Completed
                       ↘ Failed     ↓
                                AwaitingApproval ↔ Running
                                     ↓
                                Cancelling → Completed | Failed
                  any state → Disconnected
```

Use `SessionStatus::can_transition_to` to validate moves before calling `AgentSession::transition`.

### `WorkbenchEventKind`

All agent activity is normalised into a single discriminated enum. Events are sequenced per session and serialisable to JSON for persistence and replay:

```rust
SessionStarted
MessageDelta { text }     // streamed assistant text chunks
MessageCompleted
ToolStarted { id, name }
ToolUpdated { id, output }
ToolCompleted { id, success }
ApprovalRequested { request_id, description, resources }
ApprovalResolved { request_id, approved }
FilesChanged { paths }
PromptCompleted
SessionFailed { message }
```

## Tests

```bash
cargo test -p workbench-core
```

14 tests covering state machine transitions, serialisation round-trips, and domain invariants.
