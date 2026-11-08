//! Core domain types for the Deck AI agent workbench.
//!
//! This crate is dependency-free of any UI framework and can be used by all
//! other crates in the workspace. It defines:
//!
//! - **[`Workspace`]** — a local directory being supervised by Deck.
//! - **[`AgentSession`]** / **[`SessionStatus`]** — session lifecycle state machine.
//! - **[`WorkbenchEvent`]** / **[`WorkbenchEventKind`]** — ordered event log emitted by agents.
//! - **[`WorkbenchError`]** — top-level error enum.
//! - **[`AgentConfig`]** — static configuration for a registered agent adapter.

pub mod errors;
pub mod events;
pub mod models;

pub use errors::WorkbenchError;
pub use events::{WorkbenchEvent, WorkbenchEventKind};
pub use models::{AgentConfig, AgentSession, SessionStatus, Workspace};
