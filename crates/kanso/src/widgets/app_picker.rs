//! "Select a running app" — a searchable, single-select list with optional
//! per-entry icons.
//!
//! The cohort keeps re-doing this: hyprcorrect's `apps.rs`/`docker.rs`
//! pick a focused/blocklisted app; mousehop enumerates desktop entries /
//! the frontmost app; vernier picks a screenshot-handoff target. The
//! *data source* differs per app (running windows, `.desktop` files,
//! `$PATH` binaries) **and so does icon resolution** (`.desktop` themes are
//! platform-specific), so kanso owns only the **look**: the caller passes
//! the candidate [`AppEntry`] list (each carrying an already-uploaded icon
//! texture, if any) plus the selection + query state.

use std::hash::Hash;

use egui::{Response, Ui};

use crate::{metrics, palette};

/// One selectable app/target. `id` is the stable value the caller keys
/// off (window class, desktop-entry id, binary path); `name` is shown.
/// `icon` is an optional texture the **app** already uploaded — kanso does
/// not resolve `.desktop`/icon-theme entries (that's app/platform logic);
/// it only paints the texture you hand it (e.g. `Some(handle.id())`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub icon: Option<egui::TextureId>,
}

impl AppEntry {
    /// An entry with no icon.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
        }
    }

    /// Attach an icon — the app's already-uploaded texture id, e.g.
    /// `texture_handle.id()`. Rendered in a small square at the row's left.
    pub fn with_icon(mut self, icon: egui::TextureId) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A search box over a scrollable, single-select list of apps. Sets
/// `selected` to the clicked entry's `id`. Returns the search field's
/// [`Response`].
///
/// If any entry carries an [`AppEntry::icon`], every row reserves an icon
/// slot (a [`palette::CHIP`] placeholder tile for entries without one) so
/// the names stay aligned; a fully icon-less list renders as plain text
/// rows.
pub fn app_picker(
    ui: &mut Ui,
    apps: &[AppEntry],
    selected: &mut Option<String>,
    query: &mut String,
) -> Response {
    picker_list(ui, apps, selected, query).0
}

/// Shared body for [`app_picker`] and [`app_picker_combo`]: the search field
/// over the scrollable row list. Returns the search field's [`Response`] and
/// whether an entry was picked this frame (so the combo can close on pick
/// without closing when the search field is merely focused).
fn picker_list(
    ui: &mut Ui,
    apps: &[AppEntry],
    selected: &mut Option<String>,
    query: &mut String,
) -> (Response, bool) {
    let resp = super::search_field(ui, query, "Search apps…");
    let needle = query.to_lowercase();
    let show_icons = apps.iter().any(|a| a.icon.is_some());
    let picked = egui::ScrollArea::vertical()
        .max_height(LIST_MAX_HEIGHT)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let mut picked = false;
            for app in apps
                .iter()
                .filter(|a| needle.is_empty() || a.name.to_lowercase().contains(&needle))
            {
                let is_selected = selected.as_deref() == Some(app.id.as_str());
                if app_row(ui, app, is_selected, show_icons).clicked() {
                    *selected = Some(app.id.clone());
                    picked = true;
                }
            }
            picked
        })
        .inner;
    (resp, picked)
}

/// The same searchable, single-select [`app_picker`] list, collapsed behind
/// a combo button so it doesn't take vertical space until opened. The button
/// shows the selected entry's name (or `placeholder` when nothing is
/// chosen); opening it reveals the search field + scrollable icon list.
///
/// Prefer this over [`app_picker`] when the picker shares a settings pane
/// with other controls and an always-open list would make the page too
/// tall. Pass an `id_salt` unique within the parent `Ui`. Returns the combo
/// button's [`Response`].
pub fn app_picker_combo(
    ui: &mut Ui,
    id_salt: impl Hash,
    apps: &[AppEntry],
    selected: &mut Option<String>,
    query: &mut String,
    placeholder: &str,
) -> Response {
    let selected_text = selected
        .as_deref()
        .and_then(|id| apps.iter().find(|a| a.id == id))
        .map(|a| a.name.as_str())
        .unwrap_or(placeholder)
        .to_owned();
    // `.width(w)` makes the closed button's outer width ≈ w, so this fills
    // the row like a full-width field; the popup matches the button width.
    let w = ui.available_width();
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .width(w)
        // Default `CloseOnClick` closes the popup the moment the user clicks
        // the search field. Switch to close-on-click-*outside* so typing in
        // the search keeps it open, and close explicitly when a row is picked.
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show_ui(ui, |ui| {
            ui.set_min_width(w);
            let (_, picked) = picker_list(ui, apps, selected, query);
            if picked {
                ui.close();
            }
        })
        .response
}

/// One picker row. Without an icon column this is just a [`super::nav_item`]
/// (the original text-only look); with icons it paints a row-spanning
/// selection/hover wash, the icon (or a placeholder tile), then the name.
fn app_row(ui: &mut Ui, app: &AppEntry, selected: bool, show_icon: bool) -> Response {
    if !show_icon {
        return super::nav_item(ui, selected, &app.name);
    }
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ROW_HEIGHT),
        egui::Sense::click(),
    );
    if ui.is_rect_visible(rect) {
        let vis = ui.style().interact_selectable(&resp, selected);
        if selected || resp.hovered() {
            ui.painter().rect_filled(
                rect.expand(-2.0),
                egui::CornerRadius::same(metrics::RADIUS),
                vis.bg_fill,
            );
        }
        let icon_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + ROW_PAD, rect.center().y - ICON_SIZE / 2.0),
            egui::vec2(ICON_SIZE, ICON_SIZE),
        );
        if let Some(id) = app.icon {
            egui::Image::new(egui::load::SizedTexture::new(
                id,
                egui::vec2(ICON_SIZE, ICON_SIZE),
            ))
            .paint_at(ui, icon_rect);
        } else {
            // Placeholder tile so rows with and without an icon line up.
            ui.painter().rect_filled(
                icon_rect.shrink(1.0),
                egui::CornerRadius::same(4),
                palette::CHIP,
            );
        }
        ui.painter().text(
            egui::pos2(icon_rect.right() + ROW_PAD, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &app.name,
            egui::FontId::proportional(metrics::FIELD_LABEL_SIZE),
            vis.text_color(),
        );
    }
    resp
}

/// Max height of the scrollable list before it scrolls.
const LIST_MAX_HEIGHT: f32 = 220.0;
/// Height of one icon row.
const ROW_HEIGHT: f32 = 28.0;
/// Icon (and placeholder tile) edge length.
const ICON_SIZE: f32 = 20.0;
/// Left inset and icon→name gap.
const ROW_PAD: f32 = 6.0;
