//! The About screen both apps grow once they have a sidebar: a centered
//! hero with the app's logo, name, version, an optional one-line blurb,
//! and a short list of labeled links (homepage, repo, license, …).
//!
//! Pairs with [`super::sidebar_header`] — the header owns the small
//! identity mark in the sidebar; this owns the large identity hero in the
//! content pane.
//!
//! ## Reconciling the two apps
//!
//! - **vernier** (`prefs.rs::about_section`) is the richer version: a
//!   `vertical_centered` column with a large (112px) logo, a 28pt strong
//!   name, a muted `Version …` line (`from_gray(170)` → [`palette::TEXT_MUTED`]),
//!   a blurb, and link rows. Its links were hand-built `LayoutJob`s using
//!   the literal `rgb(0x4f,0xa3,0xff)` — i.e. [`palette::ACCENT`].
//! - **hyprcorrect** (`prefs.rs::about_panel`) is the leaner version:
//!   left-aligned, no logo, a `Version …` line, a blurb, then
//!   `field_label("Source") + hyperlink` and `field_label("License") +
//!   caption`.
//!
//! kanso adopts vernier's centered, logo-bearing hero (the brief's "app
//! logo (large) + name + version") and folds hyprcorrect's
//! Source/License rows into a single generic `links: Vec<(label, url)>`
//! list — so "Homepage", "Repository", "License" are just entries. Every
//! inline literal collapses onto an existing token; only the two hero
//! sizes are new metrics ([`metrics::ABOUT_LOGO_SIZE`],
//! [`metrics::ABOUT_NAME_SIZE`]).

use egui::Ui;

use crate::{metrics, palette};

/// Everything the About hero renders. The app owns the logo *texture*
/// (its rendered icon) and the link targets; kanso owns the layout,
/// sizing, and link styling.
///
/// `links` are `(label, url)` pairs shown as a centered column of
/// hyperlinks — e.g. `("Homepage", "https://…")`, `("Repository",
/// "https://github.com/…")`, `("License", "https://…")`. The `label` is
/// what the user sees; the `url` is opened on click.
pub struct AboutInfo<'a> {
    /// The app's icon, already uploaded as a texture. `None` renders the
    /// hero without a logo (matching hyprcorrect, which shipped no mark).
    pub logo: Option<egui::Image<'a>>,
    /// Display name, shown large and strong (e.g. `"Vernier"`).
    pub name: &'a str,
    /// Version string, shown muted under the name. Pass the bare version
    /// (e.g. `env!("CARGO_PKG_VERSION")`); the `"Version "` prefix is
    /// added for you.
    pub version: &'a str,
    /// Optional one-line description shown under the version (both apps
    /// had one). `None` omits the line and its spacing.
    pub blurb: Option<&'a str>,
    /// Labeled links, rendered as a centered column of hyperlinks in
    /// [`palette::ACCENT`].
    pub links: &'a [(&'a str, &'a str)],
}

impl<'a> AboutInfo<'a> {
    /// Construct the minimal hero — name + version, no logo, no blurb,
    /// no links. Use the public fields (or struct-update syntax) to fill
    /// in the rest.
    pub fn new(name: &'a str, version: &'a str) -> Self {
        Self {
            logo: None,
            name,
            version,
            blurb: None,
            links: &[],
        }
    }
}

/// Render the centered About hero: large logo, name, `Version …`, an
/// optional blurb, then the labeled links.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui, icon: &kanso::egui::TextureHandle) {
/// use kanso::{egui, widgets::{about_pane, AboutInfo}};
/// about_pane(
///     ui,
///     AboutInfo {
///         logo: Some(egui::Image::new(icon)),
///         name: "Vernier",
///         version: env!("CARGO_PKG_VERSION"),
///         blurb: Some("A cross-platform Rust measurement overlay."),
///         links: &[
///             ("Homepage", "https://example.com"),
///             ("Repository", "https://github.com/jondkinney/vernier"),
///             ("License", "https://spdx.org/licenses/MIT.html"),
///         ],
///     },
/// );
/// # }
/// ```
pub fn about_pane(ui: &mut Ui, info: AboutInfo<'_>) {
    let AboutInfo {
        logo,
        name,
        version,
        blurb,
        links,
    } = info;

    ui.vertical_centered(|ui| {
        ui.add_space(metrics::SETTING_BLOCK_SPACING);

        if let Some(logo) = logo {
            let size = metrics::ABOUT_LOGO_SIZE;
            ui.add(logo.fit_to_exact_size(egui::vec2(size, size)));
            ui.add_space(metrics::SECTION_GAP);
        }

        ui.label(
            egui::RichText::new(name)
                .size(metrics::ABOUT_NAME_SIZE)
                .strong(),
        );
        ui.add_space(metrics::HEADER_PAD / 2.0);
        ui.label(
            egui::RichText::new(format!("Version {version}"))
                .size(metrics::FIELD_LABEL_SIZE)
                .color(palette::TEXT_MUTED),
        );

        if let Some(blurb) = blurb {
            ui.add_space(metrics::SECTION_GAP);
            ui.label(egui::RichText::new(blurb).size(metrics::FIELD_LABEL_SIZE));
        }

        if !links.is_empty() {
            ui.add_space(metrics::SECTION_GAP);
            // Force kanso's canonical link blue regardless of the host
            // app's theme — vernier hand-painted this exact color, so a
            // stock `Hyperlink` (which reads `visuals.hyperlink_color`)
            // matches it only if we pin the color here.
            ui.scope(|ui| {
                ui.visuals_mut().hyperlink_color = palette::ACCENT;
                for &(label, url) in links {
                    ui.add(egui::Hyperlink::from_label_and_url(label, url));
                    ui.add_space(metrics::HEADER_PAD / 2.0);
                }
            });
        }

        ui.add_space(metrics::SETTING_BLOCK_SPACING);
    });
}
