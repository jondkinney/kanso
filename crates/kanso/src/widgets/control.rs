//! Internal: the input/button control treatment, applied in a **scope** so
//! it only affects the wrapped widget — not sibling dropdown items, nav
//! rows, checkboxes, etc.
//!
//! The border is a constant 1px whose *color* tracks state — matched to
//! the fill at rest (reads as borderless), [`palette::BORDER`] on hover,
//! [`palette::ACCENT`] on press — with `expansion = 0` so the control
//! never changes height.

use egui::Ui;

use crate::{metrics, palette};

/// Run `add` with the control visuals applied locally (restored after).
pub(crate) fn scope<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        let radius = egui::CornerRadius::same(metrics::RADIUS);
        let stroke = |c| egui::Stroke::new(metrics::BORDER_WIDTH, c);
        let v = ui.visuals_mut();
        v.extreme_bg_color = palette::CONTROL_BG; // text-edit fill

        let w = &mut v.widgets;
        w.inactive.weak_bg_fill = palette::CONTROL_BG;
        w.inactive.bg_fill = palette::CONTROL_BG;
        w.inactive.bg_stroke = stroke(palette::CONTROL_BG); // matched → invisible
        w.inactive.corner_radius = radius;
        w.inactive.expansion = 0.0;

        w.hovered.weak_bg_fill = palette::CONTROL_BG_HOVER;
        w.hovered.bg_fill = palette::CONTROL_BG_HOVER;
        w.hovered.bg_stroke = stroke(palette::BORDER); // color change only
        w.hovered.corner_radius = radius;
        w.hovered.expansion = 0.0;

        w.active.weak_bg_fill = palette::CONTROL_BG_HOVER;
        w.active.bg_fill = palette::CONTROL_BG_HOVER;
        w.active.bg_stroke = stroke(palette::ACCENT);
        w.active.corner_radius = radius;
        w.active.expansion = 0.0;

        add(ui)
    })
    .inner
}
