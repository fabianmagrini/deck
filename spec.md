# AI Agent Workbench

Product & Technical Specification · Version 1.0 · November 2026

Rust + GPUI

macOS-first

Agent Client Protocol

Local-first

## Product vision

Build a high-performance, native desktop application for supervising, interacting with and reviewing AI coding agents.

AI Agent Workbench is not intended to replace VS Code, Zed, Cursor or an existing agent runtime. It is an experimental desktop control plane for coding agents, designed to explore the capabilities of GPUI and the ergonomics of Rust-native desktop software.

The central idea is:

> One desktop workspace where developers can launch AI coding agents, observe their activity, inspect proposed changes, approve sensitive operations and understand what each agent has accomplished.

The initial implementation should optimise for architectural simplicity, extensibility and learning rather than feature completeness.

## 1. Product goals

Native desktop experience

Explore GPUI's rendering model, entities, actions, keyboard handling, window management and application lifecycle.

Agent orchestration UI

Manage multiple agent sessions without building another LLM inference or coding-agent engine.

Human-in-the-loop workflow

Make agent decisions, tool activity, code changes, approvals and outcomes visible.

Performance experiment

Benchmark GPUI's startup, memory usage, rendering responsiveness and development ergonomics.

## 2. Proposed user experience

The application follows a developer-tool layout with a persistent workspace explorer, a central agent conversation and activity area, an inspection panel and a bottom execution console.

Workbench

/ demo-project

&#x20;Agent active

WORKSPACE

&#x20;demo-project

&#x20;src

&#x20;main.rs

&#x20;agent.rs

&#x20;lib.rs

AGENT SESSIONS

&#x20;Refactor CLI

&#x20;Add tests

&#x20;Fix logging

Refactor CLI

Running

Refactor the command parser to improve testability and add unit tests.

I'll inspect the current parser, extract the command handling and run the test suite.

&#x20;Read 4 source files

&#x20;Updated parser.rs

&#x20;Running cargo test

Message agent...

INSPECTOR

CHANGES

&#x20;parser.rs

+42 lines

−18 lines

PENDING APPROVAL

Execute cargo test

Review required

&#x20;EXECUTION OUTPUT

cargo test

running 18 tests

test result: ok. 18 passed; 0 failed

Conceptual UI layout. The first iteration should use standard GPUI elements before investing in a custom component library.

The UI should be keyboard-first, dark-theme-first and support resizable panels.

## 3. MVP scope

The first usable release should deliver six capabilities.

| Feature                       | Priority | MVP behaviour                                   |
| ----------------------------- | -------- | ----------------------------------------------- |
| Workspace management          | P0       | Open a local directory and browse files         |
| Agent sessions                | P0       | Start, select, cancel and inspect sessions      |
| Agent conversation            | P0       | Send prompts and stream responses               |
| Activity timeline             | P0       | Display tool execution, status and errors       |
| Permission requests           | P0       | Approve or reject supported agent tool requests |
| Change inspection             | P0       | Display Git status and unified diffs            |
| Session persistence           | P1       | Restore local history after restart             |
| Execution output              | P1       | Read structured process output and logs         |
| Embedded interactive terminal | P2       | Full PTY terminal with input                    |
| Multiple concurrent agents    | P1       | Supervise independently running sessions        |
| Git commit workflow           | P2       | Stage and commit selected changes               |

MVP boundary: Start with one active agent execution at a time, but allow multiple stored sessions and switching between them. General concurrency, interactive terminals and advanced Git operations can follow once the GPUI foundation is stable.

The MVP should use an ACP-compatible agent as its first real integration. ACP is designed specifically to connect developer tools with coding agents and includes streaming, sessions, tool activity and permission interactions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 4. Technical implementation specification

The following is the implementation brief for building the first version with a coding agent.

# AI Agent Workbench — Engineering Specification

**Version:** 1.0\
**Target platform:** macOS (Apple Silicon)\
**Language:** Rust\
**UI framework:** GPUI\
**Agent integration:** Agent Client Protocol (ACP)\
**Architecture:** Local-first, modular Rust application

---

## 1. Architecture

### 1.1 Architectural principles

1. **GPUI owns presentation, not business logic.** All agent execution, persistence, file operations and Git functionality must work independently of the UI.
2. **Protocol-first agent integration.** Prefer established agent protocols over proprietary CLI output parsing.
3. **No custom agent runtime for the MVP.** Use existing coding agents through adapters.
4. **Asynchronous by design.** Long-running operations must never block the UI thread.
5. **Explicit user control.** Permissions, cancellation and potentially destructive operations must have clear controls.
6. **Local-first.** No account, backend server or cloud database is required.
7. **Replaceable integrations.** Components should depend on Rust traits rather than specific agent implementations.
8. **Testability.** The application must run against a deterministic mock agent without external AI credentials.

### 1.2 Logical architecture

```
┌───────────────────────────────────────────────────┐
│                  GPUI Desktop App                 │
│                                                   │
│  Workspace  Sessions  Conversation  Inspector     │
│  Explorer   Timeline  Approvals     Output        │
├───────────────────────────────────────────────────┤
│               Application Services                │
│                                                   │
│  WorkspaceService      SessionService             │
│  AgentOrchestrator      ApprovalService            │
│  ChangeInspection      PersistenceService         │
├───────────────────────────────────────────────────┤
│                  Integration Layer                │
│                                                   │
│  AgentAdapter    GitAdapter     FileSystemAdapter │
│  EventStore      ProcessManager                   │
├───────────────────────────────────────────────────┤
│                   Agent Runtimes                  │
│                                                   │
│  ACP Agent  │  Mock Agent  │  Future CLI Adapter  │
└───────────────────────────────────────────────────┘
```

The integration layer translates external agent activity into application-domain events.

The GPUI application subscribes to these events and updates its entities.

### 1.3 Process model

Use a single desktop application process with child processes for external coding agents.

- GPUI runs the application event loop.
- Asynchronous services manage I/O and subprocesses.
- ACP communicates over the selected supported transport, initially stdio.
- The agent process operates in the selected workspace directory.
- The event store persists session and execution history.
- The UI receives updates through a bounded event channel.

Do not access GPUI entities directly from worker threads. Deliver messages to an application-side bridge, then update entities through the GPUI context.

Do not assume the agent runtime is trusted merely because it is a local process.

---

## 2. Technology stack

| Concern                  | Recommended technology                            |
| ------------------------ | ------------------------------------------------- |
| Programming language     | Rust, current stable toolchain                    |
| Desktop framework        | Official Zed GPUI                                 |
| Component library        | Native GPUI elements initially; evaluate GPUI Kit |
| Agent protocol           | Official ACP Rust SDK                             |
| Async execution          | Tokio, with a deliberate GPUI event-loop bridge   |
| Serialization            | serde / serde_json                                |
| Local persistence        | SQLite via sqlx                                   |
| Git integration          | git2 or the Git CLI behind a trait                |
| Structured logs          | tracing / tracing-subscriber                      |
| Error handling           | thiserror / anyhow at application boundaries      |
| Identifiers              | uuid                                              |
| Time                     | time                                              |
| Testing                  | cargo test, GPUI test facilities                  |
| Benchmarking             | Criterion and native OS instrumentation           |
| Interactive PTY (future) | portable-pty                                      |

Pin dependencies using Cargo.lock. Do not use wildcard dependency versions.

Do not mix incompatible GPUI variants or component-library snapshots. Verify the selected UI dependency graph by compiling a minimal application before implementing features.

---

## 3. Repository structure

Use a Cargo workspace with clear internal boundaries.

```
agent-workbench/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── AGENTS.md
├── docs/
│   ├── architecture.md
│   ├── decisions/
│   ├── benchmark-plan.md
│   └── development.md
├── crates/
│   ├── workbench-app/
│   │   ├── src/main.rs
│   │   ├── src/app.rs
│   │   ├── src/theme.rs
│   │   ├── src/actions.rs
│   │   └── src/views/
│   │       ├── workspace.rs
│   │       ├── sessions.rs
│   │       ├── conversation.rs
│   │       ├── timeline.rs
│   │       ├── inspector.rs
│   │       └── approvals.rs
│   ├── workbench-core/
│   │   └── src/
│   │       ├── models.rs
│   │       ├── events.rs
│   │       ├── services.rs
│   │       └── errors.rs
│   ├── workbench-agents/
│   │   └── src/
│   │       ├── adapter.rs
│   │       ├── acp.rs
│   │       ├── mock.rs
│   │       └── process.rs
│   ├── workbench-storage/
│   │   └── src/
│   │       ├── sqlite.rs
│   │       └── migrations/
│   └── workbench-git/
│       └── src/
│           ├── status.rs
│           └── diff.rs
├── fixtures/
│   ├── mock-agent/
│   └── sample-project/
├── tests/
│   ├── integration/
│   └── fixtures/
└── scripts/
    └── benchmark.sh
```

### Dependency boundaries

- `workbench-core` must have no dependency on GPUI.
- `workbench-agents`, `workbench-storage` and `workbench-git` depend on core domain contracts.
- `workbench-app` composes the application and owns GPUI-specific state.
- Integrations must not import UI views.
- Domain events must be serialisable without importing GPUI types.

---

## 4. Domain model

### 4.1 Workspace

Represents a local project directory.

```
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub root_path: PathBuf,
    pub created_at: OffsetDateTime,
    pub last_opened_at: OffsetDateTime,
}
```

Requirements:

- Users can open existing directories.
- The application validates that the path exists.
- Recent workspaces are persisted.
- File browsing is restricted to the selected workspace unless explicitly authorised.
- Symlinks and canonical paths must be handled safely.

### 4.2 Agent session

```
pub struct AgentSession {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub adapter_id: String,
    pub title: String,
    pub status: SessionStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

pub enum SessionStatus {
    Created,
    Initializing,
    Ready,
    Running,
    AwaitingApproval,
    Cancelling,
    Completed,
    Failed,
    Disconnected,
}
```

Session state must be validated through a transition table.

A completed prompt turn does not necessarily mean the persistent agent session is closed. The service layer must distinguish prompt execution state from session lifecycle state.

### 4.3 Event model

All agent interactions must be normalised into a common event envelope.

```
pub struct WorkbenchEvent {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sequence: u64,
    pub timestamp: OffsetDateTime,
    pub kind: WorkbenchEventKind,
}

pub enum WorkbenchEventKind {
    SessionStarted,
    MessageDelta { text: String },
    MessageCompleted,
    ToolStarted { id: String, name: String },
    ToolUpdated { id: String, output: String },
    ToolCompleted { id: String, success: bool },
    ApprovalRequested { request_id: Uuid },
    ApprovalResolved { request_id: Uuid, approved: bool },
    FilesChanged { paths: Vec<PathBuf> },
    PromptCompleted,
    SessionFailed { message: String },
}
```

This model is illustrative. Introduce additional typed payloads as actual ACP integration requirements emerge.

**Important:** Do not assume every agent emits identical events or supports identical capabilities. The adapter must translate capabilities and preserve useful source metadata.

### 4.4 Event delivery

- Persist important events before acknowledging durable processing.
- Assign monotonically increasing sequence numbers per session.
- Deliver events to the UI in order.
- Coalesce high-frequency streaming text updates.
- Use bounded channels and backpressure.
- Never silently drop permission requests or final session states.
- Allow replay from persisted events for debugging and UI restoration.

---

## 5. Agent integration

### 5.1 AgentAdapter contract

Define a Rust trait describing the behaviour required by the application.

Conceptually:

```
#[async_trait::async_trait]
pub trait AgentAdapter: Send + Sync {
    async fn connect(
        &self,
        workspace: &Workspace,
    ) -> Result<AgentConnection, AgentError>;

    async fn create_session(
        &self,
        connection: &AgentConnection,
    ) -> Result<AgentSessionHandle, AgentError>;

    async fn send_prompt(
        &self,
        session: &AgentSessionHandle,
        prompt: String,
    ) -> Result<(), AgentError>;

    async fn cancel(
        &self,
        session: &AgentSessionHandle,
    ) -> Result<(), AgentError>;

    async fn respond_to_approval(
        &self,
        request_id: Uuid,
        decision: ApprovalDecision,
    ) -> Result<(), AgentError>;
}
```

The exact trait signatures may evolve as required by the chosen ACP SDK. Event streaming and inbound permission requests should be implemented using explicit async channels or protocol callbacks.

### 5.2 ACP adapter

The first production adapter must implement the Agent Client Protocol.

Support:

- Agent process startup and shutdown.
- Protocol initialisation and capability negotiation.
- Session creation.
- Prompt submission.
- Streaming message updates.
- Tool activity.
- Permission requests and responses.
- Prompt cancellation.
- Process disconnect and error handling.

Capabilities such as session resume, authentication methods and terminal integration must only be enabled when supported by the selected agent.

Do not invent compatibility for agents that do not expose ACP.

### 5.3 Mock adapter

Create a deterministic mock agent capable of:

- Streaming a scripted response.
- Simulating tool execution.
- Requesting approval.
- Generating a test file-change event.
- Simulating cancellation.
- Simulating a crash or disconnection.

Use this adapter for UI development, automated tests and performance experiments.

### 5.4 Initial agent configuration

Allow users to register an ACP-compatible executable through settings.

Store:

- Adapter ID.
- Display name.
- Executable path.
- Arguments.
- Working-directory policy.
- Optional environment variable names.
- Supported capabilities discovered at connection time.

Do not store API keys or sensitive environment variable values in plaintext settings or the event log.

Do not invoke configured commands through a shell unless explicitly required and separately authorised.

---

## 6. UI modules

### 6.1 Application shell

Create a native macOS window with:

- Persistent title and toolbar.
- Left navigation panel.
- Central content area.
- Right inspector.
- Collapsible bottom output panel.
- Global command palette.
- Status bar.

Persist window dimensions, panel visibility and layout preferences.

### 6.2 Workspace explorer

Functions:

- Open workspace.
- Browse directories.
- Expand and collapse folders.
- Open text files in a read-only viewer.
- Show modified-file indicators.
- Refresh following filesystem changes.

Avoid recursively loading the entire file tree during initial rendering. Use lazy expansion, file count limits and virtualised lists where appropriate.

### 6.3 Sessions panel

Functions:

- Create a session.
- Select a previous session.
- Display current status.
- Cancel a running prompt.
- Show active agent and workspace.
- Display error details.
- Distinguish disconnected sessions from completed sessions.

Session switching must not terminate running child processes.

### 6.4 Conversation panel

Requirements:

- Multiline prompt input.
- Submit through keyboard shortcut.
- Markdown-like message presentation.
- Streaming agent response.
- Tool activity blocks.
- Auto-scroll while pinned to bottom.
- Preserve scroll position when inspecting older messages.
- Copy message content.

Do not make network or process calls directly from render functions.

### 6.5 Activity timeline

Each significant event should appear chronologically.

Example:

```
10:42:11  Session started
10:42:12  Agent initialised
10:42:15  Read src/parser.rs
10:42:18  Read src/commands.rs
10:42:23  Edited src/parser.rs
10:42:24  Requested tool permission
10:42:29  Permission granted
10:42:30  Started cargo test
10:42:33  Tests completed successfully
10:42:35  Prompt completed
```

Support filtering by event category and navigating to event details.

### 6.6 Permission dialog

Show:

- Agent identity.
- Tool or operation description.
- Affected resources, when supplied.
- Requested permission choices.
- Approve or reject actions.

The UI must represent the actual options negotiated with the agent.

Do not assume that a UI-level approval is a universal operating-system sandbox. Approval is a protocol-level control unless further execution restrictions are implemented.

A permission request must remain visible until it is resolved or invalidated.

### 6.7 Change inspector

The MVP must support:

- Git working-tree status.
- Modified, created and deleted files.
- Unified text diffs.
- Additions and deletions.
- Refresh after agent activity.
- Detection of user changes made outside the application.

The application must clearly distinguish observed file changes from changes attributable to the agent. A Git diff alone cannot establish authorship.

Do not allow automatic file overwrite through the inspector in the MVP.

### 6.8 Execution output panel

Initially support a read-only, scrollable output stream with:

- Timestamps.
- Output source labels.
- Standard output and standard error distinction.
- Search and copy.
- Bounded memory retention.

A fully interactive terminal is explicitly deferred.

---

## 7. Persistence

### 7.1 Database

Use SQLite in the operating system's application-support directory.

Proposed tables:

```
workspaces
  id
  name
  root_path
  created_at
  last_opened_at

agent_configs
  id
  name
  executable
  arguments_json
  enabled

sessions
  id
  workspace_id
  adapter_id
  external_session_id
  title
  status
  created_at
  updated_at

events
  id
  session_id
  sequence
  timestamp
  event_type
  payload_json

settings
  key
  value_json
```

Use versioned migrations.

### 7.2 Persistence policies

- Persist completed messages and important lifecycle events.
- Persist metadata needed to reconnect where protocol support exists.
- Do not assume an external agent can resume because a session ID was stored.
- Store UI layout preferences separately from runtime state.
- Support configurable event-history retention.
- Avoid storing credentials, sensitive tool payloads and secret-bearing logs by default.

---

## 8. Security and operational controls

Although experimental, the application will run coding agents with access to local source code and potentially local credentials.

The following controls are mandatory:

1. Spawn executables using explicit argument arrays.
2. Display the agent command before first use.
3. Do not execute arbitrary text as shell commands.
4. Treat all agent output as untrusted display content.
5. Restrict the application's own file operations to authorised workspace paths.
6. Require explicit confirmation for destructive application-level actions.
7. Redact known secret patterns in logs where practical.
8. Provide clear cancellation and process-termination controls.
9. Detect unexpected child-process exit.
10. Never claim protocol approval prevents an agent from performing operations through other already-granted capabilities.

Stronger sandboxing, restricted filesystem access and outbound-network controls are future requirements, not implied capabilities of ACP or GPUI.

---

## 9. Performance requirements

These are experiment targets, not GPUI performance guarantees.

| Metric                        | Initial target                   |
| ----------------------------- | -------------------------------- |
| Cold startup                  | Under 1 second on reference Mac  |
| Idle memory                   | Under 150 MB                     |
| UI interaction responsiveness | P95 under 50 ms                  |
| Streaming update latency      | P95 under 100 ms                 |
| Timeline scrolling            | Smooth with 10,000 stored events |
| Large event ingestion         | 1,000 events without UI freeze   |
| Background agent execution    | No blocking on UI thread         |
| Cancellation feedback         | Immediate UI acknowledgement     |

Benchmark performance on a fixed reference machine with repeatable fixtures. Record median, P95 where applicable, variance, runtime versions and test conditions.

Distinguish initial UI feedback from actual agent cancellation or termination.

---

## 10. Testing strategy

### 10.1 Unit tests

Cover:

- Session state transitions.
- Event ordering.
- Event serialization.
- Agent capability mapping.
- Permission decision validation.
- Path containment.
- Persistence and migrations.

### 10.2 Integration tests

Cover:

- Mock agent lifecycle.
- ACP initialisation with a test agent.
- Streaming event delivery.
- Tool permission handling.
- Cancellation.
- Process crash.
- Database restoration.
- External file modifications.

### 10.3 GPUI tests

Cover:

- Window initialisation.
- View rendering.
- Selection and focus.
- Keyboard actions.
- Session switching.
- Permission modal interactions.
- Scroll behaviour.
- UI changes following incoming events.

### 10.4 Performance tests

Create reproducible test fixtures for:

- 1,000 messages.
- 10,000 timeline entries.
- 100 modified files.
- Sustained text streaming.
- Long-running background work.
- Rapid switching between sessions.

Capture CPU, memory and frame responsiveness. Include a benchmark result in the repository.

---

## 11. Implementation roadmap

### Milestone 0 — Technical foundation

Deliver:

- Cargo workspace.
- Compiling GPUI application.
- Dark-theme window.
- Basic application shell.
- Logging and error handling.
- Test and lint pipeline.

**Exit criterion:** A reproducible native application builds and launches on macOS.

### Milestone 1 — Workspace and UI

Deliver:

- Workspace directory picker.
- File explorer.
- Session list.
- Conversation panel.
- Timeline panel.
- Inspector layout.
- Keyboard navigation.

Use static test data only.

**Exit criterion:** The complete interface can be navigated without any agent runtime.

### Milestone 2 — Mock agent

Deliver:

- AgentAdapter trait.
- Deterministic mock implementation.
- Event streaming.
- Prompt input.
- Session state transitions.
- Tool activity display.
- Mock approval flow.
- Cancellation.

**Exit criterion:** A complete simulated agent interaction works end to end.

### Milestone 3 — Real ACP agent

Deliver:

- ACP process adapter.
- Protocol negotiation.
- Real prompt execution.
- Streaming updates.
- Permission requests.
- Cancellation.
- Error and disconnect handling.

**Exit criterion:** A supported external coding agent can successfully execute a task from the workbench.

### Milestone 4 — Git and persistence

Deliver:

- Git status.
- Unified diff viewer.
- SQLite migrations.
- Persisted workspaces and sessions.
- Saved event history.
- Reopen and restore behaviour.

**Exit criterion:** A developer can complete a coding-agent task, inspect its changes, restart the application and review the saved history.

### Milestone 5 — Hardening and benchmarks

Deliver:

- Performance test harness.
- Event batching and virtualised lists.
- Crash and recovery testing.
- Accessibility testing.
- Operational and security review.
- Architecture retrospective.

**Exit criterion:** The experiment produces reliable data about GPUI's suitability for a larger developer application.

---

## 12. Coding standards

- Use idiomatic Rust.
- Prefer composition and traits over inheritance-like abstractions.
- Use explicit error types for domain and integration boundaries.
- Avoid unnecessary cloning of large event payloads.
- Do not use global mutable state for application services.
- Keep view rendering functions free of blocking operations.
- Use typed actions for application commands.
- Separate domain events from GPUI notifications.
- Use dependency injection through constructors.
- Format with `cargo fmt`.
- Run `cargo clippy` with warnings treated as errors.
- Require tests for service logic and state transitions.
- Document significant architecture decisions as ADRs.

Avoid building a generic dependency injection framework, event bus framework or plugin engine for the first iteration.

---

## 13. Definition of done

The MVP is complete when a developer can:

1. Launch the macOS application.
2. Open a local Git repository.
3. Configure an ACP-compatible coding agent.
4. Start an agent session.
5. Submit a coding task.
6. Observe streaming responses and tool activity.
7. Review and respond to supported permission requests.
8. Cancel an executing prompt.
9. Inspect changed files in a Git diff.
10. Close and reopen the application to inspect saved session history.

Additional requirements:

- No blocking UI during agent operations.
- Graceful handling of agent crashes.
- Documented setup instructions.
- Automated unit and integration tests.
- Reproducible performance benchmark.
- No mandatory cloud backend.
- No credentials checked into source control.

---

## 14. Explicitly out of scope

The initial release will not include:

- A bespoke LLM inference or agent orchestration engine.
- Multiple agents collaborating on one task.
- Remote or cloud-hosted execution.
- Full IDE functionality.
- Language Server Protocol integration.
- Interactive PTY emulation.
- Pull request creation and review.
- Git branch or worktree orchestration.
- Enterprise authentication or team collaboration.
- Plugin marketplace.
- Custom MCP server implementation.
- Mobile or web application.
- Automatic merging or deployment.

These capabilities may be considered after the GPUI experiment is evaluated.

---

## 15. Next-phase opportunities

### Multi-agent workspace

Introduce concurrent independent agent sessions with clear process isolation, activity tracking and cancellation.

### Git worktree isolation

Run different agents against separate worktrees to reduce conflicting changes.

### Interactive terminal

Provide a real terminal using a PTY backend and terminal emulation.

### Review and approval centre

Consolidate permission requests, changed files, tests and agent outcomes into a review queue.

### Session replay

Reconstruct prior activity from persisted events for inspection and debugging.

### Agent observability

Introduce execution duration, token usage when available, tool counts, failure rates and task outcome metrics.

### Cross-platform support

Validate Windows and Linux builds after the macOS architecture stabilises.

---

## 16. Engineering guidance for implementation agents

Implement milestone by milestone, preserving compilability at each step.

For every milestone:

1. Inspect the current GPUI and ACP documentation.
2. Implement the smallest complete vertical slice.
3. Add relevant tests.
4. Run formatting, linting and tests.
5. Run or build the native application.
6. Document deviations from the specification.
7. Update the milestone checklist.

Do not invent GPUI APIs. Prefer patterns taken from the installed version's runnable examples.

Do not substitute static data for actual agent integration beyond the mock-agent milestone.

Do not introduce speculative abstractions without an explicit immediate requirement.

**Primary engineering objective:** Produce a maintainable Rust desktop application that demonstrates the effectiveness and limitations of GPUI as the foundation for AI-native developer tooling.

## 5. A few important technology decisions

Three choices are worth making before starting implementation.

GPUI vs GPUI Kit

I would begin with official GPUI for the application shell, then evaluate [GPUI Kit](https://github.com/longbridge/gpui-kit?utm_source=chatgpt.com) for input controls, docking, code viewing and accessibility.

GPUI Kit already offers a substantial component ecosystem, which could reduce the amount of custom UI code. However, adding it immediately might obscure what you're trying to learn about GPUI itself.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



ACP vs bespoke agent integration

Use the [official Rust ACP SDK](https://github.com/agentclientprotocol/rust-sdk?utm_source=chatgpt.com) rather than directly integrating an individual agent's proprietary output protocol. Start with stable ACP v1 and negotiate capabilities.

That gives the experiment an integration boundary that can accommodate multiple coding agents.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Control plane vs coding IDE

Keep Workbench as a control plane initially. File inspection and diffs are sufficient; there is no need to build a complete code editor, language server integration or terminal emulator in the MVP.

This preserves the project's focus on agent supervision and GPUI evaluation.

## 6. How to evaluate the experiment

I'd evaluate it against four questions at the end of the fifth milestone.

GPUI experiment scorecard

Illustrative

Rate each dimension after implementation on a scale from 1 (poor) to 5 (excellent). The default scores are placeholders, not measured results.

Developer productivity

3/5 · 30% weight

How difficult was it to build and maintain the UI?

Performance

3/5 · 30% weight

How does it handle sustained streams and large event histories?

Desktop capabilities

3/5 · 25% weight

How well does it handle keyboard input, windows, focus and accessibility?

Ecosystem maturity

3/5 · 15% weight

How much framework-specific infrastructure did we need to build?

Weighted score

# 3.0 / 5

Reset

The most valuable outcome is not necessarily the application itself. It's understanding whether GPUI is a practical foundation for more ambitious AI-native engineering workspaces, especially when compared with Tauri and SwiftUI.

I would keep the first milestone deliberately small: one macOS window, a three-panel layout, a session list and a deterministic mock agent streaming events into a GPUI timeline. That will expose the essential framework trade-offs before committing to a larger implementation.