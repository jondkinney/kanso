//! Inline `code` highlight — a markdown-style backtick span: monospace
//! text on a subtle background chip.

use egui::{Response, Ui};

use crate::palette;

/// Build a styled monospace [`egui::RichText`] for an inline code span.
/// Use inside a `ui.label(...)` or a `RichText`-accepting call, or
/// interleave with body text in a `ui.horizontal(...)`.
pub fn code_span(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into())
        .monospace()
        .color(palette::TEXT)
        .background_color(palette::CODE_BG)
}

/// Render an inline code span as its own label widget.
pub fn code(ui: &mut Ui, text: &str) -> Response {
    ui.label(code_span(text))
}
