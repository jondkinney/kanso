//! Internal: apply the input/button control treatment in a **scope** so it
//! only affects the wrapped widget — not sibling dropdown items, nav rows,
//! checkboxes, etc. The treatment itself lives in
//! [`crate::theme::control_visuals`] (also public, for apps that want it
//! applied globally).

use egui::Ui;

/// Run `add` with the control visuals applied locally (restored after).
pub(crate) fn scope<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        crate::theme::control_visuals(ui.visuals_mut());
        add(ui)
    })
    .inner
}
