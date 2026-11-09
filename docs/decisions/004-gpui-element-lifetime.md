# ADR-004 — Cloning data before building GPUI element trees

**Date:** November 2026  
**Status:** Accepted

## Context

GPUI requires all values returned from `render()` and render helpers to be `'static` (they are stored by the framework between frames). This creates a tension when render helpers are written as `&self` methods that want to reference fields of the view struct.

```rust
// This does NOT compile — returned element borrows `self`
fn render_sessions(&self, cx: &Context<Self>) -> impl IntoElement {
    div().children(self.sessions.iter().map(|s| {
        div().child(s.title.clone())  // closure borrows `s` from `self`
    }))
}
```

The compiler error: `returning this value requires that '1 must outlive 'static`.

## Decision

**Clone or collect all needed data into owned values before building the element tree.** The iterator that feeds `.children()` must work over owned data, not references into `self`.

```rust
fn render_sessions(&self, cx: &Context<Self>) -> impl IntoElement {
    // Collect owned data first
    let items: Vec<(String, &'static str, bool)> = self.sessions.iter()
        .enumerate()
        .map(|(i, s)| (s.title.clone(), s.status.label(), self.active == Some(i)))
        .collect();

    div().children(items.into_iter().map(|(title, label, active)| {
        div().child(title).child(label)  // no borrow of self here
    }))
}
```

For simple scalar fields, inline cloning is fine. For label strings from `SessionStatus::label()`, the method returns `&'static str` — but the return type must be declared as `&'static str` (not `&str`) for the borrow checker to allow it in a `'static` context.

## Why not use lifetime annotations?

The compiler suggests annotating the return as `impl IntoElement + 'a`. This would work if GPUI's rendering system accepted non-`'static` elements, but it does not — elements are stored across frames and must outlive the render call.

## Consequence

Render helpers incur small allocations (a `Vec` per frame for list views). This is acceptable given GPUI's GPU-accelerated rendering model; the allocation cost is negligible compared to draw call overhead.

## Pattern to follow

When adding a new render helper that iterates over a `Vec` field:

1. Collect the needed fields into an owned `Vec<(owned, types, only)>` at the top of the helper.
2. Call `.into_iter()` on the owned vec — not `.iter()` on `self.field`.
3. Ensure any string labels come from `&'static str` returns (declare the return type explicitly on `label()` methods).
