//! Compact numeric input with `−` / `+` stepper buttons flanking an
//! editable [`egui::DragValue`].
//!
//! This reconciles the two ad-hoc numeric controls the apps grew:
//!
//! - **vernier** used a bare `DragValue` with a clamped `.range(…)` and a
//!   unit `.suffix(…)` for integer settings — base font size
//!   (`prefs.rs:1186`) and screenshot context margin (`prefs.rs:1344`) —
//!   each manually casting `u32 → i32`, clamping with `.max(…)`, and
//!   reacting to `.changed()`.
//! - **hyprcorrect** used an `egui::Slider` with `.range(0..=30)` and a
//!   `" ms"` suffix for "pause per backspace" (`prefs.rs:1618`), also
//!   reacting to `.changed()`.
//!
//! Both want the *same thing*: a bounded numeric field with a unit suffix
//! and a coarse step, that lines up with the other controls and reports
//! changes. kanso settles on the **stepper** shape the task calls for —
//! `−`/`+` buttons around a click-to-edit `DragValue` — which keeps
//! vernier's compactness and adds the discrete-bump affordance
//! hyprcorrect got from its slider, without consuming a slider's full
//! row width. The whole control rides [`super::control::scope`] so the
//! field border matches every other input (color-matched at rest,
//! [`palette::BORDER`] on hover, never changing height).
//!
//! It is generic over [`egui::emath::Numeric`], so the one widget serves
//! vernier's `i32`/`u32`-derived fields and any `f32`/`f64` setting a
//! future app needs — a superset of both apps' integer-only call sites.

use egui::emath::Numeric;
use egui::{Response, Ui};

use crate::{metrics, palette};

/// Configurable numeric stepper. Build with [`NumberStepper::new`], then
/// [`show`](NumberStepper::show); or reach for the [`number_stepper`] free
/// function for the common case.
pub struct NumberStepper<Num: Numeric> {
    range: std::ops::RangeInclusive<Num>,
    step: Num,
    suffix: Option<String>,
    /// Drag speed for the central `DragValue`. Defaults to `step`, which
    /// matches the apps' "one notch per unit" feel for integer settings.
    speed: f64,
}

impl<Num: Numeric> NumberStepper<Num> {
    /// A stepper bounded to `range`, bumping by `step` per button press.
    ///
    /// `step` doubles as the default drag speed of the editable middle
    /// field; override it with [`speed`](Self::speed) if you want finer
    /// dragging than the button increment (e.g. a coarse `+1.0` button
    /// but smooth `0.1`-per-pixel dragging on a float).
    pub fn new(range: std::ops::RangeInclusive<Num>, step: Num) -> Self {
        Self {
            range,
            step,
            suffix: None,
            speed: step.to_f64(),
        }
    }

    /// A trailing unit string shown after the number, e.g. `" px"` /
    /// `" ms"` (mirrors the apps' `DragValue::suffix` / `Slider::suffix`).
    /// The leading space is the caller's, matching egui's convention.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Override the drag speed of the editable middle field (defaults to
    /// the button `step`).
    pub fn speed(mut self, speed: impl Into<f64>) -> Self {
        self.speed = speed.into();
        self
    }

    /// Render the stepper bound to `value`, returning the unioned
    /// [`Response`] of the three parts. `response.changed()` is true on
    /// any edit — a `−`/`+` press or a drag/keyboard edit of the field —
    /// so callers react exactly like they did to the bare `DragValue` /
    /// `Slider` (e.g. clearing a status line).
    pub fn show(self, ui: &mut Ui, value: &mut Num) -> Response {
        let Self {
            range,
            step,
            suffix,
            speed,
        } = self;
        let (lo, hi) = (*range.start(), *range.end());

        super::control::scope(ui, |ui| {
            // Tighten the inter-widget gap so the buttons hug the field
            // and the trio reads as one control rather than three.
            ui.spacing_mut().item_spacing.x = 4.0;
            // Square-ish stepper buttons at the shared control height, so
            // the row lines up with text inputs / dropdowns beside it.
            let btn = egui::vec2(metrics::CONTROL_HEIGHT, metrics::CONTROL_HEIGHT);

            ui.horizontal(|ui| {
                // `−` decrements; disabled once already at the floor.
                let dec = ui.add_enabled(
                    value.to_f64() > lo.to_f64(),
                    egui::Button::new(egui::RichText::new("−").color(palette::TEXT)).min_size(btn),
                );
                if dec.clicked() {
                    *value = clamp(Num::from_f64(value.to_f64() - step.to_f64()), lo, hi);
                }

                let mut field = egui::DragValue::new(value)
                    .range(lo..=hi)
                    .speed(speed)
                    .update_while_editing(false);
                if let Some(suffix) = &suffix {
                    field = field.suffix(suffix);
                }
                let mut resp = ui.add(field);

                // `+` increments; disabled once already at the ceiling.
                let inc = ui.add_enabled(
                    value.to_f64() < hi.to_f64(),
                    egui::Button::new(egui::RichText::new("+").color(palette::TEXT)).min_size(btn),
                );
                if inc.clicked() {
                    *value = clamp(Num::from_f64(value.to_f64() + step.to_f64()), lo, hi);
                }

                // Fold the button clicks into the field's response so a
                // single `.changed()` covers every way the value moved.
                if dec.clicked() || inc.clicked() {
                    resp.mark_changed();
                }
                resp.union(dec).union(inc)
            })
            .inner
        })
    }
}

/// Compact numeric stepper for an integer/float setting: a click-to-edit
/// field bounded to `range`, with `−`/`+` buttons that bump by `step`.
///
/// The value is clamped into `range`, and the returned [`Response`]'s
/// `.changed()` fires on any edit. For a unit suffix or a custom drag
/// speed, use the [`NumberStepper`] builder.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// // vernier's "context margin", as a stepper:
/// let mut padding_px: u32 = 8;
/// if kanso::widgets::number_stepper(ui, &mut padding_px, 0..=64, 1).changed() {
///     // persist…
/// }
/// // a float setting with the builder (suffix + smooth dragging):
/// let mut gain: f32 = 1.0;
/// kanso::widgets::NumberStepper::new(0.0..=4.0, 0.25)
///     .suffix("×")
///     .speed(0.05)
///     .show(ui, &mut gain);
/// # }
/// ```
pub fn number_stepper<Num: Numeric>(
    ui: &mut Ui,
    value: &mut Num,
    range: std::ops::RangeInclusive<Num>,
    step: Num,
) -> Response {
    NumberStepper::new(range, step).show(ui, value)
}

/// Clamp `v` into `[lo, hi]` without requiring `Num: Ord` (floats only
/// implement `PartialOrd`); works for every [`Numeric`] type.
fn clamp<Num: Numeric>(v: Num, lo: Num, hi: Num) -> Num {
    let x = v.to_f64().clamp(lo.to_f64(), hi.to_f64());
    Num::from_f64(x)
}
