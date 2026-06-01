//! A live gallery of kanso's theme + widgets.
//!
//! ```text
//! cargo run --example gallery
//! ```

use kanso::egui::Color32;
use kanso::widgets::{AboutInfo, Dirty, FooterAction, Severity};
use kanso::{egui, fonts, metrics, widgets};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([860.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "kanso gallery",
        options,
        Box::new(|cc| {
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
enum Provider {
    Spellbook,
    LanguageTool,
    Llm,
}

#[derive(PartialEq, Clone)]
enum Density {
    Compact,
    Cozy,
    Comfortable,
}

/// The persisted settings the dirty footer tracks. Editing any field in a
/// section dirties the footer; Save/Revert act on it.
#[derive(Clone, PartialEq)]
struct DemoSettings {
    name: String,
    launch_at_login: bool,
    review_vim: bool,
    tolerance: f32,
    delay_ms: u32,
    margin: i32,
    provider: Provider,
    density: Density,
    primary: Color32,
    guide: Color32,
    api_key: String,
    model: String,
}

impl Default for DemoSettings {
    fn default() -> Self {
        Self {
            name: String::new(),
            launch_at_login: true,
            review_vim: false,
            tolerance: 0.4,
            delay_ms: 12,
            margin: 8,
            provider: Provider::Spellbook,
            density: Density::Cozy,
            primary: Color32::from_rgb(79, 163, 255),
            guide: Color32::from_rgb(80, 200, 120),
            api_key: String::new(),
            model: "claude-opus-4-8".to_string(),
        }
    }
}

const SECTIONS: &[&str] = &[
    "Buttons",
    "Inputs",
    "Shortcuts",
    "Banners",
    "Cards & Links",
    "Toggles",
    "Sliders",
    "Pickers",
    "Apps",
    "Captions",
    "Progress",
    "About",
];

struct Gallery {
    section: usize,
    settings: Dirty<DemoSettings>,
    hotkey: String,
    capturing: bool,
    apps: Vec<widgets::AppEntry>,
    selected_app: Option<String>,
    app_query: String,
    search: String,
    logo: Option<egui::TextureHandle>,
    status: String,
}

impl Gallery {
    fn new() -> Self {
        Self {
            section: 0,
            settings: Dirty::new(DemoSettings::default()),
            hotkey: "SUPER+SHIFT+K".to_string(),
            capturing: false,
            apps: vec![
                widgets::AppEntry::new("firefox", "Firefox"),
                widgets::AppEntry::new("ghostty", "Ghostty"),
                widgets::AppEntry::new("code", "VS Code"),
                widgets::AppEntry::new("figma", "Figma"),
            ],
            selected_app: None,
            app_query: String::new(),
            search: String::new(),
            logo: None,
            status: String::new(),
        }
    }

    fn render_section(&mut self, ui: &mut egui::Ui) {
        let s = &mut self.settings.edited;
        match self.section {
            0 => {
                widgets::section_header(ui, "Buttons");
                ui.horizontal(|ui| {
                    widgets::primary_button(ui, "Save");
                    widgets::secondary_button(ui, "Cancel");
                });
            }
            1 => {
                widgets::section_header(ui, "Inputs");
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Display name");
                    widgets::padded_text_edit(ui, &mut s.name);
                    widgets::caption(ui, "Shown in the title bar. Leave blank to use `$USER`.");
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Search");
                    widgets::search_field(ui, &mut self.search, "Filter…");
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "API key");
                    widgets::password_field(ui, &mut s.api_key);
                    widgets::caption(ui, "Stored in the system keychain, never in `config.toml`.");
                });
            }
            2 => {
                widgets::section_header(ui, "Shortcuts");
                ui.horizontal(|ui| {
                    widgets::field_label(ui, "Toggle overlay");
                    widgets::shortcut_chip(ui, &["SUPER", "SHIFT", "K"]);
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    widgets::field_label(ui, "Apply correction");
                    widgets::shortcut_chip_str(ui, "CTRL+ALT+ENTER");
                });
                ui.add_space(12.0);
                widgets::caption(ui, "Click the chip to record a new binding:");
                ui.horizontal(|ui| {
                    widgets::field_label(ui, "Fix word");
                    if widgets::shortcut_capture_chip(ui, &self.hotkey, self.capturing).clicked() {
                        self.hotkey.clear();
                        self.capturing = true;
                    }
                });
            }
            3 => {
                widgets::section_header(ui, "Banners");
                widgets::banner(ui, Severity::Info, "Heads up", "An informational callout.");
                ui.add_space(8.0);
                if widgets::banner_action(
                    ui,
                    Severity::Warn,
                    "Screen Recording is off",
                    "Can track the cursor but can't snap to pixels until granted.",
                    "Open System Settings",
                ) {
                    self.status = "Would open System Settings…".to_string();
                }
                ui.add_space(8.0);
                widgets::banner(
                    ui,
                    Severity::Error,
                    "LLM unavailable",
                    "No API key configured.",
                );
                ui.add_space(8.0);
                widgets::banner(ui, Severity::Ok, "All set", "LanguageTool is reachable.");
            }
            4 => {
                widgets::section_header(ui, "Cards & Links");
                widgets::card(ui, |ui| {
                    widgets::field_label(ui, "Figma integration");
                    widgets::caption(ui, "Paste a `figma://` link to sync.");
                    ui.add_space(8.0);
                    widgets::padded_text_edit(ui, &mut self.search);
                });
                ui.add_space(12.0);
                if widgets::link(ui, "Open System Settings").clicked() {
                    self.status = "link clicked".to_string();
                }
                ui.add_space(6.0);
                widgets::link_to(
                    ui,
                    "github.com/jondkinney/kanso",
                    "https://github.com/jondkinney/kanso",
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label("Pulls the");
                    widgets::external_link(ui, "erikvl87/languagetool");
                    ui.label("image.");
                });
            }
            5 => {
                widgets::section_header(ui, "Toggles");
                widgets::setting(ui, |ui| {
                    widgets::toggle(ui, &mut s.launch_at_login, "Launch at login");
                    widgets::caption(ui, "Adds an autostart entry; removed on save if unchecked.");
                });
                widgets::setting(ui, |ui| {
                    widgets::labeled_toggle(
                        ui,
                        "Review popup",
                        &mut s.review_vim,
                        "Open in vim mode",
                        "Start the review popup in vim mode. `Ctrl+E` toggles either way.",
                    );
                });
            }
            6 => {
                widgets::section_header(ui, "Sliders & steppers");
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Edge tolerance");
                    widgets::slider(ui, &mut s.tolerance, 0.0..=1.0, "Tolerance");
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Pause per backspace");
                    widgets::Slider::new(&mut s.delay_ms, 0..=30)
                        .label("Delay")
                        .suffix(" ms")
                        .show(ui);
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Context margin");
                    widgets::number_stepper(ui, &mut s.margin, 0..=64, 1);
                });
            }
            7 => {
                widgets::section_header(ui, "Pickers");
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Default provider");
                    widgets::radio_group(
                        ui,
                        &mut s.provider,
                        &[
                            (Provider::Spellbook, "Spellbook (offline)"),
                            (Provider::LanguageTool, "LanguageTool"),
                            (Provider::Llm, "LLM"),
                        ],
                    );
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Density");
                    widgets::segmented(
                        ui,
                        &mut s.density,
                        &[
                            (Density::Compact, "Compact"),
                            (Density::Cozy, "Cozy"),
                            (Density::Comfortable, "Comfortable"),
                        ],
                    );
                });
                widgets::setting(ui, |ui| {
                    widgets::field_label(ui, "Model");
                    widgets::editable_combo(
                        ui,
                        "demo_model",
                        &mut s.model,
                        &["claude-haiku-4-5", "claude-sonnet-4-6", "claude-opus-4-8"],
                        "Pick or type a model",
                    );
                    widgets::caption(ui, "Editable — pick a suggestion or type your own.");
                });
                widgets::setting(ui, |ui| {
                    widgets::color_picker(ui, &mut s.primary, "Primary color");
                    widgets::color_picker_opaque(ui, &mut s.guide, "Guide color");
                });
            }
            8 => {
                widgets::section_header(ui, "Select a running app");
                // Demo: give a couple of entries an icon (reuse the sidebar
                // logo texture); the rest show the placeholder tile.
                if let Some(id) = self.logo.as_ref().map(|h| h.id()) {
                    self.apps[0].icon = Some(id);
                    self.apps[2].icon = Some(id);
                }
                widgets::app_picker(ui, &self.apps, &mut self.selected_app, &mut self.app_query);
                if let Some(id) = self.selected_app.clone() {
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Selected:");
                        widgets::code(ui, &id);
                    });
                }
            }
            9 => {
                widgets::section_header(ui, "Captions & code");
                widgets::caption(
                    ui,
                    "Press `Super + Space` to open the launcher, or run `vernier --daemon`.",
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label("Config lives at");
                    widgets::code(ui, "~/.config/vernier.toml");
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    widgets::field_label(ui, "Default provider");
                    widgets::info_icon(ui, "A circled-i marker with a tooltip — hover me.");
                });
            }
            10 => {
                widgets::section_header(ui, "Progress");
                widgets::progress(ui, 0.42, "Downloading 6.7 / 16.0 GB");
                ui.add_space(12.0);
                widgets::ProgressBar::indeterminate()
                    .text("Unzipping (~16 GB)…")
                    .show(ui);
                ui.add_space(16.0);
                widgets::caption(ui, "A slim, label-less rail:");
                widgets::ProgressBar::new(0.7).height(6.0).show(ui);
            }
            _ => {
                let logo = self.logo.clone();
                widgets::about_pane(
                    ui,
                    AboutInfo {
                        logo: logo.as_ref().map(egui::Image::new),
                        name: "kanso",
                        version: env!("CARGO_PKG_VERSION"),
                        blurb: Some("A shared egui design system for the desktop app cohort."),
                        links: &[
                            ("Repository", "https://github.com/jondkinney/kanso"),
                            ("License", "https://spdx.org/licenses/MIT.html"),
                        ],
                    },
                );
            }
        }
    }
}

impl eframe::App for Gallery {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        widgets::sidebar(ctx, metrics::SIDEBAR_WIDTH, |ui| {
            let logo = self.logo.get_or_insert_with(|| make_logo(ui.ctx()));
            widgets::sidebar_header(ui, Some(egui::Image::new(&*logo)), "kanso");
            widgets::nav_list(ui, &mut self.section, SECTIONS);
        });

        egui::TopBottomPanel::bottom("gallery_footer")
            .exact_height(56.0)
            .show(ctx, |ui| {
                // horizontal_centered fills the panel height and centers its
                // row vertically, so Save/Revert sit centered in the bar.
                ui.horizontal_centered(|ui| {
                    match widgets::dirty_footer(&self.settings)
                        .revert_enabled(self.capturing)
                        .show(ui)
                    {
                        FooterAction::Save => {
                            self.settings.mark_saved();
                            self.status = "Saved.".to_string();
                        }
                        FooterAction::Revert => {
                            self.settings.revert();
                            self.capturing = false;
                            self.status = "Reverted.".to_string();
                        }
                        FooterAction::None => {}
                    }
                });
            });

        widgets::content(ctx, |ui| {
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| self.render_section(ui));
        });
    }
}

/// A throwaway placeholder logo (a filled teal disc) so the gallery can
/// demo `sidebar_header` / `about_pane` with an image. Real apps pass their
/// own rendered icon texture.
fn make_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let size = 64usize;
    let teal = [0u8, 92, 128, 255];
    let mut rgba = vec![0u8; size * size * 4];
    let center = (size as f32 - 1.0) / 2.0;
    let radius = size as f32 * 0.46;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let i = (y * size + x) * 4;
                rgba[i..i + 4].copy_from_slice(&teal);
            }
        }
    }
    let image = egui::ColorImage::from_rgba_unmultiplied([size, size], &rgba);
    ctx.load_texture("kanso-demo-logo", image, egui::TextureOptions::LINEAR)
}
