//! "Select a running app" — a searchable, single-select list.
//!
//! The cohort keeps re-doing this: hyprcorrect's `apps.rs`/`docker.rs`
//! pick a focused/blocklisted app; mousehop enumerates desktop entries /
//! the frontmost app; vernier picks a screenshot-handoff target. The
//! *data source* differs per app (running windows, `.desktop` files,
//! `$PATH` binaries), so kanso owns only the **look**: the caller passes
//! the candidate [`AppEntry`] list plus the selection + query state.

use egui::{Response, Ui};

/// One selectable app/target. `id` is the stable value the caller keys
/// off (window class, desktop-entry id, binary path); `name` is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
}

impl AppEntry {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

/// A search box over a scrollable, single-select list of apps. Sets
/// `selected` to the clicked entry's `id`. Returns the search field's
/// [`Response`].
pub fn app_picker(
    ui: &mut Ui,
    apps: &[AppEntry],
    selected: &mut Option<String>,
    query: &mut String,
) -> Response {
    let resp = super::search_field(ui, query, "Search apps…");
    let needle = query.to_lowercase();
    egui::ScrollArea::vertical()
        .max_height(220.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for app in apps
                .iter()
                .filter(|a| needle.is_empty() || a.name.to_lowercase().contains(&needle))
            {
                let is_selected = selected.as_deref() == Some(app.id.as_str());
                if super::nav_item(ui, is_selected, &app.name).clicked() {
                    *selected = Some(app.id.clone());
                }
            }
        });
    resp
}
