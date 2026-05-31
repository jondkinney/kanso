//! The settings-window action bar: an unsaved-changes indicator plus
//! Save / Revert buttons gated on a dirty state.
//!
//! Both apps in the cohort grew the same pattern by hand and then drifted:
//!
//! - **vernier** ([`prefs.rs`] ~241/~573) holds two copies of one
//!   settings struct — `edited` and `saved` — and computes
//!   `dirty = edited != saved`. Its footer paints an 8px amber dot plus an
//!   "Unsaved changes" label (a *painter* circle, because egui's bundled
//!   proportional font has no `U+25CF`), then a right-aligned `Revert` +
//!   `Save`. `Save` is gated on `dirty`; `Revert` is enabled on
//!   `dirty || capturing_shortcut` so it can double as an "escape" out of
//!   an accidental chord capture.
//! - **hyprcorrect** (`prefs.rs` ~479/~704) spreads its working copy across
//!   several fields (`config`, `llm_keys`, `autostart_enabled`) each paired
//!   with a `saved_*` snapshot, ORs their inequalities into `dirty()`, and
//!   renders a right-aligned `Cancel` + `Save` both gated on `dirty()`, with
//!   no dot — just a status string.
//!
//! kanso reconciles these into two pieces:
//!
//! 1. [`Dirty`] — a tiny `edited` / `saved` snapshot pair over any
//!    `T: Clone + PartialEq`. This is vernier's model promoted to a reusable
//!    helper; hyprcorrect adopts it by folding its scattered `saved_*`
//!    fields into one `Settings` struct (or, where a field can't live in
//!    that struct, by composing extra dirtiness via [`DirtyFooter::dirty`]).
//! 2. [`dirty_footer`] / [`DirtyFooter`] — the footer widget. It paints
//!    vernier's font-independent amber dot + label when dirty and renders
//!    the right-aligned Revert + Save buttons, returning a [`FooterAction`]
//!    so the **app** keeps ownership of persistence (validation, keychain
//!    writes, daemon reloads) — the widget never touches disk.

use egui::{Response, Ui};

use crate::{metrics, palette};

/// A working copy paired with the last-saved snapshot of a settings value.
///
/// This is the canonical replacement for the apps' hand-rolled
/// `edited`/`saved` (vernier) and `config`/`saved_*` (hyprcorrect) field
/// pairs. Edit [`Dirty::edited`] freely; [`Dirty::is_dirty`] reports whether
/// it diverges from the snapshot, [`Dirty::mark_saved`] re-snapshots after a
/// successful write, and [`Dirty::revert`] discards edits.
///
/// ```
/// # use kanso::widgets::Dirty;
/// #[derive(Clone, PartialEq)]
/// struct Settings { name: String, precise: bool }
///
/// let mut d = Dirty::new(Settings { name: "Jon".into(), precise: false });
/// assert!(!d.is_dirty());
/// d.edited.precise = true;
/// assert!(d.is_dirty());
/// d.mark_saved(); // persisted → snapshot catches up
/// assert!(!d.is_dirty());
/// ```
#[derive(Clone, Debug, Default)]
pub struct Dirty<T> {
    /// The live, user-editable copy. Bind your widgets to this.
    pub edited: T,
    /// The last-saved snapshot. Updated by [`Dirty::mark_saved`]; treat as
    /// read-only otherwise.
    saved: T,
}

impl<T: Clone + PartialEq> Dirty<T> {
    /// Wrap an initial value; `edited` and `saved` start equal (not dirty).
    pub fn new(value: T) -> Self {
        Self {
            edited: value.clone(),
            saved: value,
        }
    }

    /// `true` when the working copy diverges from the last-saved snapshot.
    /// This is the gate for the Save button.
    pub fn is_dirty(&self) -> bool {
        self.edited != self.saved
    }

    /// The last-saved snapshot (read-only). Useful for diffing which fields
    /// changed when persisting (hyprcorrect only writes the keychain entries
    /// that actually differ).
    pub fn saved(&self) -> &T {
        &self.saved
    }

    /// Re-snapshot after a **successful** persist: `saved = edited`. Call
    /// this from your save path *after* the write succeeds, so a failed
    /// write leaves the footer dirty.
    pub fn mark_saved(&mut self) {
        self.saved = self.edited.clone();
    }

    /// Discard edits: `edited = saved`. Backs the Revert button.
    pub fn revert(&mut self) {
        self.edited = self.saved.clone();
    }
}

/// What the user asked the footer to do this frame.
///
/// The widget performs no persistence itself — it returns intent and the app
/// runs its own save/revert (which may validate, write a keychain, reload a
/// daemon, etc.), then calls [`Dirty::mark_saved`] / [`Dirty::revert`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FooterAction {
    /// No button was pressed this frame.
    None,
    /// The user pressed Save (only possible while dirty).
    Save,
    /// The user pressed Revert.
    Revert,
}

impl FooterAction {
    /// `true` if any button was pressed (i.e. not [`FooterAction::None`]).
    pub fn is_some(self) -> bool {
        self != FooterAction::None
    }
}

/// The action-bar widget. Construct with [`dirty_footer`], optionally tune
/// it with the builder methods, then [`DirtyFooter::show`] it inside a
/// bottom panel.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::widgets::{Dirty, FooterAction, dirty_footer};
/// # #[derive(Clone, PartialEq, Default)] struct Settings;
/// # let mut settings: Dirty<Settings> = Dirty::default();
/// # fn persist(_: &Settings) {}
/// match dirty_footer(&settings).show(ui) {
///     FooterAction::Save => {
///         persist(&settings.edited);
///         settings.mark_saved();
///     }
///     FooterAction::Revert => settings.revert(),
///     FooterAction::None => {}
/// }
/// # }
/// ```
#[must_use = "call `.show(ui)` to render the footer and read the action"]
pub struct DirtyFooter {
    dirty: bool,
    revert_enabled: bool,
    save_label: &'static str,
    revert_label: &'static str,
    unsaved_label: &'static str,
}

/// Build a [`DirtyFooter`] from a [`Dirty`] snapshot pair — the common case.
/// Save and the unsaved indicator are gated on `state.is_dirty()`.
///
/// For settings spread across more than one [`Dirty`] (hyprcorrect's
/// keychain + autostart), build with [`DirtyFooter::from_dirty`] and OR in
/// the extra dirtiness via [`DirtyFooter::dirty`].
pub fn dirty_footer<T: Clone + PartialEq>(state: &Dirty<T>) -> DirtyFooter {
    DirtyFooter::from_dirty(state.is_dirty())
}

impl DirtyFooter {
    /// Build directly from a precomputed dirty flag — for callers whose
    /// "dirty" is the OR of several sources (extra unsaved keys, an
    /// autostart toggle) and so can't come from a single [`Dirty`].
    pub fn from_dirty(dirty: bool) -> Self {
        Self {
            dirty,
            revert_enabled: dirty,
            save_label: "Save",
            revert_label: "Revert",
            unsaved_label: "Unsaved changes",
        }
    }

    /// OR additional dirtiness into the footer (e.g. an out-of-struct field
    /// hyprcorrect tracks separately). Affects the Save gate, the indicator,
    /// and — unless overridden — the Revert gate.
    pub fn dirty(mut self, also_dirty: bool) -> Self {
        let was = self.revert_enabled == self.dirty;
        self.dirty |= also_dirty;
        // Keep Revert tracking dirty unless the caller force-enabled it.
        if was {
            self.revert_enabled = self.dirty;
        }
        self
    }

    /// Force-enable Revert even when not dirty. vernier enables Revert while
    /// a shortcut row is capturing so it can serve as an "escape" from an
    /// accidental capture click (`dirty || capturing_shortcut.is_some()`).
    pub fn revert_enabled(mut self, enabled: bool) -> Self {
        self.revert_enabled = self.dirty || enabled;
        self
    }

    /// Override the Save button label (default `"Save"`).
    pub fn save_label(mut self, label: &'static str) -> Self {
        self.save_label = label;
        self
    }

    /// Override the Revert button label. hyprcorrect calls it `"Cancel"`;
    /// the canonical default is vernier's `"Revert"`.
    pub fn revert_label(mut self, label: &'static str) -> Self {
        self.revert_label = label;
        self
    }

    /// Override the unsaved-indicator label (default `"Unsaved changes"`).
    pub fn unsaved_label(mut self, label: &'static str) -> Self {
        self.unsaved_label = label;
        self
    }

    /// Render the action row into the current `ui` (typically inside an
    /// [`egui::TopBottomPanel::bottom`]) and return the user's intent.
    ///
    /// Buttons and the indicator are right-aligned, leaving the left of the
    /// row free for app-specific extras (a Quit button, a "Relaunch daemon"
    /// nudge, a status string) that you lay out *before* calling `show`.
    pub fn show(self, ui: &mut Ui) -> FooterAction {
        let mut action = FooterAction::None;
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Right-to-left: Save sits at the far right, then Revert, then
            // the unsaved dot + label — matching both apps' footer order.
            if save_button(ui, self.save_label, self.dirty).clicked() {
                action = FooterAction::Save;
            }
            ui.add_space(8.0);
            if ui
                .add_enabled(self.revert_enabled, egui::Button::new(self.revert_label))
                .clicked()
            {
                action = FooterAction::Revert;
            }
            if self.dirty {
                ui.add_space(8.0);
                unsaved_indicator(ui, self.unsaved_label);
            }
        });
        action
    }
}

/// The primary Save button: filled accent when actionable (dirty), and the
/// neutral disabled treatment otherwise. Uses kanso's primary-button fill so
/// it reads as the call-to-action the way the rest of the system does.
fn save_button(ui: &mut Ui, label: &str, enabled: bool) -> Response {
    if !enabled {
        return ui.add_enabled(false, egui::Button::new(label));
    }
    let base = ui.visuals().selection.bg_fill;
    let hover = lighten(base, 0.16);
    let text_color = ui.visuals().selection.stroke.color;
    ui.scope(|ui| {
        let radius = egui::CornerRadius::same(metrics::RADIUS);
        let w = &mut ui.visuals_mut().widgets;
        for (state, fill) in [
            (&mut w.inactive, base),
            (&mut w.hovered, hover),
            (&mut w.active, hover),
        ] {
            state.weak_bg_fill = fill;
            state.bg_fill = fill;
            state.bg_stroke = egui::Stroke::new(metrics::BORDER_WIDTH, fill);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }
        ui.add(egui::Button::new(
            egui::RichText::new(label).color(text_color).strong(),
        ))
    })
    .inner
}

/// The "unsaved changes" pip: a painter-drawn amber dot + label.
///
/// The dot is painted (not a glyph) because egui's bundled proportional font
/// carries no `U+25CF` and renders tofu — vernier hit this and switched to a
/// `circle_filled`, which kanso adopts. Laid out right-to-left, so the label
/// is allocated first and the dot lands to its left.
fn unsaved_indicator(ui: &mut Ui, label: &str) {
    ui.colored_label(palette::WARN, label);
    ui.add_space(2.0);
    let (rect, _) = ui.allocate_exact_size(UNSAVED_DOT_SIZE, egui::Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), UNSAVED_DOT_RADIUS, palette::WARN);
}

/// Blend a color toward white by `t` (0..=1) — mirrors [`super::buttons`]'
/// primary-button hover so the Save button lightens identically.
fn lighten(c: egui::Color32, t: f32) -> egui::Color32 {
    let mix = |x: u8| (x as f32 + (255.0 - x as f32) * t).round() as u8;
    egui::Color32::from_rgb(mix(c.r()), mix(c.g()), mix(c.b()))
}

/// Allocated box for the unsaved dot (vernier used an 8px square).
const UNSAVED_DOT_SIZE: egui::Vec2 = egui::vec2(8.0, 8.0);
/// Radius of the painted unsaved dot.
const UNSAVED_DOT_RADIUS: f32 = 4.0;
