//! Small settings-pane helpers the apps repeated: field labels, section
//! headers, per-setting spacing, and padded text inputs.

use egui::{Response, Ui};

use crate::metrics;

/// Bold-ish label introducing a setting/field.
pub fn field_label(ui: &mut Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .strong()
            .size(metrics::FIELD_LABEL_SIZE),
    );
}

/// A section heading followed by consistent breathing room.
pub fn section_header(ui: &mut Ui, title: &str) {
    ui.heading(title);
    ui.add_space(metrics::SECTION_GAP);
}

/// Wrap a logical settings group in a vertical block followed by uniform
/// spacing, so per-setting code stays flat and the rhythm lives in one
/// place. Returns whatever the body returns.
pub fn setting<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
    let r = ui.vertical(content).inner;
    ui.add_space(metrics::SETTING_BLOCK_SPACING);
    r
}

/// Single-line text input with consistent inner padding so fields don't
/// collapse to ~16px tall.
pub fn padded_text_edit(ui: &mut Ui, text: &mut String) -> Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .margin(egui::Margin::symmetric(8, 6))
            .desired_width(f32::INFINITY),
    )
}
