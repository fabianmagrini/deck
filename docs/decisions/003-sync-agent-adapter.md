# ADR-003 — Synchronous `AgentAdapter` for Milestone 2

**Date:** November 2026  
**Status:** Accepted (revisit at Milestone 3)

## Context

The `AgentAdapter` trait needs to handle agent I/O. The two obvious designs are:

- **Synchronous:** `fn run_prompt(...) -> Vec<WorkbenchEvent>` — returns a complete event log.
- **Async streaming:** `async fn run_prompt(...) -> impl Stream<Item = WorkbenchEvent>` — yields events as they arrive.

## Decision

Use a **synchronous trait** that returns a complete `Vec<WorkbenchEvent>` for Milestone 2.

## Rationale

1. **The mock adapter has no I/O.** `MockAgent` generates events deterministically in memory. There is no network call, subprocess, or timer to await. Async machinery would add complexity with no benefit.
2. **Unblocks the full event pipeline sooner.** The synchronous signature allowed the entire UI pipeline — event log rendering, approval flow, tool activity display — to be built and tested before any real agent integration exists.
3. **Avoids premature async abstraction.** The ACP adapter (Milestone 3) will likely need a streaming model, but its exact requirements (channel-based, `async_trait`, GPUI's own async primitives) are not yet known. Designing the trait now for async risks locking in the wrong model.

## Consequence

The UI currently calls `adapter.run_prompt()` synchronously. This is acceptable because the mock is fast. When a real adapter is added in Milestone 3, the call must be moved off the UI thread (e.g. via `cx.spawn` or a background thread + channel).

## Revisit trigger

When starting Milestone 3 (ACP adapter), replace or extend this trait. Options to evaluate:
- `async fn` via `async_trait` crate
- An event-channel model: `fn run_prompt(..., tx: Sender<WorkbenchEvent>)`
- GPUI's own task system (`cx.background_executor().spawn(...)`)

The key constraint (from the spec): **never block the GPUI UI thread on agent I/O**.
