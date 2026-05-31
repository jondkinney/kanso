//! One-call theming.
//!
//! [`apply`] installs the bundled font, the apps' shared text-style scale
//! (heading 21 / body 14 / mono 13 / small 12), and spacing — then sets
//! `expansion = 0` on every widget state so **nothing grows on hover**
//! (controls never shift next to their neighbours). Otherwise it rides
//! egui's default dark `Visuals`.
//!
//! The input/button border treatment is applied *per widget* by the
//! helpers (see [`crate::widgets`]) — not globally — so it doesn't leak
//! onto dropdown items, nav rows, or checkboxes.

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
            // Solid, space-reserving scrollbar. egui's default *floating*
            // bar overlays content, so full-width fields and combo-box drop
            // buttons get clipped under the float lane; the solid bar
            // reserves a lane (only the handle shows — track opacities 0).
            spacing.scroll = egui::style::ScrollStyle::solid();
            spacing.scroll.dormant_background_opacity = 0.0;
            spacing.scroll.active_background_opacity = 0.0;
            spacing.scroll.interact_background_opacity = 0.0;

            // One standard corner radius across every control and
            // container, and constant geometry (nothing grows on hover).
            let radius = egui::CornerRadius::same(metrics::RADIUS);
            let v = &mut style.visuals;
            v.window_corner_radius = radius;
            v.menu_corner_radius = radius;
            for state in [
                &mut v.widgets.noninteractive,
                &mut v.widgets.inactive,
                &mut v.widgets.hovered,
                &mut v.widgets.active,
                &mut v.widgets.open,
            ] {
                state.corner_radius = radius;
                state.expansion = 0.0;
            }
        });
    }
}

/// Apply the default cohort theme. Equivalent to
/// `Theme::default().apply(ctx)`.
pub fn apply(ctx: &Context) {
    Theme::default().apply(ctx);
}

/// Apply kanso's input/button **control treatment** to `visuals`: a
/// constant 1px border whose *color* tracks state — matched to the fill at
/// rest (so it reads as borderless), [`palette::BORDER`] on hover,
/// [`palette::ACCENT`] on press — plus the control fills, the standard
/// [`metrics::RADIUS`], and `expansion = 0` so a control never changes
/// height.
///
/// kanso's own inputs/buttons apply this in a *local scope* so it doesn't
/// bleed onto dropdown items or nav rows. An app with many **direct** egui
/// widgets can instead apply it **globally** in its startup style, so every
/// `egui::Button` / `TextEdit` / `ComboBox` picks up the kanso look without
/// being routed through a kanso helper:
///
/// ```no_run
/// # fn demo(ctx: &kanso::egui::Context) {
/// kanso::theme::apply(ctx);
/// ctx.style_mut(|s| kanso::theme::control_visuals(&mut s.visuals));
/// # }
/// ```
pub fn control_visuals(visuals: &mut egui::Visuals) {
    let radius = egui::CornerRadius::same(metrics::RADIUS);
    let stroke = |c| egui::Stroke::new(metrics::BORDER_WIDTH, c);
    visuals.extreme_bg_color = palette::CONTROL_BG; // text-edit fill

    let w = &mut visuals.widgets;
    w.noninteractive.corner_radius = radius;
    w.noninteractive.expansion = 0.0;

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

    w.open.weak_bg_fill = palette::CONTROL_BG_HOVER;
    w.open.bg_fill = palette::CONTROL_BG_HOVER;
    w.open.bg_stroke = stroke(palette::BORDER);
    w.open.corner_radius = radius;
    w.open.expansion = 0.0;
}
