//! Primary / secondary buttons.
//!
//! The **primary** button is filled with the same color the sidebar uses
//! for the selected nav row (`visuals.selection.bg_fill`, with its text in
//! `visuals.selection.stroke` color) — so the call-to-action and the nav
//! highlight are always the same teal, by construction. The **secondary**
//! button is neutral and picks up the theme's control border treatment.

use egui::{Response, Ui};

use crate::metrics;

/// Call-to-action button, filled with the sidebar-selection color.
pub fn primary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    // Read the selection colors the sidebar's nav rows also use, so the
    // two always match regardless of theme/accent.
    let fill = ui.visuals().selection.bg_fill;
    let text_color = ui.visuals().selection.stroke.color;
    let label = egui::RichText::new(text.into()).color(text_color).strong();
    ui.add(
        egui::Button::new(label)
            .fill(fill)
            .corner_radius(egui::CornerRadius::same(metrics::CONTROL_CORNER)),
    )
}

/// Neutral secondary button — uses the theme's control treatment
/// (color-matched border at rest, visible on hover).
pub fn secondary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(
        egui::Button::new(text.into())
            .corner_radius(egui::CornerRadius::same(metrics::CONTROL_CORNER)),
    )
}
