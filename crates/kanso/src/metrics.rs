//! Layout metrics — the spacing, sizing, and corner-radius constants the
//! apps repeated as inline literals. Keeping them here lets a rhythm
//! change land in one place.

/// Default sidebar width ([`crate::widgets::sidebar`]).
pub const SIDEBAR_WIDTH: f32 = 200.0;
/// Height of a sidebar [`crate::widgets::nav_item`] row.
pub const NAV_ROW_HEIGHT: f32 = 32.0;

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
/// Corner radius for inputs / neutral buttons.
pub const CONTROL_CORNER: u8 = 6;

/// Corner radius for an inline code pill.
pub const CODE_CORNER: f32 = 3.0;
/// Corner radius for cards / grouped surfaces.
pub const CARD_CORNER: f32 = 10.0;
/// Corner radius for chips / tokens.
pub const CHIP_CORNER: f32 = 4.0;
/// Corner radius for the selected/hover fill behind a nav row.
pub const NAV_CORNER: f32 = 6.0;
