//! Inline `code` — a rounded monospace pill, matching the inline code
//! spans [`super::caption`] paints (same [`metrics::CODE_CORNER`] rounding
//! and a snug padding) so a standalone code chip reads identically to one
//! inside a caption.

use egui::{Response, RichText, Ui};

use crate::{metrics, palette};

/// A standalone inline code pill: monospace text on a rounded
/// [`palette::CODE_BG`] background.
pub fn code(ui: &mut Ui, text: &str) -> Response {
    let font = egui::FontId::new(metrics::CODE_SIZE, egui::FontFamily::Monospace);
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_owned(), font, palette::CODE_TEXT));
    let pad = egui::vec2(metrics::CODE_PAD_X, metrics::CODE_PAD_Y);
    let (rect, response) = ui.allocate_exact_size(galley.size() + pad * 2.0, egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, metrics::CODE_CORNER, palette::CODE_BG);
    ui.painter()
        .galley(rect.min + pad, galley, palette::CODE_TEXT);
    response
}

/// Build a styled monospace [`RichText`] for an inline code span that must
/// **flow inline** with surrounding text inside a single `ui.label(...)`
/// (where a painted pill can't participate in wrapping). Uses a
/// rectangular background rather than a rounded pill; prefer [`code`] for
/// standalone chips.
pub fn code_span(text: impl Into<String>) -> RichText {
    RichText::new(text.into())
        .monospace()
        .color(palette::CODE_TEXT)
        .background_color(palette::CODE_BG)
}
