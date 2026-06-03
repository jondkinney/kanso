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
    "Scroll lab",
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
    scroll_image: Option<egui::TextureHandle>,
    show_scroll_tuning: bool,
    status: String,
}

impl Gallery {
    fn new() -> Self {
        Self {
            section: 12, // open on the Scroll lab for scroll-feel testing
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
            scroll_image: None,
            show_scroll_tuning: false,
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
                widgets::app_picker_combo(
                    ui,
                    "demo_apps",
                    &self.apps,
                    &mut self.selected_app,
                    &mut self.app_query,
                    "Choose an app…",
                );
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
            12 => {
                widgets::section_header(ui, "Scroll lab");

                let toggle = if self.show_scroll_tuning {
                    "Hide tuning panel"
                } else {
                    "Show tuning panel"
                };
                if ui.button(toggle).clicked() {
                    self.show_scroll_tuning = !self.show_scroll_tuning;
                }
                ui.add_space(8.0);

                widgets::card(ui, |ui| {
                    ui.label(
                        "This page is a scroll test bench. `kanso::scroll::scroll_view` \
                         owns its scroll offset and runs one controller for the whole \
                         gesture: drag → kinetic fling → rubber-band over-scroll at the \
                         edges. The velocity seed is frame-rate independent, so the feel \
                         is identical in a debug build and an optimized release build, \
                         and at any monitor refresh rate.",
                    );
                    ui.add_space(10.0);
                    widgets::caption(
                        ui,
                        "Tuning — click **Show tuning panel** above for a floating set of \
                         sliders that edit `kanso::scroll::ScrollTuning` live, so the page \
                         behind responds on the next flick. Find the feel, then bake the \
                         values back into the constants in `scroll.rs`.",
                    );
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("What to test").strong());
                    widgets::caption(
                        ui,
                        "1. Scroll velocity — flick and watch how far the coast carries.",
                    );
                    widgets::caption(
                        ui,
                        "2. Over-scroll flick — flick hard into the top or bottom for the rubber-band bounce.",
                    );
                    widgets::caption(
                        ui,
                        "3. Rest-to-stop — mid-coast, rest two fingers on the trackpad; it should halt instantly.",
                    );
                });
                ui.add_space(18.0);

                let lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed \
                    do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim \
                    ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut \
                    aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit \
                    in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur \
                    sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt \
                    mollit anim id est laborum.";
                let para = |ui: &mut egui::Ui, n: usize| {
                    for _ in 0..n {
                        ui.label(lorem);
                        ui.add_space(12.0);
                    }
                };

                // A generated gradient — cached on first show — gives the page real
                // pixel height to scroll past.
                let img = self
                    .scroll_image
                    .get_or_insert_with(|| make_scroll_image(ui.ctx()))
                    .clone();

                para(ui, 3);
                ui.add(egui::Image::new(&img).max_width(ui.available_width()));
                widgets::caption(ui, "Fig 1. A gradient with real height to fling past.");
                ui.add_space(16.0);
                para(ui, 3);

                code_block(
                    ui,
                    "// the fling seed — now frame-rate independent\n\
                     let alpha = 1.0 - (-dt / VEL_TAU).exp();\n\
                     self.vel_ema += (instant - self.vel_ema) * alpha;\n\
                     // ...on lift:\n\
                     if vel.abs() > FLING_MIN {\n    \
                         self.velocity = vel * FLING_SCALE;\n    \
                         self.state = ScrollState::Flinging;\n\
                     }",
                );
                ui.add_space(16.0);
                para(ui, 4);

                code_block(
                    ui,
                    "// rubber-band over-scroll (WebKit elasticity, capped at MAX_PULL)\n\
                     fn rb_pull(raw: f32, dim: f32) -> f32 {\n    \
                         let cap = MAX_PULL.min(dim);\n    \
                         raw.signum() * (1.0 - 1.0 / (raw.abs() * RB_STRETCH_C / cap + 1.0)) * cap\n\
                     }",
                );
                ui.add_space(16.0);
                para(ui, 3);
                ui.add(egui::Image::new(&img).max_width(ui.available_width()));
                ui.add_space(16.0);
                para(ui, 4);

                ui.add_space(24.0);
                widgets::caption(
                    ui,
                    "— end of scroll lab — flick back up to test the top edge —",
                );
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
        kanso::scroll::scroll_momentum(ctx);
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
            kanso::scroll::scroll_view(ui, "gallery_content", |ui| self.render_section(ui));
        });

        // Live scroll-feel tuning: a floating panel (outside the scroll view, so it
        // stays put while you fling the page behind it) that drives ScrollTuning.
        // Hidden by default — toggled from the Scroll lab page.
        if self.section == 12 && self.show_scroll_tuning {
            scroll_tuning_window(ctx);
        }
    }
}

/// Per-field lock flags for the tuning panel — a checked row is left untouched by
/// "Reset unlocked". Persisted in egui memory so it survives across frames.
#[derive(Clone, Copy, Default)]
struct TuningLocks {
    fling_gain: bool,
    fling_friction: bool,
    fling_knee: bool,
    fling_min: bool,
    vel_tau: bool,
    rb_amplitude: bool,
    rb_period: bool,
    max_pull: bool,
}

/// Floating sliders that edit the live [`kanso::scroll::ScrollTuning`] so the
/// scroll feel can be dialed in against a reference without rebuilding. Each row
/// has a lock so Reset only restores the values you're still experimenting with.
fn scroll_tuning_window(ctx: &egui::Context) {
    use kanso::scroll::{ScrollTuning, scroll_tuning, set_scroll_tuning};
    let locks_id = egui::Id::new("gallery_tuning_locks");
    let mut locks: TuningLocks = ctx.data_mut(|d| d.get_temp(locks_id).unwrap_or_default());
    let mut t = scroll_tuning(ctx);

    // One row = [lock] [slider] [label]. The label is a separate element so its
    // hover tooltip (the detailed description) fires only over the label text.
    fn row(ui: &mut egui::Ui, lock: &mut bool, slider: egui::Slider<'_>, label: &str, tip: &str) {
        ui.horizontal(|ui| {
            ui.checkbox(lock, "");
            ui.add(slider);
            ui.label(label).on_hover_text(tip);
        });
    }

    egui::Window::new("Scroll tuning")
        .default_pos([28.0, 96.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.style_mut().spacing.slider_width = 180.0;
            ui.label(
                egui::RichText::new(
                    "Check a box to lock that value — Reset only touches unlocked rows.",
                )
                .weak()
                .small(),
            );
            ui.add_space(4.0);

            ui.label(egui::RichText::new("Coast / fling").strong());
            row(
                ui,
                &mut locks.fling_gain,
                egui::Slider::new(&mut t.fling_gain, 0.0..=0.0015)
                    .custom_formatter(|v, _| format!("{v:.5}")),
                "gain — top-end coast",
                "The main top-end dial: how much farther hard flicks over-coast. \
                 Super-linear gain per px/s above the knee — it lengthens the decay τ \
                 for faster flicks so a hard throw carries disproportionately far \
                 (macOS-style). Raise if hard flicks fall short; lower if they run away.",
            );
            // Edit the glide as a time-constant (intuitive), convert to per-ms decay.
            let mut tau = -1.0 / (1000.0 * t.fling_friction.ln());
            row(
                ui,
                &mut locks.fling_friction,
                egui::Slider::new(&mut tau, 0.4..=2.5),
                "glide τ (s)",
                "Overall glide length / floatiness — the base fling decay shown as a \
                 time constant (FLING_FRICTION). Longer τ keeps every coast gliding \
                 longer before it settles; affects all flick speeds proportionally.",
            );
            t.fling_friction = (-1.0 / (1000.0 * tau)).exp();
            row(
                ui,
                &mut locks.fling_knee,
                egui::Slider::new(&mut t.fling_knee, 200.0..=1000.0),
                "super-linear knee",
                "Flick speed (px/s) where the top-end boost begins. Below it flings \
                 coast ≈ 1:1 with your finger; above it the gain kicks in. Raise the \
                 knee so only genuinely hard flicks get the extra carry.",
            );
            row(
                ui,
                &mut locks.fling_min,
                egui::Slider::new(&mut t.fling_min, 50.0..=800.0),
                "fling min (mini-flick)",
                "Minimum lift speed (px/s) that starts a fling. Lower so smaller quick \
                 flicks still coast; too low and a slow controlled scroll coasts when \
                 you meant it to stop where you lifted.",
            );
            row(
                ui,
                &mut locks.vel_tau,
                egui::Slider::new(&mut t.vel_tau, 0.004..=0.030)
                    .custom_formatter(|v, _| format!("{v:.3}")),
                "vel τ (seed window)",
                "Velocity-seed averaging window (s). Smaller captures the sharp peak of \
                 a fast flick (punchier, twitchier); larger averages more (smoother, but \
                 laggier and it under-reads a quick snap). Frame-rate independent.",
            );

            ui.separator();
            ui.label(egui::RichText::new("Over-scroll bounce").strong());
            row(
                ui,
                &mut locks.rb_amplitude,
                egui::Slider::new(&mut t.rb_amplitude, 0.2..=3.0),
                "amplitude — shoot/far",
                "How fast AND far the edge bounce throws — sets both the initial \
                 shoot-in velocity (∝ v·A) and the peak travel. Raise to make the \
                 rubber-band reach farther and snap out harder.",
            );
            row(
                ui,
                &mut locks.rb_period,
                egui::Slider::new(&mut t.rb_period, 0.6..=2.5),
                "period — snap speed",
                "Bounce timing: peak at t* = period/stiffness, settle in ~4·t*. Lower = \
                 snappier (reaches peak and springs back quicker); higher = a larger, \
                 calmer, slower bounce.",
            );
            row(
                ui,
                &mut locks.max_pull,
                egui::Slider::new(&mut t.max_pull, 20.0..=150.0),
                "manual pull cap (px)",
                "Slow-drag overscroll asymptote — how far a deliberate pull past the \
                 edge can stretch (approached but never quite reached). Independent of \
                 the velocity bounce above.",
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Reset unlocked").clicked() {
                    let d = ScrollTuning::default();
                    if !locks.fling_gain {
                        t.fling_gain = d.fling_gain;
                    }
                    if !locks.fling_friction {
                        t.fling_friction = d.fling_friction;
                    }
                    if !locks.fling_knee {
                        t.fling_knee = d.fling_knee;
                    }
                    if !locks.fling_min {
                        t.fling_min = d.fling_min;
                    }
                    if !locks.vel_tau {
                        t.vel_tau = d.vel_tau;
                    }
                    if !locks.rb_amplitude {
                        t.rb_amplitude = d.rb_amplitude;
                    }
                    if !locks.rb_period {
                        t.rb_period = d.rb_period;
                    }
                    if !locks.max_pull {
                        t.max_pull = d.max_pull;
                    }
                }
                ui.label(
                    egui::RichText::new("live — flick the page behind")
                        .weak()
                        .small(),
                );
            });
        });

    ctx.data_mut(|d| d.insert_temp(locks_id, locks));
    set_scroll_tuning(ctx, t);
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

/// A generated gradient placeholder image for the Scroll lab — real pixels with
/// real height so there's something to fling past (no external asset needed).
fn make_scroll_image(ctx: &egui::Context) -> egui::TextureHandle {
    let (w, h) = (560usize, 280usize);
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            let r = (40 + x * 180 / w) as u8;
            let g = (60 + y * 150 / h) as u8;
            let b = (120 + (x + y) * 90 / (w + h)) as u8;
            rgba[i..i + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    let image = egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba);
    ctx.load_texture("scroll-lab-image", image, egui::TextureOptions::LINEAR)
}

/// A monospace code snippet inside a kanso card — for the Scroll lab page.
fn code_block(ui: &mut egui::Ui, code: &str) {
    widgets::card(ui, |ui| {
        ui.label(egui::RichText::new(code).monospace());
    });
}
