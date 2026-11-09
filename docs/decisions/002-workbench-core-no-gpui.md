# ADR-002 — `workbench-core` must not depend on GPUI

**Date:** November 2026  
**Status:** Accepted

## Context

Domain types (`Workspace`, `AgentSession`, `WorkbenchEvent`, etc.) are needed by every crate. We need to decide whether they can import GPUI types.

## Decision

`workbench-core` **must have no dependency on GPUI**. This is enforced by the `Cargo.toml` for that crate — GPUI is simply not listed as a dependency.

## Rationale

1. **Serialisation without a UI framework.** Events must be serialisable to JSON for SQLite persistence and log replay. GPUI types do not implement `serde::Serialize`. Keeping domain types free of GPUI means the event log works independently.
2. **Fast tests.** GPUI tests require a Metal renderer, which requires the Metal Toolchain and a macOS GPU context. Pure domain tests run in milliseconds anywhere. The 27-test suite for `workbench-core` and `workbench-agents` runs with a plain `cargo test`.
3. **Future flexibility.** A CLI companion tool, a daemon, or a test harness could use the same domain model without pulling in a GPU framework.
4. **Enforced by the dependency graph.** The rule is not a convention — it is mechanically enforced. A future agent cannot accidentally add GPUI to `workbench-core` without a `Cargo.toml` edit that is immediately visible in review.

## Consequence

GPUI-specific display helpers (e.g. converting `SessionStatus` to a colour) live in `workbench-app`, not in `workbench-core`. The theme module owns colour mapping; the domain model owns state.
