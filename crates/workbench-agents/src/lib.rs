//! Agent adapter layer for the Deck workbench.
//!
//! This crate defines the [`AgentAdapter`] trait that every agent backend must
//! implement, plus a [`MockAgent`] that returns scripted event sequences for
//! use during Milestone 2 UI development.
//!
//! ## Adding a new adapter
//!
//! 1. Create a struct that implements [`AgentAdapter`].
//! 2. Register it in the application startup code in `workbench-app`.
//!
//! The synchronous trait signature is intentional for the MVP; async adapters
//! will be supported in a later milestone.

pub mod adapter;
pub mod mock;

pub use adapter::AgentAdapter;
pub use mock::MockAgent;
