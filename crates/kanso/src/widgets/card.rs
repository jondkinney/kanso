//! A grouped settings surface — a rounded, padded, bordered container that
//! wraps arbitrary content.
//!
//! Both apps reach for the same shape whenever a logical group of settings
//! needs to read as one panel: vernier's `paint_handoff_card` /
//! `paint_figma_card` (prefs.rs:1537, :1982) and hyprcorrect's `card`
//! (review.rs:1489). They diverged in the details — vernier draws a
//! [`palette::CARD_STROKE`] border and rounds to 10px with `symmetric(18,16)`
//! padding; hyprcorrect omitted the border, rounded to 8px with
//! `symmetric(14,12)`, and forced the card to fill the available width.
//! kanso reconciles them into one canonical surface: vernier's bordered,
//! 10px ([`metrics::CARD_CORNER`]) look with vernier's padding, plus
//! hyprcorrect's full-width behaviour (the better default for a settings
//! pane) and its ergonomic `card(ui, |ui| ...) -> R` closure API.

use egui::Ui;

use crate::{metrics, palette};

/// Wrap `add` in a grouped card surface: [`palette::CARD`] fill, a
/// [`palette::CARD_STROKE`] border, [`metrics::CARD_CORNER`] rounding, and
/// symmetric inner padding ([`metrics::CARD_PAD_X`] / [`metrics::CARD_PAD_Y`]).
///
/// The card stretches to the full available width so stacked cards line up
/// rather than shrink-wrapping their content. Returns whatever the body
/// closure returns.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::widgets::card(ui, |ui| {
///     kanso::widgets::field_label(ui, "Integration");
///     ui.label("Settings grouped inside one surface.");
/// });
/// # }
/// ```
pub fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .fill(palette::CARD)
        .stroke(egui::Stroke::new(
            metrics::BORDER_WIDTH,
            palette::CARD_STROKE,
        ))
        .corner_radius(metrics::CARD_CORNER)
        .inner_margin(egui::Margin::symmetric(
            metrics::CARD_PAD_X,
            metrics::CARD_PAD_Y,
        ))
        .show(ui, |ui| {
            // Fill the available width so the card reads as a panel and
            // sibling cards align, instead of hugging its content.
            ui.set_min_width(ui.available_width());
            add(ui)
        })
        .inner
}
