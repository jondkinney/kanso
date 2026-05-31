//! A labeled color swatch that opens egui's built-in color picker.
//!
//! A thin, consistent wrapper over [`egui::color_picker::color_edit_button_srgba`]:
//! a clickable swatch that pops the standard picker, a leading label, and a
//! trailing `#RRGGBB` / `#RRGGBBAA` readout rendered as a kanso code pill.
//!
//! Reconciled from vernier (`vernier-ui/src/prefs.rs:3211`, `color_picker(ui,
//! &mut ColorRgba)`): it converts its `ColorRgba` to a [`egui::Color32`], calls
//! `color_edit_button_srgba(.., Alpha::OnlyBlend)`, writes the channels back on
//! `.changed()`, and appends a `#RRGGBB (a=N)` label. hyprcorrect has no color
//! picker. The canonical kanso surface drops vernier's app-specific `ColorRgba`
//! for plain [`egui::Color32`] (kanso depends only on egui), keeps vernier's
//! `Alpha::OnlyBlend`, takes the label as an argument so the widget is
//! self-contained, and renders the hex readout through [`super::code`] so it
//! matches the rest of the design system. The swatch button is input-like, so
//! it wears the scoped control border treatment ([`super::control::scope`]).

use egui::{Color32, Response, Ui};

use crate::palette;

/// A labeled color swatch. Click the swatch to open egui's standard color
/// picker; the trailing pill shows the current value as `#RRGGBB` (or
/// `#RRGGBBAA` when not fully opaque). Edits the [`Color32`] in place and
/// returns `true` when the value changed this frame.
///
/// Alpha is editable with normal blending ([`egui::color_picker::Alpha::OnlyBlend`]),
/// matching vernier's overlay colors. For an opaque-only swatch (no alpha
/// slider, alpha forced to 255) use [`color_picker_opaque`].
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::egui::Color32;
/// let mut primary = Color32::from_rgb(79, 163, 255);
/// if kanso::widgets::color_picker(ui, &mut primary, "Primary color") {
///     // value changed — persist it
/// }
/// # }
/// ```
pub fn color_picker(ui: &mut Ui, color: &mut Color32, label: &str) -> bool {
    swatch(ui, color, label, egui::color_picker::Alpha::OnlyBlend).0
}

/// Like [`color_picker`], but opaque-only: the picker shows no alpha slider and
/// the stored alpha is pinned to `255`. The readout is always `#RRGGBB`.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::egui::Color32;
/// let mut guide = Color32::from_rgb(80, 200, 120);
/// kanso::widgets::color_picker_opaque(ui, &mut guide, "Guide color");
/// # }
/// ```
pub fn color_picker_opaque(ui: &mut Ui, color: &mut Color32, label: &str) -> bool {
    swatch(ui, color, label, egui::color_picker::Alpha::Opaque).0
}

/// Shared body: `label  [swatch]  #hex`, laid out on one baseline-aligned row.
/// Returns `(changed, response)` where `response` is the swatch button's.
fn swatch(
    ui: &mut Ui,
    color: &mut Color32,
    label: &str,
    alpha: egui::color_picker::Alpha,
) -> (bool, Response) {
    ui.horizontal(|ui| {
        if !label.is_empty() {
            ui.label(egui::RichText::new(label).color(palette::TEXT));
        }
        // The swatch button is an input-like control: borrow the scoped
        // control border so its rest/hover/press outline matches the inputs
        // and buttons beside it instead of egui's default widget stroke.
        let response = super::control::scope(ui, |ui| {
            egui::color_picker::color_edit_button_srgba(ui, color, alpha)
        });
        // Trailing hex readout as a code pill, consistent with the rest of
        // the system (vernier showed a bare `#RRGGBB (a=N)` label).
        super::code(
            ui,
            &hex(*color, matches!(alpha, egui::color_picker::Alpha::Opaque)),
        );
        (response.changed(), response)
    })
    .inner
}

/// Format a color as `#RRGGBB`, or `#RRGGBBAA` when alpha is shown and the
/// color is not fully opaque.
fn hex(c: Color32, opaque: bool) -> String {
    if opaque || c.a() == u8::MAX {
        format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", c.r(), c.g(), c.b(), c.a())
    }
}
