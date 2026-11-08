use uuid::Uuid;
use workbench_core::WorkbenchEvent;

/// Synchronous MVP interface that every agent integration must implement.
///
/// An adapter translates between Deck's domain model and a specific agent
/// backend (e.g. Claude Code CLI, a custom REST API). The synchronous
/// signature is intentional for Milestone 2: the UI dispatches adapter
/// calls on a background thread and collects the resulting event log.
///
/// # Implementing an adapter
///
/// ```ignore
/// struct MyAgent;
///
/// impl AgentAdapter for MyAgent {
///     fn adapter_id(&self) -> &str { "my-agent" }
///     fn display_name(&self) -> &str { "My Agent" }
///
///     fn run_prompt(&self, session_id: Uuid, prompt: &str) -> Vec<WorkbenchEvent> {
///         // Drive your backend here, return an ordered event log.
///         vec![]
///     }
///
///     fn resolve_approval(&self, session_id: Uuid, request_id: Uuid, approved: bool)
///         -> Vec<WorkbenchEvent>
///     {
///         vec![]
///     }
/// }
/// ```
pub trait AgentAdapter: Send + Sync {
    /// Stable machine-readable identifier for this adapter (e.g. `"mock"`, `"claude-code"`).
    fn adapter_id(&self) -> &str;

    /// Human-readable name shown in the UI adapter list.
    fn display_name(&self) -> &str;

    /// Generate a scripted sequence of events for a prompt.
    ///
    /// Returns the complete ordered event log that will result from executing
    /// `prompt` within `session_id`. The first event should be
    /// [`WorkbenchEventKind::SessionStarted`][workbench_core::WorkbenchEventKind::SessionStarted]
    /// and the last should be `PromptCompleted` or `SessionFailed`.
    fn run_prompt(&self, session_id: Uuid, prompt: &str) -> Vec<WorkbenchEvent>;

    /// Resolve a pending approval request.
    ///
    /// Called after the UI presents an `ApprovalRequested` event to the user.
    /// Set `approved = true` to allow the pending action, `false` to deny it.
    /// Returns any subsequent events (e.g. tool execution results, `PromptCompleted`).
    fn resolve_approval(
        &self,
        session_id: Uuid,
        request_id: Uuid,
        approved: bool,
    ) -> Vec<WorkbenchEvent>;
}
