use std::path::PathBuf;
use uuid::Uuid;
use workbench_core::{WorkbenchEvent, WorkbenchEventKind};

use crate::adapter::AgentAdapter;

/// Deterministic in-process mock that implements [`AgentAdapter`] without
/// spawning any external process.
///
/// `MockAgent` returns a scripted sequence of events whose shape varies based
/// on keywords in the prompt (`"refactor"`, `"test"`, `"build"`, etc.). It is
/// used by the UI during Milestone 2 to exercise the full event pipeline
/// before real agent integrations are wired up.
pub struct MockAgent;

impl MockAgent {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for MockAgent {
    fn adapter_id(&self) -> &str {
        "mock"
    }

    fn display_name(&self) -> &str {
        "Mock Agent"
    }

    fn run_prompt(&self, session_id: Uuid, prompt: &str) -> Vec<WorkbenchEvent> {
        let mut seq = 0u64;
        let mut events = Vec::new();

        let mut next_seq = || {
            let s = seq;
            seq += 1;
            s
        };

        let tool_id = Uuid::new_v4().to_string();
        let approval_id = Uuid::new_v4();

        // Determine script based on prompt content
        let response_text = generate_response_text(prompt);
        let (tool_name, tool_output) = generate_tool_activity(prompt);

        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::SessionStarted,
        ));

        // Stream response in chunks
        for chunk in chunk_text(&response_text, 60) {
            events.push(WorkbenchEvent::new(
                session_id,
                next_seq(),
                WorkbenchEventKind::MessageDelta { text: chunk },
            ));
        }

        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::MessageCompleted,
        ));

        // Simulate tool activity
        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::ToolStarted {
                id: tool_id.clone(),
                name: tool_name.clone(),
            },
        ));

        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::ToolUpdated {
                id: tool_id.clone(),
                output: tool_output,
            },
        ));

        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::ToolCompleted {
                id: tool_id.clone(),
                success: true,
            },
        ));

        // Simulate file changes
        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::FilesChanged {
                paths: vec![
                    PathBuf::from("src/main.rs"),
                    PathBuf::from("src/lib.rs"),
                ],
            },
        ));

        // Simulate approval request (requires user to call resolve_approval)
        events.push(WorkbenchEvent::new(
            session_id,
            next_seq(),
            WorkbenchEventKind::ApprovalRequested {
                request_id: approval_id,
                description: format!("Execute: {}", tool_name),
                resources: vec!["src/".to_string()],
            },
        ));

        events
    }

    fn resolve_approval(
        &self,
        session_id: Uuid,
        request_id: Uuid,
        approved: bool,
    ) -> Vec<WorkbenchEvent> {
        let mut seq = 100u64; // continuation sequence
        let tool_id = Uuid::new_v4().to_string();

        let mut events = Vec::new();

        events.push(WorkbenchEvent::new(
            session_id,
            seq,
            WorkbenchEventKind::ApprovalResolved { request_id, approved },
        ));
        seq += 1;

        if approved {
            events.push(WorkbenchEvent::new(
                session_id,
                seq,
                WorkbenchEventKind::ToolStarted {
                    id: tool_id.clone(),
                    name: "cargo test".to_string(),
                },
            ));
            seq += 1;

            events.push(WorkbenchEvent::new(
                session_id,
                seq,
                WorkbenchEventKind::ToolUpdated {
                    id: tool_id.clone(),
                    output: "running 18 tests\ntest result: ok. 18 passed; 0 failed".to_string(),
                },
            ));
            seq += 1;

            events.push(WorkbenchEvent::new(
                session_id,
                seq,
                WorkbenchEventKind::ToolCompleted {
                    id: tool_id.clone(),
                    success: true,
                },
            ));
            seq += 1;
        }

        events.push(WorkbenchEvent::new(
            session_id,
            seq,
            WorkbenchEventKind::PromptCompleted,
        ));

        events
    }
}

fn generate_response_text(prompt: &str) -> String {
    if prompt.to_lowercase().contains("refactor") || prompt.to_lowercase().contains("parser") {
        "I'll refactor the command parser to improve testability. First, let me inspect the current implementation, then extract the command handling logic into separate, testable units.\n\nI'll read the relevant source files and propose the restructured design.".to_string()
    } else if prompt.to_lowercase().contains("test") {
        "I'll add comprehensive unit tests to improve coverage. Let me first understand the existing code structure and identify the key behaviours that need testing.".to_string()
    } else if prompt.to_lowercase().contains("fix") || prompt.to_lowercase().contains("bug") {
        "I'll investigate the issue and implement a fix. Let me start by reading the relevant files to understand the current behaviour and identify the root cause.".to_string()
    } else {
        format!(
            "I'll work on your request: \"{}\"\n\nLet me start by exploring the codebase to understand the current structure before making any changes.",
            prompt.chars().take(80).collect::<String>()
        )
    }
}

fn generate_tool_activity(prompt: &str) -> (String, String) {
    if prompt.to_lowercase().contains("test") {
        (
            "cargo test".to_string(),
            "running 18 tests\ntest result: ok. 18 passed; 0 failed; 0 ignored".to_string(),
        )
    } else if prompt.to_lowercase().contains("build") {
        (
            "cargo build".to_string(),
            "   Compiling workbench-core v0.1.0\n   Compiling workbench-app v0.1.0\n    Finished dev [unoptimized + debuginfo] target(s)".to_string(),
        )
    } else {
        (
            "read_file".to_string(),
            "Read 4 source files: src/main.rs, src/lib.rs, src/parser.rs, src/commands.rs".to_string(),
        )
    }
}

fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    chars
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_start_with_session_started() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "refactor the parser");
        assert!(!events.is_empty());
        assert!(matches!(events[0].kind, WorkbenchEventKind::SessionStarted));
    }

    #[test]
    fn all_events_belong_to_session() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "add some tests");
        for event in &events {
            assert_eq!(event.session_id, session_id, "Event belongs to wrong session");
        }
    }

    #[test]
    fn events_contain_approval_request() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "refactor the parser");
        let has_approval = events.iter().any(|e| {
            matches!(e.kind, WorkbenchEventKind::ApprovalRequested { .. })
        });
        assert!(has_approval, "Expected ApprovalRequested in event log");
    }

    #[test]
    fn events_contain_tool_lifecycle() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "read some files");
        let has_started = events.iter().any(|e| matches!(e.kind, WorkbenchEventKind::ToolStarted { .. }));
        let has_completed = events.iter().any(|e| matches!(e.kind, WorkbenchEventKind::ToolCompleted { .. }));
        assert!(has_started, "Expected ToolStarted");
        assert!(has_completed, "Expected ToolCompleted");
    }

    #[test]
    fn events_contain_files_changed() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "fix the bug");
        let has_files = events.iter().any(|e| matches!(e.kind, WorkbenchEventKind::FilesChanged { .. }));
        assert!(has_files, "Expected FilesChanged event");
    }

    #[test]
    fn sequence_numbers_are_monotonically_increasing() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "build the project");
        let sequences: Vec<u64> = events.iter().map(|e| e.sequence).collect();
        let mut sorted = sequences.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sequences, sorted, "Sequence numbers must be strictly increasing");
    }

    #[test]
    fn test_prompt_uses_cargo_test_tool() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "run the tests");
        let tool_name = events.iter().find_map(|e| {
            if let WorkbenchEventKind::ToolStarted { name, .. } = &e.kind {
                Some(name.clone())
            } else {
                None
            }
        });
        assert_eq!(tool_name.as_deref(), Some("cargo test"));
    }

    #[test]
    fn build_prompt_uses_cargo_build_tool() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let events = agent.run_prompt(session_id, "build the project");
        let tool_name = events.iter().find_map(|e| {
            if let WorkbenchEventKind::ToolStarted { name, .. } = &e.kind {
                Some(name.clone())
            } else {
                None
            }
        });
        assert_eq!(tool_name.as_deref(), Some("cargo build"));
    }

    #[test]
    fn resolve_approval_approved_ends_with_prompt_completed() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let events = agent.resolve_approval(session_id, request_id, true);
        assert!(!events.is_empty());
        assert!(matches!(
            events.last().unwrap().kind,
            WorkbenchEventKind::PromptCompleted
        ));
    }

    #[test]
    fn resolve_approval_denied_ends_with_prompt_completed() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let events = agent.resolve_approval(session_id, request_id, false);
        assert!(matches!(
            events.last().unwrap().kind,
            WorkbenchEventKind::PromptCompleted
        ));
    }

    #[test]
    fn resolve_approval_approved_includes_tool_execution() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let events = agent.resolve_approval(session_id, request_id, true);
        let has_tool = events.iter().any(|e| matches!(e.kind, WorkbenchEventKind::ToolStarted { .. }));
        assert!(has_tool, "Approved path should include tool execution");
    }

    #[test]
    fn resolve_approval_denied_skips_tool_execution() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let events = agent.resolve_approval(session_id, request_id, false);
        let has_tool = events.iter().any(|e| matches!(e.kind, WorkbenchEventKind::ToolStarted { .. }));
        assert!(!has_tool, "Denied path should not include tool execution");
    }

    #[test]
    fn approval_resolved_event_carries_correct_id() {
        let agent = MockAgent::new();
        let session_id = Uuid::new_v4();
        let request_id = Uuid::new_v4();
        let events = agent.resolve_approval(session_id, request_id, true);
        let resolved_id = events.iter().find_map(|e| {
            if let WorkbenchEventKind::ApprovalResolved { request_id: id, .. } = &e.kind {
                Some(*id)
            } else {
                None
            }
        });
        assert_eq!(resolved_id, Some(request_id));
    }

    #[test]
    fn adapter_metadata() {
        let agent = MockAgent::new();
        assert_eq!(agent.adapter_id(), "mock");
        assert!(!agent.display_name().is_empty());
    }
}
