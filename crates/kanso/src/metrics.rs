//! Layout metrics — the spacing, sizing, and corner-radius constants the
//! apps repeated as inline literals. Keeping them here lets a rhythm
//! change land in one place.

/// Default sidebar width ([`crate::widgets::sidebar`]).
pub const SIDEBAR_WIDTH: f32 = 200.0;
/// Height of a sidebar [`crate::widgets::nav_item`] row.
pub const NAV_ROW_HEIGHT: f32 = 32.0;
/// Logo size in the sidebar header ([`crate::widgets::sidebar_header`]).
pub const LOGO_SIZE: f32 = 24.0;
/// Vertical padding above and below the sidebar header.
pub const HEADER_PAD: f32 = 8.0;

/// Vertical gap a section header leaves before its body.
pub const SECTION_GAP: f32 = 14.0;
/// Vertical breathing room after a settings block ([`crate::widgets::setting`]).
pub const SETTING_BLOCK_SPACING: f32 = 22.0;

/// Size of a [`crate::widgets::field_label`].
pub const FIELD_LABEL_SIZE: f32 = 14.0;
/// Caption (muted explainer) text size.
pub const CAPTION_SIZE: f32 = 12.0;
/// Monospace size inside an inline code pill within a caption.
pub const CAPTION_CODE_SIZE: f32 = 11.5;
/// Line height used while laying out captions (gives pills room).
pub const CAPTION_LINE_HEIGHT: f32 = 22.0;

/// Minimum interactive control height — inputs and buttons share this so
/// they line up and never change height on hover.
pub const CONTROL_HEIGHT: f32 = 28.0;
/// Border width for inputs / buttons. Constant across rest/hover/focus so
/// only the *color* changes, never the geometry.
pub const BORDER_WIDTH: f32 = 1.0;
/// The standard corner radius — applied to every interactive control and
/// container (nav rows, inputs, buttons, dropdowns and their popups) so
/// rounding reads consistently across the whole UI. Inline code pills
/// ([`CODE_CORNER`]) and cards ([`CARD_CORNER`]) keep their own
/// scale-appropriate radii.
pub const RADIUS: u8 = 6;

/// Corner radius for an inline code pill.
pub const CODE_CORNER: f32 = 3.0;
/// Monospace size inside a standalone [`crate::widgets::code`] pill.
pub const CODE_SIZE: f32 = 12.5;
/// Horizontal padding inside a standalone code pill.
pub const CODE_PAD_X: f32 = 5.0;
/// Vertical padding inside a standalone code pill.
pub const CODE_PAD_Y: f32 = 2.0;
/// Corner radius for cards / grouped surfaces.
pub const CARD_CORNER: f32 = 10.0;
/// Horizontal inner padding inside a [`crate::widgets::card`].
pub const CARD_PAD_X: i8 = 18;
/// Vertical inner padding inside a [`crate::widgets::card`].
pub const CARD_PAD_Y: i8 = 16;
/// Corner radius for chips / tokens.
pub const CHIP_CORNER: f32 = 4.0;

// --- Shortcut keycap chips ([`crate::widgets::shortcut_chip`]) ---
//
// vernier sized glyphs at 17pt in a 14px box with a 6px gap; hyprcorrect
// drew one 280×32 chip at 17pt. Reconciled: per-segment rounded keycaps
// sized to the glyph, sharing the control height for the interactive
// capture chip so it lines up with the inputs beside it.

/// Glyph point size on a shortcut keycap (vernier's `CHIP_LETTER_PT`,
/// sized to match the omarchy logo's cap height).
pub const SHORTCUT_GLYPH_SIZE: f32 = 17.0;
/// Height of an individual rounded keycap.
pub const SHORTCUT_KEYCAP_HEIGHT: f32 = 22.0;
/// Horizontal padding inside a keycap, each side of the glyph.
pub const SHORTCUT_KEYCAP_PAD_X: f32 = 7.0;
/// Floor on a keycap's width so narrow single glyphs (`⌘`, `+`) still
/// read as a square-ish cap rather than a sliver.
pub const SHORTCUT_KEYCAP_MIN_WIDTH: f32 = 22.0;
/// Gap between adjacent keycaps in a chord (vernier's `CHIP_GAP`).
pub const SHORTCUT_KEYCAP_GAP: f32 = 6.0;
/// Fixed width of the interactive capture chip (between vernier's 200px
/// and hyprcorrect's 280px row chip).
pub const SHORTCUT_CHIP_WIDTH: f32 = 240.0;

// --- Severity banner ([`crate::widgets::banner`]) ---
//
// vernier's notice frames both used `corner_radius(8)` + a symmetric inner
// margin (`(20, 12)` for the top-panel banner, `(12, 10)` for the inline
// "static bind" one) with a strong ~13.5 title over a ~12 body; hyprcorrect's
// review cards used the same `8` rounding and a `(14, 12)` margin. These
// reconcile the repeated literals into one banner rhythm.

/// Corner radius for a severity banner strip.
pub const BANNER_CORNER: u8 = 8;
/// Horizontal inner padding inside a banner strip.
pub const BANNER_PAD_X: i8 = 14;
/// Vertical inner padding inside a banner strip.
pub const BANNER_PAD_Y: i8 = 12;
/// Banner title text size.
pub const BANNER_TITLE_SIZE: f32 = 13.5;
/// Banner body text size.
pub const BANNER_BODY_SIZE: f32 = 12.0;
/// Gap between a banner's title and its body.
pub const BANNER_TITLE_GAP: f32 = 4.0;
/// Gap between a banner's body and its action button.
pub const BANNER_ACTION_GAP: f32 = 8.0;
/// Alpha the severity color is laid over [`crate::palette::CARD`] at to
/// derive a banner's faint tinted fill.
pub const BANNER_FILL_ALPHA: f32 = 0.16;
/// Alpha the severity color is laid over [`crate::palette::CARD`] at to
/// derive a banner's clearer edge stroke.
pub const BANNER_STROKE_ALPHA: f32 = 0.55;

/// Large hero logo size in the About pane ([`crate::widgets::about_pane`]).
/// Vernier's About screen rendered its icon at this size.
pub const ABOUT_LOGO_SIZE: f32 = 112.0;
/// App-name size in the About hero ([`crate::widgets::about_pane`]).
/// Vernier set the name at 28pt strong.
pub const ABOUT_NAME_SIZE: f32 = 28.0;
