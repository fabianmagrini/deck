use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::OffsetDateTime;
use uuid::Uuid;

/// A timestamped, sequenced event emitted by an agent session.
///
/// Events form an ordered log for a session. The `sequence` field is
/// monotonically increasing within a session and is used to replay or
/// display events in the correct order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbenchEvent {
    /// Globally unique event identifier.
    pub id: Uuid,
    /// Session this event belongs to.
    pub session_id: Uuid,
    /// Monotonically increasing counter within the session.
    pub sequence: u64,
    pub timestamp: OffsetDateTime,
    /// Payload describing what happened.
    pub kind: WorkbenchEventKind,
}

impl WorkbenchEvent {
    /// Construct a new event with the current UTC timestamp and a fresh UUID.
    pub fn new(session_id: Uuid, sequence: u64, kind: WorkbenchEventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            sequence,
            timestamp: OffsetDateTime::now_utc(),
            kind,
        }
    }
}

/// Discriminated payload for a [`WorkbenchEvent`].
///
/// Variants follow the lifecycle of a prompt: session start → streaming
/// message chunks → tool calls → optional approval gate → completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkbenchEventKind {
    /// The agent session has started processing.
    SessionStarted,
    /// A chunk of streamed assistant text.
    MessageDelta { text: String },
    /// The agent has finished producing the current message.
    MessageCompleted,
    /// An agent tool invocation has begun.
    ToolStarted { id: String, name: String },
    /// Incremental output from a running tool.
    ToolUpdated { id: String, output: String },
    /// A tool has finished. `success` indicates exit status.
    ToolCompleted { id: String, success: bool },
    /// The agent requests user approval before proceeding.
    ApprovalRequested {
        request_id: Uuid,
        description: String,
        /// Filesystem paths or other resources the action will touch.
        resources: Vec<String>,
    },
    /// The user has resolved a pending approval.
    ApprovalResolved { request_id: Uuid, approved: bool },
    /// The agent reports that it has modified files on disk.
    FilesChanged { paths: Vec<PathBuf> },
    /// The agent has finished processing the prompt.
    PromptCompleted,
    /// The session encountered an unrecoverable error.
    SessionFailed { message: String },
}

impl WorkbenchEventKind {
    /// Short human-readable label for logging and UI display.
    pub fn label(&self) -> &str {
        match self {
            Self::SessionStarted => "Session started",
            Self::MessageDelta { .. } => "Message delta",
            Self::MessageCompleted => "Message completed",
            Self::ToolStarted { .. } => "Tool started",
            Self::ToolUpdated { .. } => "Tool output",
            Self::ToolCompleted { .. } => "Tool completed",
            Self::ApprovalRequested { .. } => "Approval requested",
            Self::ApprovalResolved { .. } => "Approval resolved",
            Self::FilesChanged { .. } => "Files changed",
            Self::PromptCompleted => "Prompt completed",
            Self::SessionFailed { .. } => "Session failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialization_roundtrip() {
        let session_id = Uuid::new_v4();
        let event = WorkbenchEvent::new(
            session_id,
            1,
            WorkbenchEventKind::MessageDelta {
                text: "Hello, world!".to_string(),
            },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WorkbenchEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event.session_id, deserialized.session_id);
        assert_eq!(event.sequence, deserialized.sequence);
    }

    #[test]
    fn approval_event_roundtrip() {
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let event = WorkbenchEvent::new(
            session_id,
            5,
            WorkbenchEventKind::ApprovalRequested {
                request_id,
                description: "Execute: cargo test".to_string(),
                resources: vec!["src/".to_string()],
            },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WorkbenchEvent = serde_json::from_str(&json).unwrap();
        if let WorkbenchEventKind::ApprovalRequested {
            request_id: id, ..
        } = deserialized.kind
        {
            assert_eq!(id, request_id);
        } else {
            panic!("Wrong event kind");
        }
    }

    #[test]
    fn event_sequence_is_preserved() {
        let session_id = Uuid::new_v4();
        let event = WorkbenchEvent::new(session_id, 42, WorkbenchEventKind::PromptCompleted);
        assert_eq!(event.sequence, 42);
        assert_eq!(event.session_id, session_id);
    }

    #[test]
    fn all_kind_labels_are_non_empty() {
        let kinds = [
            WorkbenchEventKind::SessionStarted,
            WorkbenchEventKind::MessageDelta { text: "x".into() },
            WorkbenchEventKind::MessageCompleted,
            WorkbenchEventKind::ToolStarted { id: "1".into(), name: "read_file".into() },
            WorkbenchEventKind::ToolUpdated { id: "1".into(), output: "ok".into() },
            WorkbenchEventKind::ToolCompleted { id: "1".into(), success: true },
            WorkbenchEventKind::ApprovalRequested {
                request_id: Uuid::new_v4(),
                description: "desc".into(),
                resources: vec![],
            },
            WorkbenchEventKind::ApprovalResolved { request_id: Uuid::new_v4(), approved: true },
            WorkbenchEventKind::FilesChanged { paths: vec![] },
            WorkbenchEventKind::PromptCompleted,
            WorkbenchEventKind::SessionFailed { message: "oops".into() },
        ];
        for kind in &kinds {
            assert!(!kind.label().is_empty(), "{:?} label is empty", kind);
        }
    }

    #[test]
    fn files_changed_paths_roundtrip() {
        let session_id = Uuid::new_v4();
        let paths = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("Cargo.toml"),
        ];
        let event = WorkbenchEvent::new(
            session_id,
            3,
            WorkbenchEventKind::FilesChanged { paths: paths.clone() },
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WorkbenchEvent = serde_json::from_str(&json).unwrap();
        if let WorkbenchEventKind::FilesChanged { paths: p } = deserialized.kind {
            assert_eq!(p, paths);
        } else {
            panic!("Wrong event kind");
        }
    }
}
