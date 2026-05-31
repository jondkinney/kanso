//! One-call theming.
//!
//! [`apply`] installs the bundled font plus the apps' shared text-style
//! scale (heading 21 / body 14 / mono 13 / small 12), spacing, and a
//! deliberate **control treatment** for inputs and neutral buttons:
//!
//! - one fixed [`metrics::CONTROL_HEIGHT`] so inputs and buttons line up;
//! - a **constant-geometry border** — always 1px, color-matched to the
//!   control's fill at rest (so it reads as borderless), switching to
//!   [`palette::BORDER`] on hover and [`palette::ACCENT`] on press. Only
//!   the *color* changes — never the width or `expansion` — so a control
//!   never changes height next to its non-hovered neighbours.
//!
//! Everything else rides egui's default dark `Visuals`.

use egui::Context;

use crate::{fonts, metrics, palette};

/// Theme configuration. [`Theme::default`] is the cohort look; tweak the
/// type-scale fields or [`fonts::FontOptions`] before calling
/// [`Theme::apply`].
#[derive(Clone, Debug)]
pub struct Theme {
    /// Heading text size.
    pub heading: f32,
    /// Body text size.
    pub body: f32,
    /// Monospace size (inline `code`, numeric fields).
    pub mono: f32,
    /// Small / caption size.
    pub small: f32,
    /// Accent for focus/press borders and selection.
    pub accent: egui::Color32,
    /// Font installation options.
    pub fonts: fonts::FontOptions,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            heading: 21.0,
            body: 14.0,
            mono: 13.0,
            small: 12.0,
            accent: palette::ACCENT,
            fonts: fonts::FontOptions::default(),
        }
    }
}

impl Theme {
    /// Install fonts + styles into the context. Call once at startup.
    pub fn apply(&self, ctx: &Context) {
        fonts::install(ctx, &self.fonts);
        ctx.style_mut(|style| {
            use egui::FontFamily::{Monospace, Proportional};
            use egui::TextStyle::{Body, Button, Heading, Small};

            style.text_styles = [
                (Heading, egui::FontId::new(self.heading, Proportional)),
                (Body, egui::FontId::new(self.body, Proportional)),
                (
                    egui::TextStyle::Monospace,
                    egui::FontId::new(self.mono, Monospace),
                ),
                (Button, egui::FontId::new(self.body, Proportional)),
                (Small, egui::FontId::new(self.small, Proportional)),
            ]
            .into();

            let spacing = &mut style.spacing;
            spacing.item_spacing = egui::vec2(8.0, 8.0);
            spacing.button_padding = egui::vec2(12.0, 6.0);
            spacing.indent = 14.0;
            spacing.interact_size = egui::vec2(40.0, metrics::CONTROL_HEIGHT);
            spacing.icon_width = 18.0;
            spacing.icon_spacing = 6.0;

            apply_control_visuals(&mut style.visuals, self.accent);
        });
    }
}

/// Configure the input/button control treatment on top of egui's dark
/// visuals: a constant-width border whose *color* (not size) tracks the
/// interaction state, so controls never change height.
fn apply_control_visuals(visuals: &mut egui::Visuals, accent: egui::Color32) {
    let w = metrics::BORDER_WIDTH;
    let radius = egui::CornerRadius::same(metrics::CONTROL_CORNER);
    let stroke = |color| egui::Stroke::new(w, color);

    // Text-edit fill. `selection` is left untouched so the sidebar nav
    // (and the primary button, which derives from it) keep their color and
    // the text-edit focus ring stays egui's selection stroke.
    visuals.extreme_bg_color = palette::CONTROL_BG;

    let widgets = &mut visuals.widgets;

    // Rest: border color-matched to the fill → reads as borderless.
    widgets.inactive.weak_bg_fill = palette::CONTROL_BG;
    widgets.inactive.bg_fill = palette::CONTROL_BG;
    widgets.inactive.bg_stroke = stroke(palette::CONTROL_BG);
    widgets.inactive.corner_radius = radius;
    widgets.inactive.expansion = 0.0;

    // Hover: same geometry, the border simply gains a visible color.
    widgets.hovered.weak_bg_fill = palette::CONTROL_BG_HOVER;
    widgets.hovered.bg_fill = palette::CONTROL_BG_HOVER;
    widgets.hovered.bg_stroke = stroke(palette::BORDER);
    widgets.hovered.corner_radius = radius;
    widgets.hovered.expansion = 0.0;

    // Press / open: accent border.
    widgets.active.weak_bg_fill = palette::CONTROL_BG_HOVER;
    widgets.active.bg_fill = palette::CONTROL_BG_HOVER;
    widgets.active.bg_stroke = stroke(accent);
    widgets.active.corner_radius = radius;
    widgets.active.expansion = 0.0;

    widgets.open.bg_stroke = stroke(palette::BORDER);
    widgets.open.corner_radius = radius;
    widgets.open.expansion = 0.0;
}

/// Apply the default cohort theme. Equivalent to
/// `Theme::default().apply(ctx)`.
pub fn apply(ctx: &Context) {
    Theme::default().apply(ctx);
}
