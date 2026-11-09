# Architecture — Deck AI Agent Workbench

This document explains the structural decisions in the codebase: what each layer does, why the boundaries are where they are, and what an agent should understand before making changes.

---

## 1. Layered architecture

```
┌───────────────────────────────────────────────────┐
│                 workbench-app                      │
│         GPUI views · keybindings · theme           │
│    Workbench · TextInput · actions · app setup     │
├───────────────────────────────────────────────────┤
│              workbench-agents                      │
│       AgentAdapter trait · MockAgent               │
│            (ACP adapter — Milestone 3)             │
├───────────────────────────────────────────────────┤
│              workbench-core                        │
│  Workspace · AgentSession · SessionStatus          │
│  WorkbenchEvent · WorkbenchEventKind               │
│  WorkbenchError · AgentConfig                      │
├──────────────────────┬────────────────────────────┤
│   workbench-storage  │      workbench-git          │
│   SQLite (M4 stub)   │   Git status/diff (M3 stub) │
└──────────────────────┴────────────────────────────┘
```

Each layer depends only on the layer(s) below it. `workbench-app` is the only crate that imports GPUI.

---

## 2. `workbench-core` — the shared foundation

**Rule: no GPUI dependency, ever.**

This crate defines the types that every other crate speaks. By keeping it GPUI-free:

- Domain types remain serialisable with standard `serde` — no GPUI import needed to persist or transmit events.
- Tests for business logic run in milliseconds with no Metal toolchain.
- A future CLI tool or daemon could use the same domain model without pulling in a UI framework.

### Session state machine

`SessionStatus` enforces its transition table through `can_transition_to`. The table is the single source of truth for valid lifecycle moves. Never assign `session.status` directly — always call `session.transition(next)?`.

```
Created → Initializing → Ready → Running ──→ Completed
                        ↘ Failed     ↓
                                 AwaitingApproval ↔ Running
                                       ↓
                                  Cancelling → Completed | Failed
                  any state → Disconnected
```

### Event model

All agent activity is normalised into `WorkbenchEvent` / `WorkbenchEventKind`. Every event carries a session ID and a monotonically increasing sequence number. This design:

- Decouples the UI from any particular agent wire protocol.
- Makes events replayable from storage for debugging and UI restoration.
- Allows the mock adapter and a real ACP adapter to produce identical event types.

Events must be fully serialisable via `serde_json` without importing GPUI.

---

## 3. `workbench-agents` — the adapter layer

**Rule: depends on `workbench-core`, not on `workbench-app` or GPUI.**

The `AgentAdapter` trait is the integration boundary. The UI only ever calls:

```rust
adapter.run_prompt(session_id, prompt) -> Vec<WorkbenchEvent>
adapter.resolve_approval(session_id, request_id, approved) -> Vec<WorkbenchEvent>
```

This keeps agent backends fully swappable. The `MockAgent` returns scripted event logs; a future `AcpAgent` will drive a subprocess — both present the same interface to the UI.

### Synchronous vs async

The trait is synchronous for Milestone 2. The UI dispatches adapter calls on a background thread and collects the returned event log. This was a deliberate simplification to get the full event pipeline working before introducing async complexity.

Milestone 3 will either introduce an `async fn` version of the trait (via `async_trait`) or an event-streaming channel model. See [ADR-003](decisions/003-sync-agent-adapter.md).

---

## 4. `workbench-app` — the GPUI application

**Rule: the only crate that imports GPUI. All UI state lives here.**

### Key views

**`Workbench`** is the root GPUI entity. It owns:
- The session list (`Vec<SessionSummary>`)
- The active session index
- The event log for the active session (`Vec<WorkbenchEvent>`)
- The `TextInput` entity for the prompt field
- The `Arc<MockAgent>` adapter reference

**`TextInput`** is a custom GPUI element implementing `EntityInputHandler` for full macOS IME support. It manages cursor position, selection, and marked text ranges internally. When the user submits, it fires the `SendPrompt` action which `Workbench` handles.

### Render helpers and `'static`

GPUI requires all returned `impl IntoElement` values to be `'static`. Render helper methods on `&self` must therefore **clone or own all data before building the element tree** — they cannot borrow from `self` into the returned element. See [ADR-004](decisions/004-gpui-element-lifetime.md).

### Theme

All colours are in `theme.rs` as named functions returning `gpui::Hsla`. Never hard-code hex literals in view code. This keeps the entire palette in one place and makes a future light-theme variant straightforward.

---

## 5. Stub crates

`workbench-storage` and `workbench-git` are intentional empty stubs. They exist to:

1. Establish the correct dependency boundary now, before the implementations exist.
2. Allow `workbench-app` to import their future public APIs without restructuring the workspace.
3. Signal to agents and contributors that these are planned, not forgotten.

**Do not add implementation to these crates until the corresponding milestone is explicitly in scope.**

---

## 6. Event delivery (current vs target)

**Current (Milestone 2):** Synchronous. The UI calls `adapter.run_prompt()`, receives a `Vec<WorkbenchEvent>`, and applies them all at once. There is no streaming or background thread.

**Target (Milestone 3+):** Asynchronous. A background task drives the ACP adapter and sends events to the UI through a bounded channel. GPUI entities are updated via `cx.notify()` on the main thread. The key constraint from the spec:

> Do not access GPUI entities directly from worker threads. Deliver messages to an application-side bridge, then update entities through the GPUI context.

---

## 7. What is explicitly out of scope (MVP)

- LLM inference or agent orchestration engine
- Language Server Protocol
- Interactive PTY / terminal emulation
- Pull request creation
- Multiple concurrent agents
- Cloud or remote execution
- Plugin system

These are documented in the spec (`spec.md §14`) and in [ADR-001](decisions/001-gpui-over-alternatives.md).
