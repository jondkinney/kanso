//! The recurring building blocks every settings window re-implemented.

mod app_picker;
mod buttons;
mod caption;
mod code;
mod helpers;
mod inputs;
mod sidebar;

pub use app_picker::{AppEntry, app_picker};
pub use buttons::{primary_button, secondary_button};
pub use caption::caption;
pub use code::{code, code_span};
pub use helpers::{field_label, padded_text_edit, section_header, setting};
pub use inputs::{dropdown, search_field};
pub use sidebar::{app_shell, content, nav_item, nav_list, sidebar};
