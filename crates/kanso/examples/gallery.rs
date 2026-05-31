//! A live gallery of kanso's theme + widgets.
//!
//! ```text
//! cargo run --example gallery
//! ```

use kanso::egui::FontFamily;
use kanso::{egui, fonts, metrics, widgets};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([780.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "kanso gallery",
        options,
        Box::new(|cc| {
            // Enable the shortcut family so the omarchy SUPER logo renders
            // if omarchy.ttf is installed on this machine.
            let theme = kanso::Theme {
                fonts: fonts::FontOptions {
                    shortcut_family: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            theme.apply(&cc.egui_ctx);
            Ok(Box::new(Gallery::new()))
        }),
    )
}

#[derive(PartialEq, Clone)]
enum Mode {
    Fast,
    Balanced,
    Precise,
}

struct Gallery {
    section: usize,
    name: String,
    apps: Vec<widgets::AppEntry>,
    selected_app: Option<String>,
    app_query: String,
    mode: Mode,
}

impl Gallery {
    fn new() -> Self {
        Self {
            section: 0,
            name: String::new(),
            apps: vec![
                widgets::AppEntry::new("firefox", "Firefox"),
                widgets::AppEntry::new("ghostty", "Ghostty"),
                widgets::AppEntry::new("code", "VS Code"),
                widgets::AppEntry::new("figma", "Figma"),
            ],
            selected_app: None,
            app_query: String::new(),
            mode: Mode::Balanced,
        }
    }

    fn inputs_section(&mut self, ui: &mut egui::Ui) {
        widgets::section_header(ui, "Inputs");
        widgets::setting(ui, |ui| {
            widgets::field_label(ui, "Display name");
            widgets::padded_text_edit(ui, &mut self.name);
            widgets::caption(ui, "Shown in the title bar. Leave blank to use `$USER`.");
        });
        widgets::setting(ui, |ui| {
            widgets::field_label(ui, "Precision");
            widgets::dropdown(
                ui,
                "mode",
                &mut self.mode,
                &[
                    (Mode::Fast, "Fast"),
                    (Mode::Balanced, "Balanced"),
                    (Mode::Precise, "Precise"),
                ],
            );
        });
    }

    fn apps_section(&mut self, ui: &mut egui::Ui) {
        widgets::section_header(ui, "Select a running app");
        widgets::app_picker(ui, &self.apps, &mut self.selected_app, &mut self.app_query);
        if let Some(id) = self.selected_app.clone() {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Selected:");
                widgets::code(ui, &id);
            });
        }
    }
}

fn buttons_section(ui: &mut egui::Ui) {
    widgets::section_header(ui, "Buttons");
    ui.horizontal(|ui| {
        widgets::primary_button(ui, "Save");
        widgets::secondary_button(ui, "Cancel");
    });
}

fn captions_section(ui: &mut egui::Ui) {
    widgets::section_header(ui, "Captions & code");
    widgets::caption(
        ui,
        "Press `Super + Space` to open the launcher, or run `vernier --daemon` \
         to start the measurement overlay. Backtick spans become inline pills.",
    );
    ui.add_space(12.0);
    widgets::field_label(ui, "Inline code");
    ui.horizontal(|ui| {
        ui.label("Config lives at");
        widgets::code(ui, "~/.config/vernier.toml");
    });
    ui.add_space(12.0);
    widgets::field_label(ui, "Shortcut family");
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("⌃⇧⌥")
                .family(FontFamily::Name(fonts::SHORTCUT_FAMILY.into()))
                .size(16.0),
        );
        if fonts::omarchy_available() {
            ui.label(
                egui::RichText::new(fonts::OMARCHY_LOGO.to_string())
                    .family(FontFamily::Name(fonts::SHORTCUT_FAMILY.into()))
                    .size(16.0),
            );
            ui.label("omarchy SUPER logo loaded");
        } else {
            ui.label(widgets::code_span("omarchy.ttf not installed"));
        }
    });
}

impl eframe::App for Gallery {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        widgets::sidebar(ctx, metrics::SIDEBAR_WIDTH, |ui| {
            ui.add_space(8.0);
            ui.heading("kanso");
            ui.add_space(8.0);
            widgets::nav_list(
                ui,
                &mut self.section,
                &["Buttons", "Inputs", "Apps", "Captions"],
            );
        });
        widgets::content(ctx, |ui| {
            ui.add_space(8.0);
            match self.section {
                0 => buttons_section(ui),
                1 => self.inputs_section(ui),
                2 => self.apps_section(ui),
                _ => captions_section(ui),
            }
        });
    }
}
