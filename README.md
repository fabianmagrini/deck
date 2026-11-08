# Deck — AI Agent Workbench

A native macOS desktop application for supervising, interacting with and reviewing AI coding agents. Built with **Rust** and **[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)** — the GPU-accelerated UI framework powering the Zed editor.

> One workspace where developers can launch AI coding agents, observe their activity, inspect proposed changes, approve sensitive operations and understand what each agent has accomplished.

## Status

**Milestones 0–2 complete.** The application launches, renders a three-panel Tokyo Night UI, and runs a full simulated agent interaction through the mock adapter. Real ACP agent integration, Git diff inspection, and SQLite persistence are planned for Milestones 3–4.

## Screenshots

Three-panel layout (left sidebar · conversation · inspector) with Tokyo Night dark theme. Session list, streaming mock agent responses, tool activity, and approval flow are all functional.

## Architecture

```
┌───────────────────────────────────────────────────┐
│                 workbench-app (GPUI)               │
│   Workspace · Sessions · Conversation · Inspector  │
├───────────────────────────────────────────────────┤
│              workbench-agents                      │
│   AgentAdapter trait · MockAgent · (ACP, future)  │
├───────────────────────────────────────────────────┤
│              workbench-core                        │
│   Domain types · Event model · Error types        │
├───────────────────────────────────────────────────┤
│   workbench-storage (stub)  workbench-git (stub)  │
└───────────────────────────────────────────────────┘
```

**Dependency rules:**
- `workbench-core` has no dependency on GPUI.
- `workbench-agents`, `workbench-storage`, and `workbench-git` depend only on `workbench-core`.
- `workbench-app` composes everything and owns GPUI state.
- Domain events are serialisable without importing GPUI types.

## Prerequisites

| Requirement | Version |
|-------------|---------|
| macOS | 13+ (Apple Silicon recommended) |
| Rust | stable 1.97+ (`rustup update stable`) |
| Xcode | 15+ (for Metal toolchain) |
| Metal Toolchain | Required for GPUI's renderer |

### Install the Metal Toolchain

GPUI compiles Metal shaders at build time. The standard Xcode installation does not include the full Metal toolchain; download it once:

```bash
xcodebuild -downloadComponent MetalToolchain
```

The toolchain lands in an Apple cryptex volume. Set the `TOOLCHAINS` environment variable so the build system can find it:

```bash
export TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4
```

Add this to your shell profile (`~/.zshrc`) to avoid setting it on every build.

## Building

```bash
cargo build -p workbench-app
```

Or with the `TOOLCHAINS` variable inline if not set in your environment:

```bash
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 cargo build -p workbench-app
```

## Running

```bash
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 ./target/debug/deck
```

The window opens at 1280 × 800 px, centred on screen, with the Tokyo Night dark theme. Three mock agent sessions are pre-loaded in the sidebar.

**Keyboard shortcuts:**

| Shortcut | Action |
|----------|--------|
| `cmd-n` | New session |
| `cmd-return` | Send prompt |
| `cmd-q` | Quit |
| `cmd-a / c / v / x` | Select all / copy / paste / cut in input |
| `←` `→` `shift-←` `shift-→` | Cursor movement and selection |
| `home` / `end` | Jump to start / end of input |

## Testing

```bash
cargo test --workspace
```

27 tests across `workbench-core` and `workbench-agents`:
- Session state machine transitions (happy path, invalid transitions, approval cycle, cancellation, disconnect)
- Event serialisation round-trips
- Mock agent event log shape and ordering
- Approval resolved/denied flows

## Workspace layout

```
Cargo.toml                  # workspace root
Cargo.lock
spec.md                     # full product & engineering specification
crates/
  workbench-app/            # GPUI application binary (bin: deck)
    src/
      main.rs               # entry point
      app.rs                # keybindings, window creation
      theme.rs              # Tokyo Night colour palette
      actions.rs            # GPUI action types
      views/
        mod.rs
        workbench.rs        # root Workbench view (three-panel layout)
        input.rs            # TextInput widget with IME support
  workbench-core/           # domain types, events, errors (no GPUI)
    src/
      lib.rs
      models.rs             # Workspace, AgentSession, SessionStatus, AgentConfig
      events.rs             # WorkbenchEvent, WorkbenchEventKind
      errors.rs             # WorkbenchError
  workbench-agents/         # AgentAdapter trait and implementations
    src/
      lib.rs
      adapter.rs            # AgentAdapter trait
      mock.rs               # deterministic mock for UI dev and tests
  workbench-storage/        # SQLite persistence (Milestone 4, stubbed)
  workbench-git/            # Git status and diff (Milestone 3, stubbed)
```

## Roadmap

| Milestone | Status | Description |
|-----------|--------|-------------|
| 0 — Foundation | ✅ | Cargo workspace, compiling GPUI app, dark-theme window |
| 1 — Workspace & UI | ✅ | Three-panel layout, session list, conversation, timeline |
| 2 — Mock agent | ✅ | AgentAdapter trait, mock events, approval flow |
| 3 — Real ACP agent | 🔲 | ACP process adapter, real prompt execution |
| 4 — Git & persistence | 🔲 | Git diffs, SQLite storage, session restore |
| 5 — Hardening | 🔲 | Performance benchmarks, accessibility, security review |

## Design decisions

**GPUI over Tauri/SwiftUI** — this project is explicitly an experiment to evaluate GPUI's suitability for AI-native developer tooling. The goal is to understand its strengths and limitations before committing to a larger implementation.

**Synchronous `AgentAdapter` for MVP** — the trait is synchronous in Milestone 2 to keep the mock simple. Async adapters (required for the ACP integration in Milestone 3) will wrap the synchronous contract or replace it.

**Tokyo Night theme** — matches the spec's dark-theme-first requirement and provides a visually coherent palette defined entirely in `workbench-app/src/theme.rs`.

## Licence

MIT
