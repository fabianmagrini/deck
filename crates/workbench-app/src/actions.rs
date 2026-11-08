//! GPUI action types for the Deck workbench.
//!
//! Actions are zero-sized marker types that GPUI uses for its keyboard binding
//! and event dispatch system. Each action maps to exactly one keybinding
//! registered in [`crate::app::setup`].
//!
//! Two namespaces are defined:
//! - `workbench` — high-level session and workspace operations.
//! - `text_input` — cursor movement and editing within the [`TextInput`][crate::views::input::TextInput] widget.

use gpui::actions;

actions!(
    workbench,
    [
        /// Quit the application (`cmd-q`).
        Quit,
        /// Open a new agent session (`cmd-n`).
        NewSession,
        /// Open a workspace directory from disk.
        OpenWorkspace,
        /// Submit the current prompt to the active session (`cmd-return`).
        SendPrompt,
        /// Cancel the active session's current operation.
        CancelSession,
        /// Approve a pending agent approval request.
        ApproveRequest,
        /// Deny a pending agent approval request.
        DenyRequest,
        /// Move keyboard focus to the prompt input.
        FocusInput,
    ]
);

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
        /// Submit the input (fires `SendPrompt` upstream).
        Submit,
    ]
);
