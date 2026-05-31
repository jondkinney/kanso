//! Inline severity callout strip — a tinted, rounded banner with an icon,
//! a bold title, a muted body, and an optional action button.
//!
//! Both apps hand-rolled this same shape as an `egui::Frame`:
//!
//! * vernier's "⚠ Screen Recording is off" permission banner
//!   (`prefs.rs:673`) and "⚠ Static bind detected" warning
//!   (`prefs.rs:2108`) — a `Frame::NONE` with a dark severity-tinted fill,
//!   a saturated severity stroke, `corner_radius(8)`, symmetric inner
//!   margin, then a strong colored title, a muted body, and a button.
//! * hyprcorrect's review/notice cards (`review.rs:1025`, `1490`) — the
//!   same `Frame` shape (rounded `8`, symmetric `(14, 12)` margin) used as
//!   a contained block, plus its inline LLM/Docker status lines
//!   (`prefs.rs:740`, `1395`) which color the text by severity.
//!
//! Reconciliation: vernier inlined per-severity dark fills (`rgb(60,32,32)`,
//! `rgb(60,48,16)`) and matching strokes. Rather than carry four hand-mixed
//! dark literals, kanso *derives* the strip from one source of truth — the
//! severity color ([`palette::INFO`]/[`WARN`]/[`ERROR`]/[`OK`]) — by blending
//! it over the [`palette::CARD`] surface at a low alpha for the fill and a
//! higher alpha for the stroke. The title/body/spacing literals both apps
//! repeated are lifted to [`crate::metrics`]. The action button reuses the
//! shared [`super::control`] border treatment so it matches every other
//! neutral button in the system.

use egui::{Response, Ui};

use crate::{metrics, palette};

/// Severity of a [`banner`], selecting its accent color and default icon.
///
/// Maps one-to-one onto the cohort's status palette: vernier's amber
/// "static bind" / red "recording off" notices and hyprcorrect's
/// amber Docker / red error status lines all collapse onto these four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Neutral information — [`palette::INFO`].
    Info,
    /// Caution, still functional — [`palette::WARN`].
    Warn,
    /// Something is broken / blocked — [`palette::ERROR`].
    Error,
    /// Success / healthy — [`palette::OK`].
    Ok,
}

impl Severity {
    /// The accent color for this severity — the single source the fill,
    /// stroke, and title color are all derived from.
    pub fn color(self) -> egui::Color32 {
        match self {
            Severity::Info => palette::INFO,
            Severity::Warn => palette::WARN,
            Severity::Error => palette::ERROR,
            Severity::Ok => palette::OK,
        }
    }

    /// The default leading glyph, prepended to the title (matching
    /// vernier's `⚠ …` titles). All four use glyphs present in the
    /// cohort's bundled proportional font.
    pub fn icon(self) -> &'static str {
        match self {
            Severity::Info => "ⓘ",
            Severity::Warn => "⚠",
            Severity::Error => "⚠",
            Severity::Ok => "✓",
        }
    }
}

/// A tinted severity callout. Paints an icon + strong [`Severity`]-colored
/// `title` over a muted `body`, inside a rounded strip tinted toward the
/// severity color. Returns the [`Response`] for the whole strip (hover it
/// for a tooltip, etc.). For a strip with a trailing action button use
/// [`banner_action`].
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::widgets::{banner, Severity};
/// banner(
///     ui,
///     Severity::Warn,
///     "Screen Recording is off",
///     "Vernier can track the cursor but can't snap to pixels until it's granted.",
/// );
/// # }
/// ```
pub fn banner(ui: &mut Ui, severity: Severity, title: &str, body: &str) -> Response {
    strip(ui, severity, title, body, None).response
}

/// A [`banner`] with a trailing neutral action button. Returns `true` on
/// the frame the button is clicked (e.g. "Open System Settings").
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// use kanso::widgets::{banner_action, Severity};
/// if banner_action(
///     ui,
///     Severity::Error,
///     "LLM unavailable",
///     "No API key is configured for the selected provider.",
///     "Open Providers",
/// ) {
///     // navigate to the providers pane…
/// }
/// # }
/// ```
pub fn banner_action(
    ui: &mut Ui,
    severity: Severity,
    title: &str,
    body: &str,
    action_label: &str,
) -> bool {
    strip(ui, severity, title, body, Some(action_label))
        .clicked
        .unwrap_or(false)
}

/// What [`strip`] hands back: the whole-strip response plus, when an
/// action label was supplied, whether that button was clicked this frame.
struct Painted {
    response: Response,
    clicked: Option<bool>,
}

/// Shared body for both public entry points: paint the tinted [`egui::Frame`]
/// and lay out icon + title / body / optional action inside it.
fn strip(
    ui: &mut Ui,
    severity: Severity,
    title: &str,
    body: &str,
    action_label: Option<&str>,
) -> Painted {
    let accent = severity.color();
    // Derive the fill and stroke from the one accent color so a palette
    // tweak restyles all four severities coherently (vs. vernier's four
    // hand-mixed dark literals). Both are the accent blended over the card
    // surface — a faint wash for the fill, a clearer edge for the stroke.
    let fill = palette::CARD.blend(accent.gamma_multiply(metrics::BANNER_FILL_ALPHA));
    let stroke_color = palette::CARD.blend(accent.gamma_multiply(metrics::BANNER_STROKE_ALPHA));

    let mut clicked = None;
    let inner = egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(metrics::BORDER_WIDTH, stroke_color))
        .corner_radius(egui::CornerRadius::same(metrics::BANNER_CORNER))
        .inner_margin(egui::Margin::symmetric(
            metrics::BANNER_PAD_X,
            metrics::BANNER_PAD_Y,
        ))
        .show(ui, |ui| {
            // Fill the available width so the strip spans the pane like the
            // apps' top-panel / card banners, rather than hugging the text.
            ui.set_min_width(ui.available_width());

            ui.label(
                egui::RichText::new(format!("{} {title}", severity.icon()))
                    .color(accent)
                    .size(metrics::BANNER_TITLE_SIZE)
                    .strong(),
            );

            if !body.is_empty() {
                ui.add_space(metrics::BANNER_TITLE_GAP);
                // A plain wrapping label (not inside `ui.horizontal`, which
                // hands out unbounded width) so the body wraps at the strip
                // edge — the exact bug vernier's banner comment calls out.
                ui.label(
                    egui::RichText::new(body)
                        .color(palette::TEXT_MUTED)
                        .size(metrics::BANNER_BODY_SIZE),
                );
            }

            if let Some(label) = action_label {
                ui.add_space(metrics::BANNER_ACTION_GAP);
                clicked = Some(super::secondary_button(ui, label).clicked());
            }
        });

    Painted {
        response: inner.response,
        clicked,
    }
}
