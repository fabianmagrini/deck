# workbench-app

The GPUI desktop application binary (`deck`). This crate composes the domain logic from the other workspace crates into a native macOS window.

## Modules

| Module | Description |
|--------|-------------|
| `main.rs` | Entry point — initialises tracing, creates the GPUI `App`, calls `app::setup` |
| `app.rs` | Registers keybindings, opens the main window, sets initial focus |
| `theme.rs` | Tokyo Night dark-theme colour palette as named `Hsla` functions |
| `actions.rs` | GPUI action types for keyboard dispatch (`workbench` and `text_input` namespaces) |
| `views/workbench.rs` | Root `Workbench` GPUI view — three-panel layout, session management, event handling |
| `views/input.rs` | `TextInput` widget — full IME support via `EntityInputHandler`, cursor, selection |

## Building

From the workspace root:

```bash
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 cargo build -p workbench-app
```

See the [root README](../../README.md) for Metal Toolchain setup instructions.

## Running

```bash
TOOLCHAINS=com.apple.dt.toolchain.Metal.32023.922.4 ./target/debug/deck
```

## UI layout

```
┌────────────────────────────────────────────────────────────┐
│  Deck — AI Agent Workbench                                 │
├──────────────┬──────────────────────────┬──────────────────┤
│  WORKSPACE   │                          │   INSPECTOR      │
│  demo-proj   │   Session title          │                  │
│    src/      │   ● Running              │   CHANGES        │
│              │                          │   src/main.rs    │
│  SESSIONS    │   [conversation          │   +42  −18       │
│  Refactor ●  │    messages]             │                  │
│  Add tests   │                          │   PENDING        │
│  Fix logging │                          │   APPROVAL       │
│              ├──────────────────────────┤   Execute test   │
│         [+]  │  Message agent...  Send  ├──────────────────┤
│              │                          │   EXECUTION      │
│              │                          │   OUTPUT         │
└──────────────┴──────────────────────────┴──────────────────┘
```

**Left panel (220 px):** workspace file tree and session list with status indicators.  
**Centre panel:** conversation history with streaming text, tool activity blocks, and the prompt input.  
**Right panel:** file change list (with line counts), pending approval requests, and execution output.

## Theme

All colours are defined in `theme.rs` as functions returning `gpui::Hsla`. Never hard-code hex values in view code — always call a theme function so the palette stays in one place.

| Function | Hex | Use |
|----------|-----|-----|
| `bg_base()` | `#1a1b26` | Main content area |
| `bg_panel()` | `#1f2335` | Sidebars |
| `bg_hover()` | `#292e42` | Hover states |
| `bg_selected()` | `#33467c` | Active session |
| `text_primary()` | `#c0caf5` | Body text |
| `accent_blue()` | `#7aa2f7` | Links, titles |
| `status_running()` | `#9ece6a` | Green dot |
| `status_waiting()` | `#e0af68` | Yellow dot |
| `status_failed()` | `#f7768e` | Red dot |

## Keybindings

Keybindings are registered in `app::setup` and dispatch typed GPUI actions.

| Binding | Action | Context |
|---------|--------|---------|
| `cmd-q` | `Quit` | global |
| `cmd-n` | `NewSession` | `Workbench` |
| `cmd-return` | `SendPrompt` | `Workbench` |
| `backspace` | `Backspace` | `TextInput` |
| `delete` | `Delete` | `TextInput` |
| `←` / `→` | `Left` / `Right` | `TextInput` |
| `shift-←` / `shift-→` | `SelectLeft` / `SelectRight` | `TextInput` |
| `cmd-a` | `SelectAll` | `TextInput` |
| `cmd-v / c / x` | `Paste / Copy / Cut` | `TextInput` |
| `home` / `end` | `Home` / `End` | `TextInput` |
| `return` | `Submit` | `TextInput` |
