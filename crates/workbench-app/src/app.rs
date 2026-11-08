//! Application bootstrap: keybindings, window creation, and initial focus.

use gpui::{App, AppContext, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions, px, size};

use crate::actions::{
    ApproveRequest, Backspace, CancelSession, Copy, Cut, Delete, DenyRequest, End, Home,
    Left, NewSession, Paste, Quit, Right, SelectAll, SelectLeft, SelectRight, SendPrompt, Submit,
};
use crate::views::Workbench;

/// Register global keybindings and open the main workbench window.
///
/// Called once from `main` after GPUI's [`App`] is created. The returned
/// window handle is not retained because the workbench owns its own lifecycle.
pub fn setup(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-n", NewSession, Some("Workbench")),
        KeyBinding::new("cmd-return", SendPrompt, Some("Workbench")),
        // TextInput bindings
        KeyBinding::new("backspace", Backspace, Some("TextInput")),
        KeyBinding::new("delete", Delete, Some("TextInput")),
        KeyBinding::new("left", Left, Some("TextInput")),
        KeyBinding::new("right", Right, Some("TextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
        KeyBinding::new("cmd-a", SelectAll, Some("TextInput")),
        KeyBinding::new("cmd-v", Paste, Some("TextInput")),
        KeyBinding::new("cmd-c", Copy, Some("TextInput")),
        KeyBinding::new("cmd-x", Cut, Some("TextInput")),
        KeyBinding::new("home", Home, Some("TextInput")),
        KeyBinding::new("end", End, Some("TextInput")),
        KeyBinding::new("return", Submit, Some("TextInput")),
    ]);

    cx.on_action(|_: &Quit, cx| cx.quit());

    let bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Deck — AI Agent Workbench".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                focus: true,
                show: true,
                is_resizable: true,
                is_minimizable: true,
                window_min_size: Some(size(px(900.), px(600.))),
                ..Default::default()
            },
            |_window, cx| cx.new(Workbench::new),
        )
        .unwrap();

    window
        .update(cx, |view: &mut crate::views::Workbench, window, cx| {
            let handle = view.focus_handle.clone();
            window.focus(&handle);
            cx.activate(true);
        })
        .unwrap();
}
