//! Font installation.
//!
//! Regular text uses **egui's standard font** — the cohort decided the
//! default reads well for body copy. kanso bundles `AdwaitaSans-Regular.ttf`
//! (once, for the whole cohort) and reserves it for the **modifier glyphs**:
//! it builds the [`SHORTCUT_FAMILY`] used to render keyboard-shortcut chips
//! — Adwaita's key glyphs (⌃ ⇧ ⌥ ⌘ …) plus the omarchy SUPER logo at
//! [`OMARCHY_LOGO`], loaded at runtime from the user's font dir.
//!
//! ([`FontOptions::adwaita_sans`] can still make Adwaita the body font, but
//! it's off by default.)

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::{Context, FontData, FontDefinitions, FontFamily, FontTweak};

/// Adwaita Sans Regular — covers ASCII plus the macOS-style key glyphs
/// the shortcut chips use (⌃ ⇧ ⌥ ⌘ ⎋ ↵ ⇥ ⌫ ⌦ ␣ ↑↓←→). Bundled so the UI
/// looks identical regardless of which fonts are installed on the host.
pub const ADWAITA_SANS: &[u8] = include_bytes!("../assets/AdwaitaSans-Regular.ttf");

/// The Hyprland/omarchy launcher logo lives at this private-use
/// codepoint in `omarchy.ttf`. Render it with [`SHORTCUT_FAMILY`] once
/// [`FontOptions::shortcut_family`] is enabled and the font is present
/// (gate on [`omarchy_available`]).
pub const OMARCHY_LOGO: char = '\u{E900}';

/// Name of the custom font family for shortcut/key chips — Adwaita key
/// glyphs + the omarchy SUPER logo, with the default proportional fonts
/// as fallback. Use via `egui::FontFamily::Name(SHORTCUT_FAMILY.into())`.
pub const SHORTCUT_FAMILY: &str = "kanso-shortcut";

static OMARCHY_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Whether `omarchy.ttf` was found and wired into [`SHORTCUT_FAMILY`]
/// during [`install`]. Apps gate the SUPER-logo chip segment on this.
pub fn omarchy_available() -> bool {
    OMARCHY_AVAILABLE.load(Ordering::Relaxed)
}

/// Options for [`install`].
#[derive(Clone, Debug)]
pub struct FontOptions {
    /// Install the bundled Adwaita Sans as the *body* (proportional) font.
    /// Default: `false` — the cohort uses egui's standard font for regular
    /// text and reserves Adwaita Sans for the modifier glyphs in
    /// [`SHORTCUT_FAMILY`]. Set `true` only if you want Adwaita body text.
    pub adwaita_sans: bool,
    /// Build [`SHORTCUT_FAMILY`] for keyboard-shortcut chips (loads
    /// `omarchy.ttf` if present). Default: `false`.
    pub shortcut_family: bool,
    /// Append the omarchy logo font at the *end* of the shortcut chain
    /// rather than right after Adwaita. `true` matches hyprcorrect's
    /// cmap fix (Adwaita's ASCII wins; omarchy only supplies U+E900);
    /// `false` matches vernier. Default: `true`.
    pub omarchy_last: bool,
}

impl Default for FontOptions {
    fn default() -> Self {
        Self {
            adwaita_sans: false,
            shortcut_family: false,
            omarchy_last: true,
        }
    }
}

/// Install kanso's fonts into the egui context. Call once at startup
/// (or via [`crate::theme::apply`], which calls this for you).
pub fn install(ctx: &Context, opts: &FontOptions) {
    let mut fonts = FontDefinitions::default();
    // Capture egui's default proportional chain before we prepend
    // Adwaita, so the shortcut family's fallback is the *stock* chain.
    let default_prop = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    if opts.adwaita_sans || opts.shortcut_family {
        fonts.font_data.insert(
            "kanso-adwaita".to_owned(),
            Arc::new(FontData::from_static(ADWAITA_SANS)),
        );
    }
    if opts.adwaita_sans {
        // Prepend so it wins over egui's bundled proportional faces,
        // which remain as fallback for any glyph Adwaita lacks.
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "kanso-adwaita".to_owned());
    }

    if opts.shortcut_family {
        let omarchy_loaded = load_omarchy(&mut fonts);
        OMARCHY_AVAILABLE.store(omarchy_loaded, Ordering::Relaxed);

        let mut chain = vec!["kanso-adwaita".to_owned()];
        if omarchy_loaded && !opts.omarchy_last {
            chain.push("kanso-omarchy".to_owned());
        }
        chain.extend(default_prop);
        if omarchy_loaded && opts.omarchy_last {
            chain.push("kanso-omarchy".to_owned());
        }
        fonts
            .families
            .insert(FontFamily::Name(SHORTCUT_FAMILY.into()), chain);
    }

    ctx.set_fonts(fonts);
}

fn load_omarchy(fonts: &mut FontDefinitions) -> bool {
    for path in omarchy_paths() {
        if let Ok(bytes) = std::fs::read(&path) {
            let mut data = FontData::from_owned(bytes);
            // The omarchy glyph fills its em square; scale it down and
            // nudge it onto the text baseline so it sits like a letter.
            data.tweak = FontTweak {
                scale: 0.75,
                y_offset_factor: 0.09,
                ..Default::default()
            };
            fonts
                .font_data
                .insert("kanso-omarchy".to_owned(), Arc::new(data));
            return true;
        }
    }
    false
}

fn omarchy_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(std::path::PathBuf::from(home).join(".local/share/fonts/omarchy.ttf"));
    }
    paths.push(std::path::PathBuf::from("/usr/share/fonts/omarchy.ttf"));
    paths
}

/// Register an extra named font family from raw TTF/OTF bytes, appending
/// the default proportional chain as fallback so the family is never
/// left unbound (which would panic at first paint). Rebuilds and re-sets
/// the font set, so call after [`install`].
pub fn register_family(ctx: &Context, family: &str, bytes: Vec<u8>) {
    let mut fonts = FontDefinitions::default();
    let key = format!("kanso-extra-{family}");
    fonts
        .font_data
        .insert(key.clone(), Arc::new(FontData::from_owned(bytes)));
    let mut chain = vec![key];
    if let Some(default_prop) = fonts.families.get(&FontFamily::Proportional).cloned() {
        chain.extend(default_prop);
    }
    fonts
        .families
        .insert(FontFamily::Name(family.into()), chain);
    ctx.set_fonts(fonts);
}
