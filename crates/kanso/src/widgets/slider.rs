//! Labeled value slider.
//!
//! Both apps had a slider, and they pulled in opposite directions:
//!
//! - **hyprcorrect** (`prefs.rs:1617`) used a bare
//!   `egui::Slider::new(value, 0..=30).suffix(" ms")` wrapped in `ui.add`,
//!   introduced by a [`super::field_label`] + [`super::caption`]. Plain,
//!   idiomatic, integer-typed, with a unit suffix on the value readout.
//! - **vernier** (`prefs.rs:1778`, `tick_slider` + `tolerance_section`)
//!   hand-painted a slider from `Painter` primitives: a decorative tick
//!   "ruler", a free-moving circular knob, a progress rail, a fixed-width
//!   right-aligned label column, and a trailing monospace value readout —
//!   and crucially derived the knob colors from
//!   `visuals.widgets.{inactive,hovered,active}` so it matched the rest of
//!   the prefs theming.
//!
//! Reconciliation: kanso wraps `egui::Slider` (the tick ruler was
//! tolerance-specific decoration, not a reusable primitive — egui exposes
//! no tick API, which is the only reason vernier hand-rolled it). We keep
//! what was *shared and load-bearing*: the leading label, a trailing value
//! readout (vernier's monospace style, hyprcorrect's optional unit suffix),
//! and theming that tracks the cohort's control visuals. That last point
//! is why the slider runs inside [`super::control::scope`] — egui's slider
//! draws its handle from `widgets.{inactive,hovered,active}.bg_fill/stroke`
//! and its trailing fill from `selection.bg_fill`, so the scope gives the
//! handle the exact rest/hover/press border colors every other kanso
//! control uses, while the filled track reads in the cohort accent.
//!
//! Generic over `egui::emath::Numeric`, so it serves vernier's `u32`
//! tolerance values, hyprcorrect's integer millisecond pauses, and the
//! `f32` continuous case alike.

use egui::Response;
use egui::emath::Numeric;

use crate::{metrics, palette};

/// A labeled value slider: a left-aligned [`super::field_label`]-style
/// caption, the slider rail (filled in the cohort accent up to the knob),
/// and a trailing monospace value readout.
///
/// `value` is clamped to `range` by egui. Returns the slider's
/// [`Response`]; check [`Response::changed`] to react to edits.
///
/// For a unit suffix on the readout (e.g. `" ms"`, `"%"`) or custom
/// formatting, use [`Slider::new`] + [`Slider::show`].
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// let mut tolerance: f32 = 0.5;
/// kanso::widgets::slider(ui, &mut tolerance, 0.0..=1.0, "Tolerance");
/// # }
/// ```
pub fn slider<Num: Numeric>(
    ui: &mut egui::Ui,
    value: &mut Num,
    range: std::ops::RangeInclusive<Num>,
    label: &str,
) -> Response {
    Slider::new(value, range).label(label).show(ui)
}

/// Builder for [`slider`] with an optional unit suffix and custom value
/// formatting — reconciling hyprcorrect's `.suffix(" ms")` and vernier's
/// readout into one configurable readout.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::widgets::Slider;
/// let mut pause_ms: u32 = 8;
/// Slider::new(&mut pause_ms, 0..=30)
///     .label("Pause per backspace")
///     .suffix(" ms")
///     .show(ui);
/// # }
/// ```
pub struct Slider<'a, Num: Numeric> {
    value: &'a mut Num,
    range: std::ops::RangeInclusive<Num>,
    label: Option<&'a str>,
    suffix: String,
    formatter: Option<Box<dyn Fn(f64) -> String + 'a>>,
}

impl<'a, Num: Numeric> Slider<'a, Num> {
    /// Start a slider bound to `value`, clamped to `range`.
    pub fn new(value: &'a mut Num, range: std::ops::RangeInclusive<Num>) -> Self {
        Self {
            value,
            range,
            label: None,
            suffix: String::new(),
            formatter: None,
        }
    }

    /// Leading caption shown to the left of the rail.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Unit suffix appended to the value readout (e.g. `" ms"`, `"%"`).
    /// Ignored when a [`Self::custom_formatter`] is set.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Fully custom value readout, taking the current value as `f64`.
    /// Overrides [`Self::suffix`].
    pub fn custom_formatter(mut self, formatter: impl Fn(f64) -> String + 'a) -> Self {
        self.formatter = Some(Box::new(formatter));
        self
    }

    /// Lay the slider out and return its [`Response`].
    pub fn show(self, ui: &mut egui::Ui) -> Response {
        let Self {
            value,
            range,
            label,
            suffix,
            formatter,
        } = self;

        ui.horizontal(|ui| {
            // Vernier centered its label/readout against a 22px row; do
            // the same so a slider lines up with field labels stacked
            // above it and with sibling sliders.
            ui.set_min_height(LABEL_ROW_HEIGHT);

            if let Some(label) = label {
                ui.label(
                    egui::RichText::new(label)
                        .size(metrics::FIELD_LABEL_SIZE)
                        .color(palette::TEXT),
                );
                ui.add_space(LABEL_GAP);
            }

            // The slider's own value readout is the editable
            // drag-value box; we show our own trailing monospace
            // readout instead (vernier's style — muted, fixed font) so
            // the number reads as a label, not another input.
            let mut slider = egui::Slider::new(value, range.clone())
                .show_value(false)
                .trailing_fill(true);
            if let Some(ref fmt) = formatter {
                slider = slider.custom_formatter(move |n, _| fmt(n));
            }

            // `control::scope` gives the handle the cohort's rest/hover/
            // press border colors (it reads widgets.{inactive,hovered,
            // active}); the filled track uses selection.bg_fill (accent).
            let resp = super::control::scope(ui, |ui| ui.add(slider));

            ui.add_space(READOUT_GAP);
            let text = match &formatter {
                Some(fmt) => fmt(value.to_f64()),
                None => format!("{}{suffix}", format_value(value.to_f64())),
            };
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(metrics::CAPTION_SIZE)
                    .color(palette::TEXT_MUTED),
            );

            resp
        })
        .inner
    }
}

/// Height of the label/readout row — matches vernier's `tolerance_section`
/// 22px label band so a slider aligns with the field labels around it.
const LABEL_ROW_HEIGHT: f32 = 22.0;
/// Gap between the leading label and the rail (vernier used 12px).
const LABEL_GAP: f32 = 12.0;
/// Gap between the rail and the trailing value readout (vernier used 10px).
const READOUT_GAP: f32 = 10.0;

/// Default readout formatting: drop a trailing `.0` so integers read
/// cleanly (`8`, not `8.0`) while fractional values keep a compact two
/// decimals (`0.50`).
fn format_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.2}")
    }
}
