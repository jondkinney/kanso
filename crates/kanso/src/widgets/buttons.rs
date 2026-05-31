//! Primary / secondary buttons with the cohort's accent + corner radius.

use egui::{Response, Ui};

use crate::palette;

/// Accent-filled call-to-action button.
pub fn primary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    let label = egui::RichText::new(text.into())
        .color(palette::ON_ACCENT)
        .strong();
    ui.add(
        egui::Button::new(label)
            .fill(palette::ACCENT)
            .corner_radius(egui::CornerRadius::same(6)),
    )
}

/// Neutral secondary button (default widget fill).
pub fn secondary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.add(egui::Button::new(text.into()).corner_radius(egui::CornerRadius::same(6)))
}
