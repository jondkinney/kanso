//! A compact circled-`i` info affordance with a tooltip.
//!
//! Both apps want a small "more info" marker beside a label — a circled `i`
//! the user hovers for an explanation that would otherwise need its own
//! caption line. The catch: the bundled font (and many system fonts) lack
//! the Unicode ⓘ (U+24D8) glyph, so a plain `RichText("ⓘ")` renders as a
//! tofu box. hyprcorrect's `info_icon` solved this by *hand-painting* the
//! marker — a circle stroke plus a small `i` — sized to the body text so it
//! sits flush next to the label. kanso adopts that, in cohort tokens.

use egui::{Response, Ui};

use crate::palette;

/// Paint a circled-`i` info marker at the cursor — sized to the body text
/// so it sits flush beside a [`super::field_label`] — and attach `tip` as
/// its hover tooltip. The marker is [`palette::TEXT_MUTED`] at rest and
/// brightens to [`palette::TEXT`] on hover. Returns the [`Response`] (with
/// the tooltip already attached).
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// ui.horizontal(|ui| {
///     kanso::widgets::field_label(ui, "Default provider");
///     kanso::widgets::info_icon(ui, "Which engine corrects a word when you fix it.");
/// });
/// # }
/// ```
pub fn info_icon(ui: &mut Ui, tip: &str) -> Response {
    // Size to the resolved body text so the marker lines up with the label
    // it annotates rather than the (taller) row hit-box around it.
    let size = egui::TextStyle::Body.resolve(ui.style()).size;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let color = if response.hovered() {
            palette::TEXT
        } else {
            palette::TEXT_MUTED
        };
        let painter = ui.painter();
        let center = rect.center();
        let radius = (size * 0.5) - 1.0;
        painter.circle_stroke(center, radius, egui::Stroke::new(1.0, color));
        // The `i` baseline sits low in most fonts; nudge up half a px so it
        // reads as vertically centered in the circle.
        painter.text(
            center + egui::vec2(0.0, -0.5),
            egui::Align2::CENTER_CENTER,
            "i",
            egui::FontId::proportional(size * 0.75),
            color,
        );
    }
    response.on_hover_text(tip)
}
