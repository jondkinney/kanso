//! Progress bar.
//!
//! hyprcorrect's prefs window shows the n-gram corpus download with a bare
//! `egui::ProgressBar::new(frac).text(text)` and a faux-full
//! `ProgressBar::new(1.0).text("Unzipping…")` standing in for the
//! indeterminate unzip phase (egui's bar has no real busy state). kanso
//! reconciles those into one styled bar: the cohort [`palette::ACCENT`]
//! fill on a [`palette::CONTROL_BG`] track at the standard
//! [`metrics::RADIUS`], an optional centered overlay label, and a proper
//! animated [`ProgressBar::indeterminate`] mode for "busy, no measurable
//! progress" states.
//!
//! It is hand-painted (rather than wrapping `egui::ProgressBar`) so the
//! fill, track, rounding, and text all read in kanso tokens regardless of
//! the ambient `Visuals`, exactly like [`super::shortcut_chip`].

use egui::{Response, Sense, Vec2};

use crate::{metrics, palette};

/// A determinate progress bar: a full-width [`palette::CONTROL_BG`] track
/// filled to `fraction` (clamped to `0.0..=1.0`) in the cohort
/// [`palette::ACCENT`], with `text` centered over it.
///
/// Self-sizing to the available width; advances the cursor and returns the
/// hover [`Response`].
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::widgets::progress(ui, 0.42, "Downloading 6.7 / 16.0 GB");
/// # }
/// ```
pub fn progress(ui: &mut egui::Ui, fraction: f32, text: &str) -> Response {
    ProgressBar::new(fraction).text(text).show(ui)
}

/// Builder for [`progress`] — set an overlay [`Self::text`], override the
/// [`Self::height`], or switch to an animated [`Self::indeterminate`] busy
/// bar for work with no measurable fraction.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::widgets::ProgressBar;
/// // measurable download
/// ProgressBar::new(0.6).text("Downloading…").show(ui);
/// // busy phase with no fraction
/// ProgressBar::indeterminate().text("Unzipping (~16 GB)…").show(ui);
/// # }
/// ```
pub struct ProgressBar {
    fraction: f32,
    text: Option<String>,
    height: f32,
    indeterminate: bool,
}

impl ProgressBar {
    /// A determinate bar filled to `fraction` (clamped to `0.0..=1.0`).
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction,
            text: None,
            height: metrics::PROGRESS_HEIGHT,
            indeterminate: false,
        }
    }

    /// An indeterminate "busy" bar: ignores the fraction and sweeps an
    /// accent segment across the track, requesting repaints so it animates.
    /// Use for work whose total isn't known (unzipping, probing, waiting).
    pub fn indeterminate() -> Self {
        Self {
            fraction: 0.0,
            text: None,
            height: metrics::PROGRESS_HEIGHT,
            indeterminate: true,
        }
    }

    /// Label centered over the bar (e.g. `"Downloading 6.7 / 16.0 GB"`).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Override the bar height (default [`metrics::PROGRESS_HEIGHT`]). Drop
    /// to a few px for a slim, label-less rail.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Lay the bar out across the available width and return its hover
    /// [`Response`].
    pub fn show(self, ui: &mut egui::Ui) -> Response {
        let Self {
            fraction,
            text,
            height,
            indeterminate,
        } = self;

        let width = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return resp;
        }

        let painter = ui.painter();
        let radius = egui::CornerRadius::same(metrics::RADIUS);
        painter.rect_filled(rect, radius, palette::CONTROL_BG);

        if indeterminate {
            // A segment ~30% of the track width sweeps left→right→left on a
            // cosine ease, so it slows at each end. Animating means we must
            // keep asking egui to repaint.
            let t = ui.input(|i| i.time) as f32;
            ui.ctx().request_repaint();
            let seg_w = (rect.width() * 0.3).max(metrics::RADIUS as f32 * 2.0);
            let phase = (t / INDETERMINATE_PERIOD).fract();
            let eased = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
            let max_x = (rect.width() - seg_w).max(0.0);
            let x = rect.left() + eased * max_x;
            let seg = egui::Rect::from_min_size(
                egui::pos2(x, rect.top()),
                Vec2::new(seg_w, rect.height()),
            );
            ui.painter().rect_filled(seg, radius, palette::ACCENT);
        } else {
            let frac = if fraction.is_finite() {
                fraction.clamp(0.0, 1.0)
            } else {
                0.0
            };
            let fill_w = rect.width() * frac;
            if fill_w > 0.0 {
                let fill = egui::Rect::from_min_size(rect.min, Vec2::new(fill_w, rect.height()));
                ui.painter().rect_filled(fill, radius, palette::ACCENT);
            }
        }

        if let Some(text) = text {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(PROGRESS_TEXT_SIZE),
                palette::TEXT,
            );
        }

        resp
    }
}

/// Seconds for one full sweep of the indeterminate segment (out and back).
const INDETERMINATE_PERIOD: f32 = 1.6;
/// Overlay label size — small enough to sit comfortably inside the bar.
const PROGRESS_TEXT_SIZE: f32 = 12.5;
