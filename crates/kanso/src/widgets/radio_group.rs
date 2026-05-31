//! Single-select groups — classic radios or a segmented control — for
//! picking one of N options, generic over a value enum.
//!
//! This is the always-visible sibling of [`super::dropdown`]: same
//! `(value, label)` option model and same `&mut T -> bool` (changed)
//! contract, but every option stays on screen instead of hiding behind a
//! popup. Reach for [`radio_group`] when there are a handful of options and
//! the choice benefits from being scannable at a glance; reach for
//! [`super::dropdown`] when the list is long or space is tight.
//!
//! ## Reconciling the two apps
//!
//! Both apps hand-rolled this against the bare egui primitives:
//!
//! * **vernier** (`prefs.rs`) stacks `ui.radio_value(...)` vertically for
//!   "Clipboard format", "Units", and "Aspect ratio" — classic radio
//!   buttons, one per line.
//! * **hyprcorrect** (`prefs.rs`) has *both*: `provider_radio` lays
//!   `ui.radio_value(...)` out **horizontally** in a row (Spellbook /
//!   LanguageTool / LLM), while the "Word definitions" picker uses
//!   `ui.selectable_value(...)` as a **segmented control** (Offline /
//!   Online / Off).
//!
//! Rather than bless one and drop the others, kanso exposes the same
//! single-select group in all three layouts behind one option model:
//!
//! * [`radio_group`] — vertical radios (vernier's stacked form, the
//!   default).
//! * [`radio_group_horizontal`] — radios in a row (hyprcorrect's
//!   `provider_radio`).
//! * [`segmented`] — a segmented control (hyprcorrect's `selectable_value`
//!   row), framed with the shared control border so it reads like the rest
//!   of the design system.
//!
//! All three take `&[(T, &str)]` and return `true` when the selection
//! changed this frame, matching [`super::dropdown`] so callers can swap
//! between popup and always-visible variants without reshaping state.

use egui::{Response, Ui};

// Note: `metrics`/`palette` tokens are reused indirectly via
// `super::control::scope` (the segmented control's border treatment) and
// referenced by intra-doc links below; this module adds no inline literals
// of its own, so it needs no direct token imports.

/// Vertical radio group: one classic [`egui::RadioButton`] per line.
///
/// Mutates `selected` in place; returns `true` if the selection changed
/// this frame. The default single-select layout — prefer it when the
/// labels are long enough to want their own line (vernier's "Clipboard
/// format" / "Units" / "Aspect ratio").
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// #[derive(PartialEq, Clone)]
/// enum Units { Points, Pixels }
/// let mut units = Units::Points;
/// kanso::widgets::radio_group(
///     ui,
///     &mut units,
///     &[(Units::Points, "Points (logical)"), (Units::Pixels, "Screen pixels")],
/// );
/// # }
/// ```
pub fn radio_group<T: PartialEq + Clone>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    for (value, label) in options {
        if radio_value(ui, selected, value, label) {
            changed = true;
        }
    }
    changed
}

/// Horizontal radio group: the same classic radios as [`radio_group`] laid
/// out in a single row (hyprcorrect's `provider_radio`). Mutates
/// `selected`; returns `true` if the selection changed this frame.
///
/// Prefer this when the labels are short and reading them side by side
/// communicates "these are alternatives" (Spellbook / LanguageTool / LLM).
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// #[derive(PartialEq, Clone)]
/// enum Provider { Offline, Hosted }
/// let mut provider = Provider::Offline;
/// kanso::widgets::radio_group_horizontal(
///     ui,
///     &mut provider,
///     &[(Provider::Offline, "Offline"), (Provider::Hosted, "Hosted")],
/// );
/// # }
/// ```
pub fn radio_group_horizontal<T: PartialEq + Clone>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    ui.horizontal(|ui| {
        let mut changed = false;
        for (value, label) in options {
            if radio_value(ui, selected, value, label) {
                changed = true;
            }
        }
        changed
    })
    .inner
}

/// Segmented control: a row of joined selectable buttons, one per option,
/// with the active option filled in the theme's selection color
/// (hyprcorrect's `selectable_value` "Word definitions" picker). Mutates
/// `selected`; returns `true` if the selection changed this frame.
///
/// The segments share the scoped control border treatment (matched at
/// rest, [`crate::palette::BORDER`] on hover) so the group reads as one framed
/// control rather than three loose buttons. Prefer this over
/// [`radio_group_horizontal`] for compact, mutually-exclusive toggles
/// where a filled "pressed" segment communicates the choice better than a
/// radio dot (Offline / Online / Off).
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// #[derive(PartialEq, Clone)]
/// enum Definitions { Offline, Online, Off }
/// let mut defs = Definitions::Offline;
/// kanso::widgets::segmented(
///     ui,
///     &mut defs,
///     &[
///         (Definitions::Offline, "Offline"),
///         (Definitions::Online, "Online"),
///         (Definitions::Off, "Off"),
///     ],
/// );
/// # }
/// ```
pub fn segmented<T: PartialEq + Clone>(
    ui: &mut Ui,
    selected: &mut T,
    options: &[(T, &str)],
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        // Each segment is a real kanso button so it gets the same
        // constant-geometry control treatment as every other button: the
        // selected one is filled with the selection teal (like the primary
        // button), the rest are neutral with the matched control border.
        // Nothing grows on hover.
        for (value, label) in options {
            let is_selected = *selected == *value;
            let response = if is_selected {
                super::primary_button(ui, *label)
            } else {
                super::secondary_button(ui, *label)
            };
            if response.clicked() && !is_selected {
                *selected = value.clone();
                changed = true;
            }
        }
    });
    changed
}

/// One classic radio row: paints an [`egui::RadioButton`] for `value`,
/// flips `selected` to it on click, and reports whether it changed. Shared
/// by [`radio_group`] and [`radio_group_horizontal`]; kept private so the
/// public surface stays the option-list form.
///
/// This mirrors `egui::Ui::radio_value` but returns a plain `bool` (changed
/// this frame) instead of a [`Response`], matching the rest of kanso's
/// `&mut T -> bool` selectors.
fn radio_value<T: PartialEq + Clone>(
    ui: &mut Ui,
    selected: &mut T,
    value: &T,
    label: &str,
) -> bool {
    let is_selected = *selected == *value;
    let response: Response = ui.radio(is_selected, label);
    if response.clicked() && !is_selected {
        *selected = value.clone();
        true
    } else {
        false
    }
}
