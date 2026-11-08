use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::OffsetDateTime;
use uuid::Uuid;

/// A workspace represents a local directory being supervised by Deck.
///
/// Each workspace has a unique ID and tracks when it was created and last
/// opened, allowing the UI to restore recent workspaces across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Stable unique identifier for this workspace.
    pub id: Uuid,
    /// Human-readable name, typically the directory basename.
    pub name: String,
    /// Absolute path to the workspace root on disk.
    pub root_path: PathBuf,
    pub created_at: OffsetDateTime,
    pub last_opened_at: OffsetDateTime,
}

impl Workspace {
    /// Create a new workspace rooted at `root_path` with the given display `name`.
    pub fn new(name: impl Into<String>, root_path: PathBuf) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            root_path,
            created_at: now,
            last_opened_at: now,
        }
    }
}

/// Lifecycle state of an [`AgentSession`].
///
/// The state machine enforces which transitions are valid via
/// [`SessionStatus::can_transition_to`]. Terminal states
/// (`Completed`, `Failed`, `Disconnected`) accept no further transitions
/// except to `Disconnected`.
///
/// ```text
/// Created → Initializing → Ready → Running ──→ Completed
///                       ↘ Failed        ↓
///                                  AwaitingApproval ↔ Running
///                                        ↓
///                                   Cancelling → Completed | Failed
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session object created but agent not yet launched.
    Created,
    /// Agent process is starting up.
    Initializing,
    /// Agent is ready and waiting for a prompt.
    Ready,
    /// Agent is actively processing a prompt.
    Running,
    /// Agent has paused and is waiting for user approval.
    AwaitingApproval,
    /// A cancellation has been requested; agent is winding down.
    Cancelling,
    /// Session ended successfully.
    Completed,
    /// Session ended with an unrecoverable error.
    Failed,
    /// Session lost its connection to the agent process.
    Disconnected,
}

impl SessionStatus {
    /// Returns `true` if no further valid transitions exist (except `Disconnected`).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            SessionStatus::Completed | SessionStatus::Failed | SessionStatus::Disconnected
        )
    }

    /// Returns `true` when transitioning from `self` to `next` is a valid move
    /// in the session state machine.
    pub fn can_transition_to(&self, next: &SessionStatus) -> bool {
        use SessionStatus::*;
        matches!(
            (self, next),
            (Created, Initializing)
                | (Initializing, Ready)
                | (Initializing, Failed)
                | (Ready, Running)
                | (Running, AwaitingApproval)
                | (Running, Completed)
                | (Running, Failed)
                | (Running, Cancelling)
                | (AwaitingApproval, Running)
                | (AwaitingApproval, Cancelling)
                | (Cancelling, Completed)
                | (Cancelling, Failed)
                | (_, Disconnected)
        )
    }

    /// Short human-readable label suitable for display in the UI sidebar.
    pub fn label(&self) -> &'static str {
        match self {
            SessionStatus::Created => "Created",
            SessionStatus::Initializing => "Initializing",
            SessionStatus::Ready => "Ready",
            SessionStatus::Running => "Running",
            SessionStatus::AwaitingApproval => "Awaiting Approval",
            SessionStatus::Cancelling => "Cancelling",
            SessionStatus::Completed => "Completed",
            SessionStatus::Failed => "Failed",
            SessionStatus::Disconnected => "Disconnected",
        }
    }
}

/// A running (or historical) agent session within a workspace.
///
/// Each `AgentSession` has its own lifecycle tracked by [`SessionStatus`].
/// Call [`AgentSession::transition`] to advance the state machine; invalid
/// transitions return an `Err` without mutating the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// Stable unique identifier for this session.
    pub id: Uuid,
    /// The workspace this session belongs to.
    pub workspace_id: Uuid,
    /// Identifier of the [`AgentAdapter`][crate] implementation driving this session.
    pub adapter_id: String,
    /// Human-readable title shown in the sidebar.
    pub title: String,
    /// Current lifecycle state.
    pub status: SessionStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl AgentSession {
    /// Create a new session in the [`SessionStatus::Created`] state.
    pub fn new(workspace_id: Uuid, adapter_id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            adapter_id: adapter_id.into(),
            title: title.into(),
            status: SessionStatus::Created,
            created_at: now,
            updated_at: now,
        }
    }

    /// Attempt to advance the session to `next`.
    ///
    /// Returns `Err` if the transition is not permitted by the state machine.
    /// On success, `updated_at` is refreshed to the current UTC time.
    pub fn transition(&mut self, next: SessionStatus) -> Result<(), String> {
        if self.status.can_transition_to(&next) {
            self.status = next;
            self.updated_at = OffsetDateTime::now_utc();
            Ok(())
        } else {
            Err(format!(
                "Invalid transition from {:?} to {:?}",
                self.status, next
            ))
        }
    }
}

/// Static configuration for a registered agent adapter.
///
/// These are discovered at startup and represent available agent integrations
/// (e.g. Claude Code, a custom CLI). The `enabled` flag gates whether the
/// adapter is offered to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Matches [`AgentAdapter::adapter_id`][crate].
    pub id: String,
    /// Display name shown in the UI.
    pub name: String,
    /// Path to the agent executable.
    pub executable: PathBuf,
    /// Additional arguments passed to the executable.
    pub arguments: Vec<String>,
    /// Whether this adapter is offered to the user.
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_status_happy_path() {
        let mut session = AgentSession::new(Uuid::new_v4(), "mock", "Test Session");
        assert_eq!(session.status, SessionStatus::Created);

        session.transition(SessionStatus::Initializing).unwrap();
        assert_eq!(session.status, SessionStatus::Initializing);

        session.transition(SessionStatus::Ready).unwrap();
        assert_eq!(session.status, SessionStatus::Ready);

        session.transition(SessionStatus::Running).unwrap();
        assert_eq!(session.status, SessionStatus::Running);

        session.transition(SessionStatus::Completed).unwrap();
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.status.is_terminal());
    }

    #[test]
    fn invalid_transition_fails() {
        let mut session = AgentSession::new(Uuid::new_v4(), "mock", "Test Session");
        let result = session.transition(SessionStatus::Running);
        assert!(result.is_err());
        // Status unchanged on failure
        assert_eq!(session.status, SessionStatus::Created);
    }

    #[test]
    fn approval_cycle_transitions() {
        let mut session = AgentSession::new(Uuid::new_v4(), "mock", "Approval Session");
        session.transition(SessionStatus::Initializing).unwrap();
        session.transition(SessionStatus::Ready).unwrap();
        session.transition(SessionStatus::Running).unwrap();
        session.transition(SessionStatus::AwaitingApproval).unwrap();
        // Approve → back to Running
        session.transition(SessionStatus::Running).unwrap();
        assert_eq!(session.status, SessionStatus::Running);
    }

    #[test]
    fn cancellation_path() {
        let mut session = AgentSession::new(Uuid::new_v4(), "mock", "Cancel Session");
        session.transition(SessionStatus::Initializing).unwrap();
        session.transition(SessionStatus::Ready).unwrap();
        session.transition(SessionStatus::Running).unwrap();
        session.transition(SessionStatus::Cancelling).unwrap();
        session.transition(SessionStatus::Completed).unwrap();
        assert!(session.status.is_terminal());
    }

    #[test]
    fn disconnected_is_always_valid() {
        // Any state can transition to Disconnected
        for status in [
            SessionStatus::Created,
            SessionStatus::Initializing,
            SessionStatus::Ready,
            SessionStatus::Running,
            SessionStatus::AwaitingApproval,
            SessionStatus::Cancelling,
        ] {
            assert!(
                status.can_transition_to(&SessionStatus::Disconnected),
                "{:?} should be able to transition to Disconnected",
                status
            );
        }
    }

    #[test]
    fn terminal_states_are_complete() {
        assert!(SessionStatus::Completed.is_terminal());
        assert!(SessionStatus::Failed.is_terminal());
        assert!(SessionStatus::Disconnected.is_terminal());
        assert!(!SessionStatus::Running.is_terminal());
        assert!(!SessionStatus::AwaitingApproval.is_terminal());
    }

    #[test]
    fn labels_are_non_empty() {
        for status in [
            SessionStatus::Created,
            SessionStatus::Initializing,
            SessionStatus::Ready,
            SessionStatus::Running,
            SessionStatus::AwaitingApproval,
            SessionStatus::Cancelling,
            SessionStatus::Completed,
            SessionStatus::Failed,
            SessionStatus::Disconnected,
        ] {
            assert!(!status.label().is_empty());
        }
    }

    #[test]
    fn workspace_new_sets_timestamps() {
        let ws = Workspace::new("my-project", PathBuf::from("/tmp/my-project"));
        assert_eq!(ws.name, "my-project");
        assert_eq!(ws.root_path, PathBuf::from("/tmp/my-project"));
        assert_eq!(ws.created_at, ws.last_opened_at);
    }
}
