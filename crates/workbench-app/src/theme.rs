//! Tokyo Night dark theme colours.
//!
//! All colours are taken directly from the
//! [Tokyo Night](https://github.com/folke/tokyonight.nvim) palette so the UI
//! matches the spec. Use these functions anywhere GPUI needs an `Hsla` value;
//! never hard-code hex literals in view code.

use gpui::rgb;
use gpui::Hsla;

// ── Background layers ────────────────────────────────────────────────────────

/// Main editor / content area background (`#1a1b26`).
pub fn bg_base() -> Hsla {
    rgb(0x1a1b26).into()
}

/// Sidebar / panel background (`#1f2335`).
pub fn bg_panel() -> Hsla {
    rgb(0x1f2335).into()
}

/// Background on hover (`#292e42`).
pub fn bg_hover() -> Hsla {
    rgb(0x292e42).into()
}

/// Background for the active / selected item (`#33467c`).
pub fn bg_selected() -> Hsla {
    rgb(0x33467c).into()
}

/// Input field background (`#24283b`).
pub fn bg_input() -> Hsla {
    rgb(0x24283b).into()
}

// ── Borders ──────────────────────────────────────────────────────────────────

/// Panel border colour (`#3b4261`).
pub fn border() -> Hsla {
    rgb(0x3b4261).into()
}

// ── Text ─────────────────────────────────────────────────────────────────────

/// Primary text (`#c0caf5`).
pub fn text_primary() -> Hsla {
    rgb(0xc0caf5).into()
}

/// Secondary / de-emphasised text (`#7982a9`).
pub fn text_secondary() -> Hsla {
    rgb(0x7982a9).into()
}

/// Muted section-label text (`#565f89`).
pub fn text_muted() -> Hsla {
    rgb(0x565f89).into()
}

// ── Accent colours ───────────────────────────────────────────────────────────

/// Tokyo Night blue — links, active session titles (`#7aa2f7`).
pub fn accent_blue() -> Hsla {
    rgb(0x7aa2f7).into()
}

/// Tokyo Night green — success, file additions (`#9ece6a`).
pub fn accent_green() -> Hsla {
    rgb(0x9ece6a).into()
}

/// Tokyo Night red — errors, file deletions (`#f7768e`).
pub fn accent_red() -> Hsla {
    rgb(0xf7768e).into()
}

/// Tokyo Night yellow/orange — warnings (`#e0af68`).
pub fn accent_yellow() -> Hsla {
    rgb(0xe0af68).into()
}

/// Tokyo Night cyan — info, secondary highlights (`#7dcfff`).
pub fn accent_cyan() -> Hsla {
    rgb(0x7dcfff).into()
}

// ── Status indicator colours ─────────────────────────────────────────────────

/// Green dot — session is actively running.
pub fn status_running() -> Hsla {
    rgb(0x9ece6a).into()
}

/// Yellow dot — session is waiting for user approval.
pub fn status_waiting() -> Hsla {
    rgb(0xe0af68).into()
}

/// Red dot — session failed or disconnected.
pub fn status_failed() -> Hsla {
    rgb(0xf7768e).into()
}

/// Grey dot — session is idle (Created / Ready / Completed).
pub fn status_idle() -> Hsla {
    rgb(0x7982a9).into()
}
