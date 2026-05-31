//! Keyboard-shortcut keycap chips — the cohort's signature widget.
//!
//! vernier's Shortcuts pane and hyprcorrect's Hotkeys page both hand-paint
//! segmented `⌃⇧⌥⌘` chips from a stored `CTRL+SHIFT+ALT+SUPER+KEY`
//! accelerator string. This reconciles the two into one canonical look:
//! each token becomes its own **rounded keycap** (vernier's per-segment
//! intent) laid out on a single row, glyphs rendered through
//! [`fonts::SHORTCUT_FAMILY`] so the macOS-style key symbols resolve even
//! when the host's proportional font lacks them. The `SUPER` modifier
//! renders as the omarchy launcher logo ([`fonts::OMARCHY_LOGO`]) when
//! [`fonts::omarchy_available`] reports the font is installed, falling
//! back to the Command glyph `⌘` everywhere else.
//!
//! Three entry points share the rendering core:
//!
//! - [`shortcut_chip`] — paint a chord from an explicit token slice.
//! - [`shortcut_chip_str`] — paint a chord from a stored
//!   `"CTRL+SHIFT+K"` accelerator string (the form both apps persist).
//! - [`shortcut_capture_chip`] — the full interactive row chip: shows the
//!   chord, a `Press a shortcut…` prompt while recording, or a
//!   `Click to set` placeholder when empty, and returns a click
//!   [`Response`] so the caller can arm/cancel capture.
//!
//! The token vocabulary is the union of both apps' chord-capture
//! mappings (see vernier's `chord_capture::chord_key_token` and
//! hyprcorrect's `chord_glyphs`): modifier names plus their aliases,
//! named keys, arrows, and the punctuation tokens chord-capture spells
//! out so the saved string can't collide with the `+` separator.

use egui::{Color32, Response, Sense, Ui, Vec2};

use crate::{fonts, metrics, palette};

/// Map one chord token (e.g. `"CTRL"`, `"SUPER"`, `"K"`) to the glyph
/// drawn on its keycap. Case-insensitive; accepts the alias spellings
/// both apps' capture paths can emit. `SUPER`/`META`/`CMD`/`WIN` resolve
/// to the omarchy logo when [`fonts::omarchy_available`] is true, else to
/// the Command glyph.
///
/// Unknown tokens pass through verbatim (upper-cased single letters,
/// `F5`, etc.) so the chip degrades gracefully on accelerators kanso
/// doesn't have a symbol for.
pub fn token_glyph(token: &str) -> String {
    match token.trim().to_ascii_uppercase().as_str() {
        // Modifiers (+ the aliases each platform's capture path emits).
        "SUPER" | "META" | "CMD" | "COMMAND" | "WIN" | "WINDOWS" | "MOD4" => {
            if fonts::omarchy_available() {
                fonts::OMARCHY_LOGO.to_string()
            } else {
                "\u{2318}".to_string() // ⌘
            }
        }
        "CTRL" | "CONTROL" => "\u{2303}".to_string(), // ⌃
        "SHIFT" => "\u{21E7}".to_string(),            // ⇧
        "ALT" | "OPTION" | "MOD1" => "\u{2325}".to_string(), // ⌥
        // Named keys.
        "RETURN" | "ENTER" => "\u{21B5}".to_string(), // ↵
        "TAB" => "\u{21E5}".to_string(),              // ⇥
        "ESC" | "ESCAPE" => "\u{238B}".to_string(),   // ⎋
        "BACKSPACE" => "\u{232B}".to_string(),        // ⌫
        "DELETE" | "DEL" => "\u{2326}".to_string(),   // ⌦
        "SPACE" => "\u{2423}".to_string(),            // ␣
        "UP" => "\u{2191}".to_string(),               // ↑
        "DOWN" => "\u{2193}".to_string(),             // ↓
        "LEFT" => "\u{2190}".to_string(),             // ←
        "RIGHT" => "\u{2192}".to_string(),            // →
        "PRIOR" | "PAGEUP" => "PgUp".to_string(),
        "NEXT" | "PAGEDOWN" => "PgDn".to_string(),
        // Punctuation chord-capture spells out so the saved string can't
        // collide with the `+` modifier separator — drawn as the literal
        // character it stands for.
        "PLUS" => "+".to_string(),
        "MINUS" => "-".to_string(),
        "EQUAL" => "=".to_string(),
        "UNDERSCORE" => "_".to_string(),
        other => other.to_string(),
    }
}

/// Split a stored accelerator string (`"CTRL+SHIFT+K"`) into its tokens,
/// dropping empties so a trailing/leading `+` or a stray space can't
/// produce a blank keycap.
fn split_chord(stored: &str) -> Vec<&str> {
    stored
        .split('+')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Render a shortcut chord as a row of rounded keycaps from an explicit
/// token slice (e.g. `&["CTRL", "SHIFT", "K"]`), advancing the cursor and
/// returning the hover [`Response`] over the whole row.
///
/// Each token is mapped through [`token_glyph`] and painted on its own
/// [`palette::CHIP`] keycap. Self-sizing: it allocates exactly the space
/// the keycaps occupy, so it composes inside `ui.horizontal(...)` next to
/// a label like any other widget.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::widgets::shortcut_chip(ui, &["SUPER", "SHIFT", "K"]);
/// # }
/// ```
pub fn shortcut_chip(ui: &mut Ui, tokens: &[&str]) -> Response {
    let glyphs: Vec<String> = tokens.iter().map(|t| token_glyph(t)).collect();
    keycaps(ui, &glyphs, palette::CHIP, palette::TEXT)
}

/// Render a shortcut chord from a stored accelerator string such as
/// `"CTRL+SHIFT+K"` (the persisted form both apps round-trip). Empty or
/// whitespace-only input renders nothing and allocates no space.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::widgets::shortcut_chip_str(ui, "SUPER+SHIFT+K");
/// # }
/// ```
pub fn shortcut_chip_str(ui: &mut Ui, stored: &str) -> Response {
    let glyphs: Vec<String> = split_chord(stored).iter().map(|t| token_glyph(t)).collect();
    keycaps(ui, &glyphs, palette::CHIP, palette::TEXT)
}

/// The interactive capture chip used by a Shortcuts/Hotkeys row.
///
/// Reconciles vernier's `shortcut_row` chip and hyprcorrect's
/// `chord_chip` into one click-to-record control. It draws a fixed-width
/// surface and either:
///
/// - while `capturing`: a `Press a shortcut…` prompt on the active
///   [`palette::CAPTURE`] highlight;
/// - when `stored` is empty: a muted `Click to set` placeholder; or
/// - otherwise: the chord rendered as keycaps.
///
/// Returns the click [`Response`]; the caller owns capture state and
/// typically does `if resp.clicked() { stored.clear(); capturing = true; }`
/// (matching both apps, where clicking the chip clears the old binding
/// and arms recording in one step).
pub fn shortcut_capture_chip(ui: &mut Ui, stored: &str, capturing: bool) -> Response {
    let size = Vec2::new(metrics::SHORTCUT_CHIP_WIDTH, metrics::CONTROL_HEIGHT);
    let resp = ui.allocate_response(size, Sense::click());
    let rect = resp.rect;
    let radius = egui::CornerRadius::same(metrics::RADIUS);

    if capturing {
        ui.painter().rect_filled(rect, radius, palette::CAPTURE);
        prompt_text(ui, rect, "Press a shortcut\u{2026}", palette::TEXT);
    } else if stored.trim().is_empty() {
        let bg = if resp.hovered() {
            palette::CONTROL_BG_HOVER
        } else {
            palette::CONTROL_BG
        };
        ui.painter().rect_filled(rect, radius, bg);
        prompt_text(ui, rect, "Click to set", palette::TEXT_FAINT);
    } else {
        // A hover wash on the surface, then keycaps centered on it.
        if resp.hovered() {
            ui.painter()
                .rect_filled(rect, radius, palette::CONTROL_BG_HOVER);
        }
        let glyphs: Vec<String> = split_chord(stored).iter().map(|t| token_glyph(t)).collect();
        paint_keycaps_centered(ui, rect, &glyphs, palette::CHIP, palette::TEXT);
    }
    resp
}

/// Lay out a single prompt string centered in `rect`, in the shortcut
/// font so it shares the chip's metrics.
fn prompt_text(ui: &Ui, rect: egui::Rect, text: &str, color: Color32) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        shortcut_font(metrics::SHORTCUT_GLYPH_SIZE),
        color,
    );
}

/// The font used for keycap glyphs: [`fonts::SHORTCUT_FAMILY`] at the
/// keycap glyph size, so modifier symbols + the omarchy logo resolve.
fn shortcut_font(size: f32) -> egui::FontId {
    egui::FontId::new(size, egui::FontFamily::Name(fonts::SHORTCUT_FAMILY.into()))
}

/// Width one keycap needs to fit `glyph`: the glyph's advance (measured
/// in the shortcut font) plus symmetric horizontal padding, floored at a
/// minimum so single narrow glyphs (`⌘`, `+`) still read as a square-ish
/// cap rather than a sliver.
fn keycap_width(ui: &Ui, glyph: &str) -> f32 {
    let font = shortcut_font(metrics::SHORTCUT_GLYPH_SIZE);
    let advance: f32 = ui.fonts(|f| glyph.chars().map(|c| f.glyph_width(&font, c)).sum());
    (advance + metrics::SHORTCUT_KEYCAP_PAD_X * 2.0).max(metrics::SHORTCUT_KEYCAP_MIN_WIDTH)
}

/// Self-allocating keycap row: measure every keycap, allocate exactly the
/// row's footprint (height [`metrics::CONTROL_HEIGHT`]) and paint the caps
/// into it. Returns the hover [`Response`] over the row. Empty input
/// allocates nothing.
fn keycaps(ui: &mut Ui, glyphs: &[String], fill: Color32, fg: Color32) -> Response {
    if glyphs.is_empty() {
        return ui.allocate_response(Vec2::ZERO, Sense::hover());
    }
    let widths: Vec<f32> = glyphs.iter().map(|g| keycap_width(ui, g)).collect();
    let total_w: f32 =
        widths.iter().sum::<f32>() + metrics::SHORTCUT_KEYCAP_GAP * (glyphs.len() as f32 - 1.0);
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(total_w, metrics::CONTROL_HEIGHT), Sense::hover());
    paint_row(ui, rect, glyphs, &widths, fill, fg);
    resp
}

/// Center a measured keycap row horizontally within `rect` and paint it
/// (used when the surface is a fixed-width capture chip, not self-sized).
fn paint_keycaps_centered(
    ui: &Ui,
    rect: egui::Rect,
    glyphs: &[String],
    fill: Color32,
    fg: Color32,
) {
    if glyphs.is_empty() {
        return;
    }
    let widths: Vec<f32> = glyphs.iter().map(|g| keycap_width(ui, g)).collect();
    let total_w: f32 =
        widths.iter().sum::<f32>() + metrics::SHORTCUT_KEYCAP_GAP * (glyphs.len() as f32 - 1.0);
    let row = egui::Rect::from_center_size(
        rect.center(),
        Vec2::new(total_w, metrics::SHORTCUT_KEYCAP_HEIGHT),
    );
    paint_row(ui, row, glyphs, &widths, fill, fg);
}

/// Paint keycaps left-to-right starting at `row.left()`, each vertically
/// centered on `row` and sized [`metrics::SHORTCUT_KEYCAP_HEIGHT`] tall.
/// Glyphs are drawn through the shortcut font so the symbols and omarchy
/// logo render with a uniform baseline.
fn paint_row(
    ui: &Ui,
    row: egui::Rect,
    glyphs: &[String],
    widths: &[f32],
    fill: Color32,
    fg: Color32,
) {
    let painter = ui.painter();
    let font = shortcut_font(metrics::SHORTCUT_GLYPH_SIZE);
    let radius = egui::CornerRadius::same(metrics::CHIP_CORNER as u8);
    let cy = row.center().y;
    let mut x = row.left();
    for (glyph, &w) in glyphs.iter().zip(widths) {
        let cap = egui::Rect::from_min_size(
            egui::pos2(x, cy - metrics::SHORTCUT_KEYCAP_HEIGHT / 2.0),
            Vec2::new(w, metrics::SHORTCUT_KEYCAP_HEIGHT),
        );
        painter.rect_filled(cap, radius, fill);
        painter.text(
            cap.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            font.clone(),
            fg,
        );
        x += w + metrics::SHORTCUT_KEYCAP_GAP;
    }
}
