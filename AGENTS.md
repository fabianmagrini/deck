# Agent Instructions — Deck AI Agent Workbench

This file is read by AI coding agents (Claude Code, etc.) at the start of every session. It contains everything needed to work effectively on this codebase without re-deriving context from scratch.

---

## Quick orientation

**What this is:** A native macOS desktop app built with Rust + GPUI for supervising AI coding agents. It is an experiment to evaluate GPUI's suitability for AI-native developer tooling — not a production product.

**Current status:** Milestones 0–2 complete. The app builds and launches. A mock agent drives the full event pipeline. Real ACP agent integration, Git diff inspection, and SQLite persistence are not yet implemented.

**Primary objective:** Maintain architectural simplicity. Prefer the smallest correct change over clever abstractions.

---

## Build and run

### Prerequisite — Metal Toolchain

GPUI compiles Metal shaders at build time. Set this environment variable before every build:

```bash
export TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4
```

Without it, the build fails with:
```
cargo::error=metal shader compilation failed: cannot execute tool 'metal'
```

Add it to `~/.zshrc` to avoid setting it every time.

### Commands

```bash
# Build the application
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 cargo build -p workbench-app

# Run the application
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 ./target/debug/deck

# Run all tests (no Metal toolchain needed)
cargo test --workspace

# Check formatting
cargo fmt --check

# Run linter
cargo clippy --workspace -- -D warnings
```

---

## Crate map

| Crate | Binary/lib | Purpose |
|-------|-----------|---------|
| `workbench-app` | bin `deck` | GPUI views, keybindings, app setup |
| `workbench-core` | lib | Domain types, events, errors — **no GPUI** |
| `workbench-agents` | lib | `AgentAdapter` trait + `MockAgent` |
| `workbench-storage` | lib | SQLite persistence — **stub, Milestone 4** |
| `workbench-git` | lib | Git status/diff — **stub, Milestone 3** |

---

## Hard rules — do not break these

1. **`workbench-core` must never import GPUI.** Domain types must be serialisable without UI framework types.
2. **`workbench-agents`, `workbench-storage`, `workbench-git` must not import GPUI.** Only `workbench-app` owns GPUI state.
3. **Never call blocking operations from a GPUI render function.** `render()` and render helpers run on the UI thread.
4. **The Metal Toolchain env var is required for every build that touches GPUI.** Document it in any new build scripts.
5. **Do not invent GPUI APIs.** GPUI 0.2.2 is pinned; it differs from Zed's current main. Always verify method signatures against `~/.cargo/registry/src/.../gpui-0.2.2/src/`.
6. **State machine transitions must go through `AgentSession::transition`.** Never assign `session.status` directly.
7. **Keep `Cargo.lock` committed.** This is a binary application; reproducible builds require it.

---

## GPUI 0.2.2 gotchas

These are non-obvious API differences from newer Zed versions that have already caused build failures:

| Issue | Correct form in 0.2.2 |
|-------|----------------------|
| `AppContext` trait for `cx.new()` on `&mut App` | Must `use gpui::AppContext` explicitly — not in prelude |
| `Window::focus` | Takes 1 arg: `window.focus(&handle)` — no `cx` param |
| `overflow_y_scroll()` | Only on `Stateful<Div>` — call `.id("some-id")` first |
| `EntityInputHandler` trait | Has exactly 8 methods; no `set_selected_text_range`, `text_length_utf16`, `selection_bounds_for_range` |
| `Window` notify methods | `notify_text_input_changed`, `notify_text_selection_changed`, `request_virtual_keyboard` do not exist |
| `ShapedLine::paint` | 4 args: `(origin, line_height, window, cx)` — no `TextAlign` or `None` |
| `impl IntoElement` return from `&self` helpers | Returned element must be `'static`; clone/own all data before building the element tree |

---

## Where things live

### Views (`workbench-app/src/views/`)

- **`workbench.rs`** — Root `Workbench` GPUI view. Owns the three-panel layout, session list, all event handling. `Workbench::new` takes `&mut Context<Self>`.
- **`input.rs`** — `TextInput` widget with full IME via `EntityInputHandler`. `TextElement` is the custom GPUI element that renders the cursor and selection.

### Domain types (`workbench-core/src/`)

- **`models.rs`** — `Workspace`, `AgentSession`, `SessionStatus` (with transition table), `AgentConfig`
- **`events.rs`** — `WorkbenchEvent`, `WorkbenchEventKind`
- **`errors.rs`** — `WorkbenchError`

### Agent adapters (`workbench-agents/src/`)

- **`adapter.rs`** — `AgentAdapter` trait (synchronous for Milestone 2)
- **`mock.rs`** — `MockAgent`: scripted events keyed on prompt keywords (`test` → `cargo test`, `build` → `cargo build`, else `read_file`)

---

## Testing

27 tests across `workbench-core` and `workbench-agents`. Run with:

```bash
cargo test --workspace
```

Expected output: all 27 pass, 0 failures.

When adding features, tests are required for:
- Any new `SessionStatus` transitions
- Any new `WorkbenchEventKind` variants (serialisation round-trip)
- Any new `AgentAdapter` implementation (event shape, ordering, adapter metadata)
- Any new service-layer business logic

GPUI view tests are not yet wired up (requires GPUI's test harness — deferred to Milestone 5).

---

## Milestone context

| Milestone | Status | What was built |
|-----------|--------|----------------|
| 0 — Foundation | ✅ | Cargo workspace, compiling GPUI app, Metal toolchain, logging |
| 1 — Workspace & UI | ✅ | Three-panel layout, session list, conversation panel, Tokyo Night theme |
| 2 — Mock agent | ✅ | `AgentAdapter` trait, `MockAgent`, event pipeline, approval flow |
| 3 — Real ACP agent | 🔲 | ACP process adapter — `workbench-agents/src/acp.rs` to be created |
| 4 — Git & persistence | 🔲 | `workbench-git` and `workbench-storage` stubs to be implemented |
| 5 — Hardening | 🔲 | Benchmarks, accessibility, security review |

**Do not build Milestone 3 or 4 features unless the task explicitly says so.** The stubs exist to hold the dependency boundary — leave them as stubs.

---

## Coding standards (from spec)

- Idiomatic Rust; prefer composition and traits.
- Explicit error types at domain and integration boundaries — use `WorkbenchError`.
- No global mutable state for application services.
- No blocking operations on the GPUI/UI thread.
- Typed GPUI actions for all keyboard commands.
- Format with `cargo fmt` before committing.
- Run `cargo clippy --workspace -- -D warnings` and fix all warnings.
- Document significant architecture decisions as ADRs in `docs/decisions/`.
