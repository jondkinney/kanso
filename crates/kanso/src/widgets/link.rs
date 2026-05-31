//! Accent-colored text links — the navigation / "open this" links both
//! apps render.
//!
//! Both apps draw two flavors of link that this module reconciles into
//! one look:
//!
//! * **URL links** that open a web page — vernier's About-tab
//!   `hyperlink_to("github.com/jondkinney/vernier", …)` (`prefs.rs:2917`)
//!   and hyprcorrect's `hyperlink_to`/`hyperlink` for the Docker image
//!   and GitHub source (`prefs.rs:1494`, `prefs.rs:1904`). Both delegate
//!   to egui's stock [`egui::Hyperlink`], which paints accent text,
//!   underlines on hover, shows a pointing-hand cursor, and calls
//!   `ctx().open_url(..)` on click.
//! * **Action links** whose click does something other than open a URL —
//!   vernier's "Open System Settings" call-to-action (`prefs.rs:700`,
//!   rendered as a plain button) and its hand-rolled, accent + underlined
//!   clickable label for the settings-file path (`prefs.rs:2957`) that
//!   runs `open_path_with_default_app`.
//!
//! kanso unifies the *look* (egui's `Link`: accent text from
//! [`palette::ACCENT`], underline only on hover/focus, pointing-hand
//! cursor) and lets the **caller own the side effect** — exactly how
//! vernier's "Open System Settings" and settings-path links work. So
//! [`link`] returns a plain [`egui::Response`]; the caller runs its
//! action (`if link(ui, "Open System Settings").clicked() { … }`).
//!
//! For the common pure-URL case, [`link_to`] wraps [`external_link`] and
//! performs the `ctx().open_url` itself, matching the apps'
//! `hyperlink_to` usage. The "optional external-link glyph" is provided
//! by [`external_link`] (and [`link_to`]); [`link`] omits it.

use egui::{Response, Ui};

use crate::{metrics, palette};

/// Glyph appended after an [`external_link`] / [`link_to`] label to mark
/// that the link leaves the app (opens a browser / external handler).
/// Uses the common "north-east arrow" so it renders without a custom
/// font. Prefixed with a no-break space so the glyph stays welded to the
/// label (never wrapping onto its own line) while keeping a small gap.
const EXTERNAL_GLYPH: &str = "\u{00A0}\u{2197}"; // ↗

/// A clickable, accent-colored text link. No external glyph — use this
/// for in-app navigation or "open this" *actions* where the caller runs
/// the side effect (the look of vernier's "Open System Settings" and its
/// settings-path link).
///
/// Underlines on hover/focus only, shows a pointing-hand cursor, and
/// returns the click [`Response`] — the caller decides what a click does:
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// if kanso::widgets::link(ui, "Open System Settings").clicked() {
///     // open_screen_recording_settings();
/// }
/// # }
/// ```
pub fn link(ui: &mut Ui, text: &str) -> Response {
    link_inner(ui, text, false)
}

/// Like [`link`] but appends a trailing external-link glyph (`↗`) to
/// signal the click leaves the app. The caller still owns the side
/// effect; for the common "just open a URL" case prefer [`link_to`].
pub fn external_link(ui: &mut Ui, text: &str) -> Response {
    link_inner(ui, text, true)
}

/// A URL link: renders `text` as an [`external_link`] and opens `url` in
/// the user's browser on click (via `ctx().open_url`). This is the
/// drop-in for the apps' `ui.hyperlink_to(text, url)` calls.
///
/// Returns the [`Response`] so callers can still observe the click.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::widgets::link_to(ui, "github.com/jondkinney/vernier",
///     "https://github.com/jondkinney/vernier");
/// # }
/// ```
pub fn link_to(ui: &mut Ui, text: &str, url: &str) -> Response {
    let response = external_link(ui, text);
    if response.clicked() {
        ui.ctx().open_url(egui::OpenUrl::same_tab(url));
    }
    response
}

/// Shared implementation: lay out an accent-colored, click-sensing label,
/// then hand-paint the galley so the underline can be toggled per state
/// (none at rest, accent on hover/focus) — the same approach egui's stock
/// [`egui::Link`] uses, but wired to kanso's [`palette::ACCENT`] and with
/// an optional trailing external glyph.
fn link_inner(ui: &mut Ui, text: &str, external: bool) -> Response {
    // Build the label text once; the external glyph carries its own
    // leading no-break space so it doesn't crowd the last character.
    let label_text = if external {
        format!("{text}{EXTERNAL_GLYPH}")
    } else {
        text.to_owned()
    };

    let label = egui::Label::new(egui::RichText::new(label_text).color(palette::ACCENT))
        .sense(egui::Sense::click());
    let (galley_pos, galley, response) = label.layout_in_ui(ui);

    if ui.is_rect_visible(response.rect) {
        let visuals = ui.style().interact(&response);
        // Underline only on hover/focus, matching egui's `Link` and the
        // task's "underline-on-hover" spec. Width tracks the resolved
        // foreground stroke so it scales with the text.
        let underline = if response.hovered() || response.has_focus() {
            egui::Stroke::new(
                visuals.fg_stroke.width.max(metrics::BORDER_WIDTH),
                palette::ACCENT,
            )
        } else {
            egui::Stroke::NONE
        };
        ui.painter().add(
            egui::epaint::TextShape::new(galley_pos, galley, palette::ACCENT)
                .with_underline(underline),
        );
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    response
}
