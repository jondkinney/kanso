//! Left sidebar + right content shell, and the left-aligned nav rows
//! that fill the sidebar.
//!
//! egui's stock `SelectableLabel` centers its text; settings panes want
//! a left-aligned `Section name` row, so [`nav_item`] paints one over a
//! clickable rect (lifted from vernier's `sidebar_item`).

use egui::{Context, Response, Ui};

use crate::metrics;

/// The sidebar's app-identity header — an optional logo to the left of the
/// app name (heading style), with consistent padding. Put it at the very
/// top of your [`sidebar`]. The app owns the logo *image* (its rendered
/// icon texture); kanso owns the size, alignment, and spacing.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui, icon: &kanso::egui::TextureHandle) {
/// use kanso::egui;
/// kanso::widgets::sidebar_header(ui, Some(egui::Image::new(icon)), "Vernier");
/// // …or text-only:
/// kanso::widgets::sidebar_header(ui, None, "Vernier");
/// # }
/// ```
pub fn sidebar_header(ui: &mut Ui, logo: Option<egui::Image<'_>>, name: &str) {
    ui.add_space(metrics::HEADER_PAD);
    ui.horizontal(|ui| {
        if let Some(logo) = logo {
            ui.add(logo.fit_to_exact_size(egui::vec2(metrics::LOGO_SIZE, metrics::LOGO_SIZE)));
        }
        ui.heading(name);
    });
    ui.add_space(metrics::HEADER_PAD);
}

/// A left-aligned, full-width, selectable sidebar row. Returns the
/// click [`Response`]; the caller owns the selection state.
pub fn nav_item(ui: &mut Ui, selected: bool, label: &str) -> Response {
    let height = 32.0;
    let response = ui.allocate_response(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    let visuals = ui.style().interact_selectable(&response, selected);
    if selected || response.hovered() {
        ui.painter().rect_filled(
            response.rect.expand(-2.0),
            egui::CornerRadius::same(metrics::RADIUS),
            visuals.bg_fill,
        );
    }
    let text_pos = response.rect.left_center() + egui::vec2(12.0, 0.0);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(14.0),
        visuals.text_color(),
    );
    response
}

/// Render a list of [`nav_item`]s bound to a selected index. Clicking a
/// row updates `selected`.
pub fn nav_list(ui: &mut Ui, selected: &mut usize, labels: &[&str]) {
    for (i, &label) in labels.iter().enumerate() {
        if nav_item(ui, *selected == i, label).clicked() {
            *selected = i;
        }
    }
}

/// The fixed-width left sidebar panel. Call before [`content`].
///
/// Prefer `sidebar` + [`content`] over [`app_shell`] when both halves
/// need `&mut` access to the same state: two separate calls borrow
/// sequentially, so the sidebar's `&mut self.section` is released before
/// the content closure reads it (a single `app_shell` call holds both
/// closures live at once and the borrow checker rejects the overlap).
pub fn sidebar(ctx: &Context, width: f32, add: impl FnOnce(&mut Ui)) {
    egui::SidePanel::left("kanso_sidebar")
        .resizable(false)
        .exact_width(width)
        .show(ctx, add);
}

/// The filling right content area. Call after [`sidebar`].
pub fn content(ctx: &Context, add: impl FnOnce(&mut Ui)) {
    egui::CentralPanel::default().show(ctx, add);
}

/// Convenience for the cohort's window shape — a fixed-width left
/// sidebar and a filling right content area — in one call.
///
/// Only use this when the two closures don't both mutate the same state;
/// otherwise use [`sidebar`] + [`content`] as separate statements (see
/// [`sidebar`] for why).
///
/// ```no_run
/// # fn demo(ctx: &kanso::egui::Context) {
/// kanso::widgets::app_shell(
///     ctx,
///     200.0,
///     |ui| { ui.heading("Nav"); },
///     |ui| { ui.heading("General"); },
/// );
/// # }
/// ```
pub fn app_shell(
    ctx: &Context,
    sidebar_width: f32,
    sidebar: impl FnOnce(&mut Ui),
    content: impl FnOnce(&mut Ui),
) {
    egui::SidePanel::left("kanso_sidebar")
        .resizable(false)
        .exact_width(sidebar_width)
        .show(ctx, sidebar);
    egui::CentralPanel::default().show(ctx, content);
}
