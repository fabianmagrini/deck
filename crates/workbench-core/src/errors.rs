use thiserror::Error;
use uuid::Uuid;

/// Top-level error type for all Deck workbench operations.
///
/// Variants map to distinct failure categories so callers can decide
/// whether to surface a message to the user, retry, or escalate.
#[derive(Debug, Error)]
pub enum WorkbenchError {
    /// An agent adapter returned an error message.
    #[error("Agent error: {0}")]
    Agent(String),

    /// A session ID was referenced that does not exist.
    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    /// [`AgentSession::transition`][crate::AgentSession::transition] was called
    /// with an invalid state pair.
    #[error("Invalid session state transition: {0}")]
    InvalidTransition(String),

    /// The workspace root directory could not be found on disk.
    #[error("Workspace not found at path: {0}")]
    WorkspaceNotFound(String),

    /// An operation was refused due to insufficient permissions.
    #[error("Permission denied for operation: {0}")]
    PermissionDenied(String),

    /// A filesystem operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialisation or deserialisation failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
