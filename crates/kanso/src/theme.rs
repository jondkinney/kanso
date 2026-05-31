//! One-call theming.
//!
//! [`apply`] installs the bundled font plus the apps' shared text-style
//! scale (heading 21 / body 14 / mono 13 / small 12), spacing, and
//! button padding. Like the apps, it **rides egui's default dark
//! `Visuals`** and only overrides text styles + spacing +
//! `widgets.inactive.expansion` — so dropping it into vernier/hyprcorrect
//! does not repaint any surface. This is exactly vernier's `apply_style`
//! + `install_glyph_fonts`, lifted into one place.

use egui::Context;

use crate::fonts;

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
            spacing.interact_size = egui::vec2(40.0, 28.0);
            spacing.icon_width = 18.0;
            spacing.icon_spacing = 6.0;

            // The one Visuals tweak the apps make: kill the 1px grow on
            // hover so settings rows don't twitch.
            style.visuals.widgets.inactive.expansion = 0.0;
        });
    }
}

/// Apply the default cohort theme. Equivalent to
/// `Theme::default().apply(ctx)`.
pub fn apply(ctx: &Context) {
    Theme::default().apply(ctx);
}
