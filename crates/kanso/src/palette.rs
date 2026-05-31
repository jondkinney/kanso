//! Named colors for the cohort's dark UI.
//!
//! These consolidate the ad-hoc `Color32::from_gray(N)` / `from_rgb`
//! literals the apps repeated. The values here are reconciled to the
//! **audited** literals in vernier/hyprcorrect so adopting kanso does not
//! silently restyle a window.
//!
//! Surfaces deliberately match the apps' default: the prefs windows ride
//! egui's default dark `Visuals` and only override text styles + spacing
//! (see [`crate::theme`]), so kanso does NOT repaint panel/window fills.
//! These tokens are for the *widgets* that opt into them (cards, code
//! pills, accent buttons, severity banners).

use egui::Color32;

/// Primary accent — links, selection emphasis, primary buttons.
///
/// The apps diverge slightly (vernier's link blue `rgb(79,163,255)`,
/// hyprcorrect's brand `rgb(74,134,192)`); kanso picks vernier's
/// interactive link blue as the single canonical accent.
pub const ACCENT: Color32 = Color32::from_rgb(79, 163, 255);
/// Text painted on top of an [`ACCENT`] fill (e.g. a primary button).
pub const ON_ACCENT: Color32 = Color32::from_gray(15);

/// Default body text on a dark surface (`from_gray(225)`).
pub const TEXT: Color32 = Color32::from_gray(225);
/// Secondary / caption text (`from_gray(170)`).
pub const TEXT_MUTED: Color32 = Color32::from_gray(170);
/// Tertiary / hint text (`from_gray(150)`).
pub const TEXT_FAINT: Color32 = Color32::from_gray(150);

/// Card / grouped-row background (`from_gray(34)`).
pub const CARD: Color32 = Color32::from_gray(34);
/// Card / modal border (`from_gray(60)`).
pub const CARD_STROKE: Color32 = Color32::from_gray(60);
/// Resting chip / token background (`from_gray(64)`).
pub const CHIP: Color32 = Color32::from_gray(64);
/// Background behind an inline `code` pill (`from_gray(48)`).
pub const CODE_BG: Color32 = Color32::from_gray(48);
/// Text inside an inline `code` pill (`from_gray(225)`).
pub const CODE_TEXT: Color32 = Color32::from_gray(225);

/// Success / positive status (`rgb(80,200,120)`).
pub const OK: Color32 = Color32::from_rgb(80, 200, 120);
/// Error / destructive (`rgb(220,90,90)`).
pub const ERROR: Color32 = Color32::from_rgb(220, 90, 90);
/// Warning amber (`rgb(220,160,50)`).
pub const WARN: Color32 = Color32::from_rgb(220, 160, 50);
/// Informational accent — spell/grammar hints (`rgb(96,165,250)`).
pub const INFO: Color32 = Color32::from_rgb(96, 165, 250);
/// Active "capturing a chord" highlight (`rgb(50,90,140)`).
pub const CAPTURE: Color32 = Color32::from_rgb(50, 90, 140);
