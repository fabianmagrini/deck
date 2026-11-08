use std::sync::Arc;

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, Hsla, MouseButton, MouseUpEvent,
    Render, Window, div, prelude::*, px,
};
use uuid::Uuid;

use workbench_agents::{AgentAdapter, MockAgent};
use workbench_core::{SessionStatus, WorkbenchEventKind};

use crate::actions::{ApproveRequest, CancelSession, DenyRequest, NewSession, SendPrompt, Submit};
use crate::theme;
use crate::views::input::TextInput;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: Uuid,
    pub title: String,
    pub status: SessionStatus,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Agent,
}

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ToolActivity {
    pub id: String,
    pub name: String,
    pub output: Option<String>,
    pub completed: bool,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct TimelineEntry {
    pub description: String,
    pub is_tool: bool,
    pub is_approval: bool,
}

#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub request_id: Uuid,
    pub description: String,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
}

pub struct Workbench {
    // Core state
    workspace_name: String,
    sessions: Vec<SessionSummary>,
    active_session_idx: Option<usize>,

    // Conversation
    messages: Vec<ConversationMessage>,
    current_agent_text: String,
    tool_activities: Vec<ToolActivity>,

    // Timeline
    timeline: Vec<TimelineEntry>,

    // Inspector
    pending_approval: Option<PendingApproval>,
    changed_files: Vec<ChangedFile>,

    // Output
    output_lines: Vec<String>,

    // Input
    input: Entity<TextInput>,
    pub focus_handle: FocusHandle,

    // Agent
    agent: Arc<MockAgent>,
    active_session_id: Option<Uuid>,
}

impl Workbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx, "Message agent..."));
        let focus_handle = cx.focus_handle();

        // Demo sessions
        let sessions = vec![
            SessionSummary {
                id: Uuid::new_v4(),
                title: "Refactor CLI".to_string(),
                status: SessionStatus::Completed,
            },
            SessionSummary {
                id: Uuid::new_v4(),
                title: "Add tests".to_string(),
                status: SessionStatus::Completed,
            },
        ];

        Self {
            workspace_name: "demo-project".to_string(),
            sessions,
            active_session_idx: None,
            messages: Vec::new(),
            current_agent_text: String::new(),
            tool_activities: Vec::new(),
            timeline: Vec::new(),
            pending_approval: None,
            changed_files: Vec::new(),
            output_lines: Vec::new(),
            input,
            focus_handle,
            agent: Arc::new(MockAgent::new()),
            active_session_id: None,
        }
    }

    fn new_session(&mut self, _: &NewSession, _window: &mut Window, cx: &mut Context<Self>) {
        let session_id = Uuid::new_v4();
        let title = format!("Session {}", self.sessions.len() + 1);
        self.sessions.push(SessionSummary {
            id: session_id,
            title: title.clone(),
            status: SessionStatus::Ready,
        });
        self.active_session_idx = Some(self.sessions.len() - 1);
        self.active_session_id = Some(session_id);
        self.messages.clear();
        self.tool_activities.clear();
        self.timeline.clear();
        self.pending_approval = None;
        self.changed_files.clear();
        self.output_lines.clear();
        self.current_agent_text.clear();
        self.timeline.push(TimelineEntry {
            description: format!("Session started: {}", title),
            is_tool: false,
            is_approval: false,
        });
        cx.notify();
    }

    fn send_prompt(&mut self, _: &SendPrompt, window: &mut Window, cx: &mut Context<Self>) {
        self.do_send(window, cx);
    }

    fn on_input_submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        self.do_send(window, cx);
    }

    fn do_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = self.input.read(cx).get_text().to_string();
        if prompt.trim().is_empty() {
            return;
        }

        // Ensure we have an active session
        if self.active_session_idx.is_none() {
            let session_id = Uuid::new_v4();
            let title = "New Session".to_string();
            self.sessions.push(SessionSummary {
                id: session_id,
                title: title.clone(),
                status: SessionStatus::Running,
            });
            self.active_session_idx = Some(self.sessions.len() - 1);
            self.active_session_id = Some(session_id);
        } else if let Some(idx) = self.active_session_idx {
            self.sessions[idx].status = SessionStatus::Running;
        }

        let session_id = self.active_session_id.unwrap_or_else(Uuid::new_v4);

        // Add user message
        self.messages.push(ConversationMessage {
            role: MessageRole::User,
            text: prompt.clone(),
        });

        // Clear previous agent response
        self.current_agent_text.clear();
        self.tool_activities.clear();

        // Run mock agent
        let events = self.agent.run_prompt(session_id, &prompt);

        for event in events {
            match event.kind {
                WorkbenchEventKind::SessionStarted => {
                    self.timeline.push(TimelineEntry {
                        description: "Agent initialised".to_string(),
                        is_tool: false,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::MessageDelta { text } => {
                    self.current_agent_text.push_str(&text);
                }
                WorkbenchEventKind::MessageCompleted => {
                    if !self.current_agent_text.is_empty() {
                        let text = std::mem::take(&mut self.current_agent_text);
                        self.messages.push(ConversationMessage {
                            role: MessageRole::Agent,
                            text,
                        });
                    }
                    self.timeline.push(TimelineEntry {
                        description: "Response completed".to_string(),
                        is_tool: false,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::ToolStarted { id, name } => {
                    self.tool_activities.push(ToolActivity {
                        id: id.clone(),
                        name: name.clone(),
                        output: None,
                        completed: false,
                        success: false,
                    });
                    self.timeline.push(TimelineEntry {
                        description: format!("Tool started: {}", name),
                        is_tool: true,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::ToolUpdated { id, output } => {
                    if let Some(tool) = self.tool_activities.iter_mut().find(|t| t.id == id) {
                        tool.output = Some(output.clone());
                    }
                    self.output_lines.push(output);
                }
                WorkbenchEventKind::ToolCompleted { id, success } => {
                    if let Some(tool) = self.tool_activities.iter_mut().find(|t| t.id == id) {
                        tool.completed = true;
                        tool.success = success;
                    }
                    self.timeline.push(TimelineEntry {
                        description: format!(
                            "Tool completed: {}",
                            if success { "success" } else { "failed" }
                        ),
                        is_tool: true,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::FilesChanged { paths } => {
                    for path in paths {
                        self.changed_files.push(ChangedFile {
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                    self.timeline.push(TimelineEntry {
                        description: format!(
                            "Files changed: {}",
                            self.changed_files.len()
                        ),
                        is_tool: false,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::ApprovalRequested {
                    request_id,
                    description,
                    resources,
                } => {
                    self.pending_approval = Some(PendingApproval {
                        request_id,
                        description: description.clone(),
                        resources,
                    });
                    if let Some(idx) = self.active_session_idx {
                        self.sessions[idx].status = SessionStatus::AwaitingApproval;
                    }
                    self.timeline.push(TimelineEntry {
                        description: format!("Approval requested: {}", description),
                        is_tool: false,
                        is_approval: true,
                    });
                }
                _ => {}
            }
        }

        // Clear the input
        self.input.update(cx, |input, cx| {
            input.clear(window, cx);
        });

        cx.notify();
    }

    fn approve_request(&mut self, _: &ApproveRequest, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(approval) = self.pending_approval.take() {
            let session_id = self.active_session_id.unwrap_or_else(Uuid::new_v4);
            let events = self.agent.resolve_approval(session_id, approval.request_id, true);
            self.apply_post_approval_events(events);
            if let Some(idx) = self.active_session_idx {
                self.sessions[idx].status = SessionStatus::Completed;
            }
        }
        cx.notify();
    }

    fn deny_request(&mut self, _: &DenyRequest, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(approval) = self.pending_approval.take() {
            let session_id = self.active_session_id.unwrap_or_else(Uuid::new_v4);
            let events = self.agent.resolve_approval(session_id, approval.request_id, false);
            self.apply_post_approval_events(events);
            if let Some(idx) = self.active_session_idx {
                self.sessions[idx].status = SessionStatus::Completed;
            }
        }
        cx.notify();
    }

    fn cancel_session(&mut self, _: &CancelSession, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(idx) = self.active_session_idx {
            self.sessions[idx].status = SessionStatus::Completed;
        }
        self.pending_approval = None;
        self.timeline.push(TimelineEntry {
            description: "Session cancelled".to_string(),
            is_tool: false,
            is_approval: false,
        });
        cx.notify();
    }

    fn apply_post_approval_events(&mut self, events: Vec<workbench_core::WorkbenchEvent>) {
        for event in events {
            match event.kind {
                WorkbenchEventKind::ApprovalResolved { approved, .. } => {
                    self.timeline.push(TimelineEntry {
                        description: format!(
                            "Approval {}",
                            if approved { "granted" } else { "denied" }
                        ),
                        is_tool: false,
                        is_approval: true,
                    });
                }
                WorkbenchEventKind::ToolStarted { id, name } => {
                    self.tool_activities.push(ToolActivity {
                        id,
                        name: name.clone(),
                        output: None,
                        completed: false,
                        success: false,
                    });
                    self.timeline.push(TimelineEntry {
                        description: format!("Started: {}", name),
                        is_tool: true,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::ToolUpdated { id, output } => {
                    if let Some(tool) = self.tool_activities.iter_mut().find(|t| t.id == id) {
                        tool.output = Some(output.clone());
                    }
                    self.output_lines.push(output);
                }
                WorkbenchEventKind::ToolCompleted { id, success } => {
                    if let Some(tool) = self.tool_activities.iter_mut().find(|t| t.id == id) {
                        tool.completed = true;
                        tool.success = success;
                    }
                    self.timeline.push(TimelineEntry {
                        description: format!(
                            "Tests {}",
                            if success { "passed" } else { "failed" }
                        ),
                        is_tool: true,
                        is_approval: false,
                    });
                }
                WorkbenchEventKind::PromptCompleted => {
                    self.timeline.push(TimelineEntry {
                        description: "Prompt completed".to_string(),
                        is_tool: false,
                        is_approval: false,
                    });
                }
                _ => {}
            }
        }
    }

    fn select_session(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.active_session_idx = Some(idx);
        self.active_session_id = Some(self.sessions[idx].id);
        cx.notify();
    }

    // ── Render helpers ───────────────────────────────────────────────────────

    fn render_left_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(px(220.))
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::bg_panel())
            .border_r_1()
            .border_color(theme::border())
            // Workspace section
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child("WORKSPACE"),
            )
            .child(
                div()
                    .px(px(12.))
                    .py(px(4.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme::accent_blue())
                            .child(format!(" {}", self.workspace_name)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .pl(px(16.))
                            .child("  src"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .pl(px(28.))
                            .child("main.rs"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .pl(px(28.))
                            .child("lib.rs"),
                    ),
            )
            // Divider
            .child(div().h(px(1.)).bg(theme::border()).my(px(8.)))
            // Sessions section
            .child(
                div()
                    .px(px(12.))
                    .py(px(4.))
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("SESSIONS"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::accent_blue())
                            .px(px(6.))
                            .py(px(2.))
                            .rounded(px(3.))
                            .cursor_pointer()
                            .hover(|s| s.bg(theme::bg_hover()))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, window, cx| {
                                    this.new_session(&NewSession, window, cx);
                                }),
                            )
                            .child("+ New"),
                    ),
            )
            .child({
                let session_items: Vec<(usize, String, Hsla, &'static str, bool)> = self
                    .sessions
                    .iter()
                    .enumerate()
                    .map(|(idx, session)| {
                        let status_color = match session.status {
                            SessionStatus::Running => theme::status_running(),
                            SessionStatus::AwaitingApproval => theme::status_waiting(),
                            SessionStatus::Failed | SessionStatus::Disconnected => {
                                theme::status_failed()
                            }
                            _ => theme::status_idle(),
                        };
                        (idx, session.title.clone(), status_color, session.status.label(), self.active_session_idx == Some(idx))
                    })
                    .collect();
                div()
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .px(px(8.))
                    .children(session_items.into_iter().map(|(idx, title, status_color, status_label, is_active)| {
                        div()
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(4.))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .cursor_pointer()
                            .bg(if is_active {
                                theme::bg_selected()
                            } else {
                                theme::bg_panel()
                            })
                            .hover(|s| {
                                s.bg(theme::bg_hover())
                            })
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseUpEvent, _window, cx| {
                                    this.select_session(idx, cx);
                                }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::text_primary())
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(status_color)
                                    .child(status_label),
                            )
                    }))
            })
    }

    fn render_conversation(&self, cx: &Context<Self>) -> impl IntoElement {
        let session_title = self
            .active_session_idx
            .and_then(|i| self.sessions.get(i))
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "No session selected".to_string());

        let session_status = self
            .active_session_idx
            .and_then(|i| self.sessions.get(i))
            .map(|s| s.status.label().to_owned())
            .unwrap_or_else(|| "—".to_owned());

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::bg_base())
            // Header
            .child(
                div()
                    .px(px(16.))
                    .py(px(12.))
                    .flex()
                    .justify_between()
                    .items_center()
                    .border_b_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .items_center()
                            .child(
                                div()
                                    .text_base()
                                    .text_color(theme::text_primary())
                                    .child(session_title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .px(px(6.))
                                    .py(px(2.))
                                    .rounded(px(3.))
                                    .bg(theme::bg_panel())
                                    .child(session_status),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::accent_red())
                            .px(px(8.))
                            .py(px(4.))
                            .rounded(px(3.))
                            .cursor_pointer()
                            .hover(|s| s.bg(theme::bg_hover()))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, window, cx| {
                                    this.cancel_session(&CancelSession, window, cx);
                                }),
                            )
                            .child("Cancel"),
                    ),
            )
            // Messages
            .child(
                div()
                    .id("messages-scroll")
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .px(px(16.))
                    .py(px(12.))
                    .overflow_y_scroll()
                    .children(self.messages.iter().map(|msg| {
                        let (bg, text_color, label) = match msg.role {
                            MessageRole::User => (
                                theme::bg_selected(),
                                theme::text_primary(),
                                "You",
                            ),
                            MessageRole::Agent => (
                                theme::bg_panel(),
                                theme::text_primary(),
                                "Agent",
                            ),
                        };
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .p(px(12.))
                            .rounded(px(6.))
                            .bg(bg)
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(label),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_color)
                                    .child(msg.text.clone()),
                            )
                    }))
                    // Tool activity blocks
                    .children(self.tool_activities.iter().map(|tool| {
                        let status_text = if !tool.completed {
                            "Running...".to_string()
                        } else if tool.success {
                            "✓ Completed".to_string()
                        } else {
                            "✗ Failed".to_string()
                        };
                        let status_color = if !tool.completed {
                            theme::accent_yellow()
                        } else if tool.success {
                            theme::accent_green()
                        } else {
                            theme::accent_red()
                        };
                        div()
                            .p(px(10.))
                            .rounded(px(4.))
                            .bg(theme::bg_panel())
                            .border_l_2()
                            .border_color(theme::accent_cyan())
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::accent_cyan())
                                            .child(format!(" {}", tool.name)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(status_color)
                                            .child(status_text),
                                    ),
                            )
                    })),
            )
            // Input area
            .child(
                div()
                    .border_t_1()
                    .border_color(theme::border())
                    .p(px(12.))
                    .flex()
                    .gap(px(8.))
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(theme::border())
                            .overflow_hidden()
                            .on_action(cx.listener(|this, action: &Submit, window, cx| {
                                this.on_input_submit(action, window, cx);
                            }))
                            .child(self.input.clone()),
                    )
                    .child(
                        div()
                            .px(px(14.))
                            .py(px(8.))
                            .rounded(px(6.))
                            .bg(theme::accent_blue())
                            .text_sm()
                            .text_color(theme::bg_base())
                            .cursor_pointer()
                            .hover(|s| s.bg(theme::bg_hover()))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseUpEvent, window, cx| {
                                    this.send_prompt(&SendPrompt, window, cx);
                                }),
                            )
                            .child("Send"),
                    ),
            )
    }

    fn render_right_panel(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .w(px(260.))
            .h_full()
            .flex()
            .flex_col()
            .bg(theme::bg_panel())
            .border_l_1()
            .border_color(theme::border())
            // Inspector header
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .text_xs()
                    .text_color(theme::text_muted())
                    .border_b_1()
                    .border_color(theme::border())
                    .child("INSPECTOR"),
            )
            // Changes section
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("CHANGES"),
                    )
                    .children(if self.changed_files.is_empty() {
                        vec![div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child("No changes")]
                    } else {
                        self.changed_files
                            .iter()
                            .map(|f| {
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .items_center()
                                    .py(px(2.))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::accent_yellow())
                                            .child("M"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_primary())
                                            .child(f.path.clone()),
                                    )
                            })
                            .collect()
                    }),
            )
            // Divider
            .child(div().h(px(1.)).bg(theme::border()).mx(px(12.)))
            // Approvals section
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("PENDING APPROVAL"),
                    )
                    .child(if let Some(approval) = &self.pending_approval {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .p(px(10.))
                            .rounded(px(6.))
                            .bg(theme::bg_hover())
                            .border_1()
                            .border_color(theme::accent_yellow())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::text_primary())
                                    .child(approval.description.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::accent_yellow())
                                    .child("Review required"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .py(px(6.))
                                            .rounded(px(4.))
                                            .bg(theme::accent_green())
                                            .text_xs()
                                            .text_color(theme::bg_base())
                                            .cursor_pointer()
                                            .hover(|s| s.bg(theme::bg_hover()))
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(|this, _: &MouseUpEvent, window, cx| {
                                                    this.approve_request(&ApproveRequest, window, cx);
                                                }),
                                            )
                                            .child("Approve"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .py(px(6.))
                                            .rounded(px(4.))
                                            .bg(theme::accent_red())
                                            .text_xs()
                                            .text_color(theme::bg_base())
                                            .cursor_pointer()
                                            .hover(|s| s.bg(theme::bg_hover()))
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(|this, _: &MouseUpEvent, window, cx| {
                                                    this.deny_request(&DenyRequest, window, cx);
                                                }),
                                            )
                                            .child("Deny"),
                                    ),
                            )
                    } else {
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child("No pending approvals")
                    }),
            )
            // Timeline
            .child(div().h(px(1.)).bg(theme::border()).mx(px(12.)))
            .child(
                div()
                    .px(px(12.))
                    .py(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .mb(px(4.))
                            .child("TIMELINE"),
                    )
                    .children(self.timeline.iter().rev().take(12).map(|entry| {
                        let color = if entry.is_approval {
                            theme::accent_yellow()
                        } else if entry.is_tool {
                            theme::accent_cyan()
                        } else {
                            theme::text_secondary()
                        };
                        div()
                            .text_xs()
                            .text_color(color)
                            .py(px(1.))
                            .child(format!("▸ {}", entry.description))
                    })),
            )
            // Output panel
            .child(
                div()
                    .h(px(120.))
                    .bg(theme::bg_base())
                    .border_t_1()
                    .border_color(theme::border())
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .text_xs()
                            .text_color(theme::text_muted())
                            .border_b_1()
                            .border_color(theme::border())
                            .child("EXECUTION OUTPUT"),
                    )
                    .child(
                        div()
                            .id("output-scroll")
                            .flex_1()
                            .px(px(12.))
                            .py(px(4.))
                            .overflow_y_scroll()
                            .children(self.output_lines.iter().map(|line| {
                                div()
                                    .text_xs()
                                    .text_color(theme::accent_green())
                                    .child(line.clone())
                            })),
                    ),
            )
    }
}

impl Render for Workbench {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("Workbench")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::new_session))
            .on_action(cx.listener(Self::send_prompt))
            .on_action(cx.listener(Self::approve_request))
            .on_action(cx.listener(Self::deny_request))
            .on_action(cx.listener(Self::cancel_session))
            .on_action(cx.listener(Self::on_input_submit))
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_base())
            // Status bar at top
            .child(
                div()
                    .h(px(32.))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(theme::bg_panel())
                    .border_b_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::accent_blue())
                                    .child("Deck"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("AI Agent Workbench"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!(" {}", self.workspace_name)),
                    ),
            )
            // Main content area
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.render_left_panel(cx))
                    .child(self.render_conversation(cx))
                    .child(self.render_right_panel(cx)),
            )
    }
}

impl Focusable for Workbench {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
