//! Single-line secret input with bullet masking.
//!
//! The same padding, height, and scoped control border as
//! [`super::padded_text_edit`] — only the contents render as bullets
//! (`TextEdit::password(true)`). hyprcorrect uses this for LLM API keys
//! (`padded_password_edit`, prefs.rs:2678, and inline at prefs.rs:2538);
//! vernier has no secret input, so there was nothing to reconcile — this
//! is hyprcorrect's widget reskinned onto kanso's tokens.
//!
//! Two entry points:
//! - [`password_field`] — the faithful port: always-masked, fills the row.
//! - [`password_field_revealable`] — additive. A trailing eye toggle that
//!   flips between bullets and plaintext. Neither app shipped a reveal
//!   toggle, so this is opt-in and never changes the default behaviour.

use egui::{Response, Ui};

use crate::metrics;

/// Single-line secret input. Contents render as bullets; padding, height,
/// and border match [`super::padded_text_edit`] so it lines up with the
/// fields and buttons around it.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// let mut api_key = String::new();
/// let resp = kanso::widgets::password_field(ui, &mut api_key);
/// if resp.changed() {
///     // persist the secret
/// }
/// # }
/// ```
pub fn password_field(ui: &mut Ui, text: &mut String) -> Response {
    super::control::scope(ui, |ui| {
        ui.add(
            egui::TextEdit::singleline(text)
                .password(true)
                .margin(egui::Margin::symmetric(8, 6))
                .min_size(egui::vec2(0.0, metrics::CONTROL_HEIGHT))
                .desired_width(f32::INFINITY),
        )
    })
}

/// Like [`password_field`], but with a trailing eye toggle that reveals the
/// secret as plaintext while held visible. The reveal state is keyed by
/// `id_salt` and stored in egui's per-widget memory, so it survives frames
/// without the caller threading a `bool`.
///
/// Returns the [`Response`] of the text field (the meaningful one for
/// `.changed()` / `.lost_focus()`), unioned with the toggle so the whole
/// row participates in hit-testing.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// let mut token = String::new();
/// kanso::widgets::password_field_revealable(ui, "github_token", &mut token);
/// # }
/// ```
pub fn password_field_revealable(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    text: &mut String,
) -> Response {
    let toggle_id = ui.make_persistent_id(("kanso_password_reveal", id_salt));
    // Default false → masked. Persisted in temp memory, cleared on restart.
    let mut revealed = ui.data_mut(|d| d.get_temp::<bool>(toggle_id).unwrap_or(false));

    // Reserve room on the right for a square toggle the same height as the
    // control, plus the inter-item gap, so the field fills the remainder.
    let gap = ui.spacing().item_spacing.x;
    let toggle_w = metrics::CONTROL_HEIGHT;

    let row = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let field_w = (ui.available_width() - toggle_w - gap).max(40.0);

            let field = super::control::scope(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(text)
                        // `password` masks only when NOT revealed.
                        .password(!revealed)
                        .margin(egui::Margin::symmetric(8, 6))
                        .min_size(egui::vec2(0.0, metrics::CONTROL_HEIGHT))
                        .desired_width(field_w),
                )
            });

            // Eye / eye-off glyph; a neutral scoped button so it shares the
            // control's resting/hover/press border treatment.
            let icon = if revealed { "🙈" } else { "👁" };
            let toggle = super::control::scope(ui, |ui| {
                ui.add_sized(
                    egui::vec2(toggle_w, metrics::CONTROL_HEIGHT),
                    egui::Button::new(icon),
                )
            })
            .on_hover_text(if revealed { "Hide" } else { "Reveal" });

            if toggle.clicked() {
                revealed = !revealed;
            }
            field.union(toggle)
        })
        .inner;

    ui.data_mut(|d| d.insert_temp(toggle_id, revealed));
    row
}
