//! Primary / secondary buttons.
//!
//! The **primary** button is filled with the sidebar-selection color (the
//! same teal the selected nav row uses) with a border *matched to that
//! fill* — so it reads as a solid teal button with no visible outline and
//! never changes height; hover just lightens the fill. The **secondary**
//! button is neutral and uses the scoped control border treatment.

use egui::{Color32, Response, Ui};

use crate::metrics;

/// Call-to-action button, filled with the sidebar-selection color.
pub fn primary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    // Same source the sidebar's selected row uses, so they always match.
    let base = ui.visuals().selection.bg_fill;
    let hover = lighten(base, 0.16);
    let text_color = ui.visuals().selection.stroke.color;

    ui.scope(|ui| {
        let radius = egui::CornerRadius::same(metrics::CONTROL_CORNER);
        let w = &mut ui.visuals_mut().widgets;
        // Border matched to the fill in every state (invisible outline),
        // constant geometry; hover/press only lighten the fill.
        for (state, fill) in [
            (&mut w.inactive, base),
            (&mut w.hovered, hover),
            (&mut w.active, hover),
        ] {
            state.weak_bg_fill = fill;
            state.bg_fill = fill;
            state.bg_stroke = egui::Stroke::new(metrics::BORDER_WIDTH, fill);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }
        ui.add(egui::Button::new(
            egui::RichText::new(text.into()).color(text_color).strong(),
        ))
    })
    .inner
}

/// Neutral secondary button — scoped control border (matched at rest,
/// visible on hover; never changes height).
pub fn secondary_button(ui: &mut Ui, text: impl Into<String>) -> Response {
    super::control::scope(ui, |ui| ui.add(egui::Button::new(text.into())))
}

/// Blend a color toward white by `t` (0..=1).
fn lighten(c: Color32, t: f32) -> Color32 {
    let mix = |x: u8| (x as f32 + (255.0 - x as f32) * t).round() as u8;
    Color32::from_rgb(mix(c.r()), mix(c.g()), mix(c.b()))
}
