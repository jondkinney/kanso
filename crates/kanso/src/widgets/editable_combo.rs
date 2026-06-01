//! A typeable combo box — a text field you can edit *or* pick from a menu.
//!
//! [`super::dropdown`] wraps egui's `ComboBox`: fixed options, no typing.
//! But both apps also need the *editable* combobox — "pick a known value or
//! enter your own" — for fields like an LLM backend/model id, where the
//! suggestion list is a convenience, not a constraint (hyprcorrect's
//! `editable_combo`: type `ollama` or a custom model tag the menu doesn't
//! list). egui has no editable ComboBox, so this hand-wires a single-line
//! field + a chevron drop button + an anchored [`egui::Popup`] menu.
//!
//! The chevron is hand-painted (like [`super::shortcut_chip`]'s glyphs and
//! [`super::info_icon`]) because the bundled fonts lack the Geometric-Shapes
//! triangle glyph and would tofu it.

use std::hash::Hash;

use egui::{Response, Ui};

use crate::{metrics, palette};

/// A typeable combo box: a single-line text field plus a chevron button that
/// drops a menu of suggested `options`. Picking an option overwrites the
/// field; the user can also type a value not in the list. Returns `true` if
/// `text` changed this frame (typed or picked).
///
/// The field and chevron share the cohort control-border treatment, and the
/// whole field is the drop trigger (click the field or the chevron to open
/// the menu). Pass an `id_salt` unique within the parent `Ui` so multiple
/// combos don't share popup state.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// let mut model = "claude-opus-4-8".to_string();
/// kanso::widgets::editable_combo(
///     ui,
///     "model",
///     &mut model,
///     &["claude-haiku-4-5", "claude-sonnet-4-6", "claude-opus-4-8"],
///     "Pick or type a model",
/// );
/// # }
/// ```
pub fn editable_combo(
    ui: &mut Ui,
    id_salt: impl Hash,
    text: &mut String,
    options: &[&str],
    hint: &str,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = GAP;
        let btn_w = metrics::CONTROL_HEIGHT;
        // egui draws a TextEdit `desired_width + margin.sum().x` wide (the
        // 8px-each-side margin is added ON TOP of desired_width), so reserve
        // that 16px too — otherwise field + gap + chevron overflow the row
        // and the chevron clips off the right edge.
        let field_w = (ui.available_width() - btn_w - GAP - FIELD_MARGIN_X).max(80.0);
        let edit = super::control::scope(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .hint_text(hint)
                    .margin(egui::Margin::symmetric(8, 6))
                    .min_size(egui::vec2(0.0, metrics::CONTROL_HEIGHT))
                    .desired_width(field_w),
            )
        });
        changed |= edit.changed();
        let btn = combo_arrow_button(ui, edit.rect.height());
        // The whole field is the trigger: clicking the field OR the chevron
        // toggles the menu. Anchor the popup to the field (not the narrow
        // chevron, which egui would clamp to a sliver) at the field's width;
        // the menu frame adds 6px each side, so subtract 12.
        let popup_w = (edit.rect.width() - 12.0).max(80.0);
        let toggled = edit.clicked() || btn.clicked();
        egui::Popup::menu(&btn)
            .anchor(&edit)
            .gap(4.0)
            .open_memory(toggled.then_some(egui::SetOpenCommand::Toggle))
            .id(ui.make_persistent_id((id_salt, "popup")))
            .width(popup_w)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                // Pin to the field width every frame (`.width()` only seeds
                // the default) and stop labels wrapping.
                ui.set_min_width(popup_w);
                ui.set_max_width(popup_w);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                if options.is_empty() {
                    ui.add_enabled(false, egui::Button::new("(none available)").frame(false));
                } else {
                    for opt in options {
                        if ui.selectable_label(text.as_str() == *opt, *opt).clicked() {
                            *text = (*opt).to_string();
                            changed = true;
                        }
                    }
                }
            });
    });
    changed
}

/// A combo drop button: a real kanso-bordered button (so its height,
/// rounding, and hover match the field beside it) with a downward chevron
/// painted on top — the bundled fonts lack the Geometric-Shapes glyph.
fn combo_arrow_button(ui: &mut Ui, height: f32) -> Response {
    super::control::scope(ui, |ui| {
        // `min_size` (not `add_sized`) so the button fills the cell rather
        // than centering an empty button and leaving dead space on the right.
        let resp =
            ui.add(egui::Button::new("").min_size(egui::vec2(metrics::CONTROL_HEIGHT, height)));
        let c = resp.rect.center();
        let stroke = egui::Stroke::new(1.6, palette::TEXT_MUTED);
        let painter = ui.painter();
        painter.line_segment(
            [c + egui::vec2(-4.0, -2.0), c + egui::vec2(0.0, 2.5)],
            stroke,
        );
        painter.line_segment(
            [c + egui::vec2(4.0, -2.0), c + egui::vec2(0.0, 2.5)],
            stroke,
        );
        resp
    })
}

/// Gap between the field and the chevron button.
const GAP: f32 = 4.0;
/// Horizontal margin egui's `TextEdit` adds on top of `desired_width`
/// (`Margin::symmetric(8, …)` → 8px each side).
const FIELD_MARGIN_X: f32 = 16.0;
