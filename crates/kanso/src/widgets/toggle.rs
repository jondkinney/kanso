//! Labeled on/off toggle for boolean settings.
//!
//! Both apps express every boolean setting as a stock `ui.checkbox(&mut
//! bool, "inline label")` (vernier `prefs.rs:1147,1220,1273,…`,
//! hyprcorrect `prefs.rs:1521,1540,1560,…`) and rely on the cohort theme's
//! global checkbox geometry (`icon_width = 18.0`, `icon_spacing = 6.0` —
//! see [`crate::theme`]) plus the dark default checkmark visuals. The
//! recurring shape around it is identical in both:
//!
//! ```text
//! field_label(ui, "Review popup");
//! ui.add_space(4.0);
//! if ui.checkbox(&mut flag, "Open in vim mode").changed() { … }
//! ui.add_space(6.0);
//! caption(ui, "…explainer…");
//! ```
//!
//! Neither app restyles the checkbox itself, so the canonical kanso
//! widget is a *tidy labeled wrapper* over the stock checkbox rather than
//! a bespoke iOS switch: [`toggle`] is the one-liner, and
//! [`labeled_toggle`] reproduces the full `field_label` + checkbox +
//! caption block both apps hand-rolled, so per-setting call sites stay
//! flat and the rhythm lives in one place.

use egui::{Response, Ui};

/// A boolean on/off toggle: the cohort's stock checkbox with an inline
/// `label`, flipping `on` and returning the click [`Response`] (check
/// `.changed()` to react). Inherits the global checkbox geometry and the
/// dark theme's checkmark visuals, so it lines up with every other
/// control and reads identically across the apps.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui, vim: &mut bool) {
/// if kanso::widgets::toggle(ui, vim, "Open in vim mode").changed() {
///     // persist the new value…
/// }
/// # }
/// ```
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    ui.checkbox(on, label)
}

/// The full labeled-toggle settings block both apps repeat: a bold
/// [`super::field_label`] title, a small gap, the [`toggle`] row, and —
/// when `caption` is non-empty — the matching gap and muted
/// [`super::caption`] explainer (which renders `` `backtick` `` spans as
/// inline code pills). Returns the toggle's [`Response`]; the surrounding
/// [`super::setting`] block owns the trailing spacing.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui, vim: &mut bool) {
/// kanso::widgets::setting(ui, |ui| {
///     kanso::widgets::labeled_toggle(
///         ui,
///         "Review popup",
///         vim,
///         "Open in vim mode",
///         "Start the review popup in vim mode. `Ctrl+E` toggles either way.",
///     );
/// });
/// # }
/// ```
pub fn labeled_toggle(
    ui: &mut Ui,
    title: &str,
    on: &mut bool,
    label: &str,
    caption: &str,
) -> Response {
    super::field_label(ui, title);
    ui.add_space(TITLE_GAP);
    let response = toggle(ui, on, label);
    if !caption.is_empty() {
        ui.add_space(CAPTION_GAP);
        super::caption(ui, caption);
    }
    response
}

/// Gap between a [`super::field_label`] and the toggle row beneath it —
/// the `add_space(4.0)` both apps place after the label.
const TITLE_GAP: f32 = 4.0;
/// Gap between the toggle row and its caption — the `add_space(6.0)` both
/// apps place before the explainer.
const CAPTION_GAP: f32 = 6.0;
