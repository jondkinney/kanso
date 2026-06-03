//! # kanso
//!
//! A small shared **egui design system** for the desktop utility-app
//! cohort (vernier, hyprcorrect, and future egui apps). It pulls the
//! pieces every app's preferences/settings window was re-implementing
//! into one dependency:
//!
//! - [`theme`] — one call ([`theme::apply`]) installs the bundled font,
//!   text-style scale, spacing, and a dark palette so every app looks
//!   the same out of the box.
//! - [`fonts`] — bundles Adwaita Sans (the apps each `include_bytes!`'d
//!   their own copy) and reserves it for the shortcut/modifier glyphs;
//!   body text uses egui's standard font.
//! - [`palette`] — named colors, replacing scattered `Color32::from_gray`
//!   literals.
//! - [`widgets`] — the recurring building blocks: a left-sidebar / right-
//!   content [`widgets::app_shell`], left-aligned [`widgets::nav_item`]s,
//!   [`widgets::primary_button`]/[`widgets::secondary_button`],
//!   [`widgets::search_field`], [`widgets::dropdown`], an inline
//!   [`widgets::code`] highlight (markdown-style backtick spans), and a
//!   searchable [`widgets::app_picker`] for "select a running app".
//!
//! The egui version is re-exported as [`kanso::egui`](egui) so consumers
//! can depend on exactly the egui this crate was built against.
//!
//! ```no_run
//! # fn demo(ctx: &kanso::egui::Context) {
//! kanso::theme::apply(ctx); // once, at startup
//! # }
//! ```

pub mod fonts;
pub mod metrics;
pub mod palette;
pub mod scroll;
pub mod theme;
pub mod widgets;

/// Re-export so consumers can write `kanso::egui::…` and stay on the
/// exact egui version kanso was compiled against.
pub use egui;

pub use theme::{Theme, apply};
