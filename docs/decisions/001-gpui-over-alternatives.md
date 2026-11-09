# ADR-001 — Use GPUI as the UI framework

**Date:** November 2026  
**Status:** Accepted

## Context

Deck needs a native macOS UI framework. The main candidates were:

| Framework | Language | Rendering | Notes |
|-----------|----------|-----------|-------|
| GPUI | Rust | Metal GPU | Powers Zed; designed for high-performance dev tools |
| Tauri | Rust + WebView | WebKit | Mature; large ecosystem; web tech for UI |
| SwiftUI | Swift | Core Animation | First-class macOS; requires Swift/ObjC interop from Rust |
| egui | Rust | wgpu/Metal | Immediate-mode; simpler but less native feel |

## Decision

Use **GPUI 0.2.2**.

## Rationale

1. **The experiment is the point.** The primary objective is to evaluate GPUI's suitability for AI-native developer tooling — the application is the vehicle, not the goal.
2. **GPU-accelerated by design.** GPUI's Metal renderer is built for sustained text streaming and large event histories, which are core performance requirements.
3. **Retained-mode with explicit entities.** GPUI's entity model maps cleanly to the session/event domain model. Each session can be a GPUI entity.
4. **Full Rust.** No JavaScript, no WebView, no FFI boundary — the entire application is one language.

## Trade-offs accepted

- **Smaller ecosystem** than Tauri or egui. Some standard UI components (rich text, file picker, docking) must be built from scratch.
- **Pinned to 0.2.2.** GPUI is not semver-stable. We pin to a known-good version and accept the risk of API churn when upgrading.
- **macOS-only for now.** GPUI supports Linux experimentally but Windows support is limited. Cross-platform is explicitly deferred.
- **Metal Toolchain dependency.** An extra install step for contributors (see `AGENTS.md`).

## Alternatives not chosen

- **Tauri:** Would have accelerated UI development but hidden GPUI's characteristics — the entire point of the experiment.
- **SwiftUI:** Requires a Swift bridge. All domain logic would live in Rust anyway, making SwiftUI a thin wrapper with significant FFI overhead.
