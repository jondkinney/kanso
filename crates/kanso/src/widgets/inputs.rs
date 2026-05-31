//! Text search field and a typed dropdown wrapper over `ComboBox`.

use std::hash::Hash;

use egui::{Response, Ui};

/// Full-width single-line search/filter field with a hint.
pub fn search_field(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    )
}

/// A labeled dropdown over a fixed set of `(value, label)` options.
/// Mutates `selected` in place; returns `true` if the selection changed.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// #[derive(PartialEq, Clone)]
/// enum Mode { Fast, Precise }
/// let mut mode = Mode::Fast;
/// kanso::widgets::dropdown(
///     ui,
///     "mode",
///     &mut mode,
///     &[(Mode::Fast, "Fast"), (Mode::Precise, "Precise")],
/// );
/// # }
/// ```
pub fn dropdown<T: PartialEq + Clone>(
    ui: &mut Ui,
    id_salt: impl Hash,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let current = options
        .iter()
        .find(|(v, _)| *v == *selected)
        .map(|(_, l)| *l)
        .unwrap_or("—");
    let mut changed = false;
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(current)
        .show_ui(ui, |ui| {
            for (value, label) in options {
                if ui.selectable_label(*selected == *value, *label).clicked() {
                    *selected = value.clone();
                    changed = true;
                }
            }
        });
    changed
}
