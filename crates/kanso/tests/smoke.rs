//! Headless smoke test: run one real egui frame through every widget so a
//! font-family-unbound panic or a galley-walk bug in the caption pill
//! painter fails CI — without needing a GPU or a window.

use kanso::egui::FontFamily;
use kanso::{egui, fonts, metrics, widgets};

#[test]
fn widgets_render_one_frame_without_panic() {
    let ctx = egui::Context::default();
    // shortcut_family = true exercises the custom-font-family path (the
    // family must be bound even when omarchy.ttf is absent in CI).
    let theme = kanso::Theme {
        fonts: fonts::FontOptions {
            shortcut_family: true,
            ..Default::default()
        },
        ..Default::default()
    };
    theme.apply(&ctx);

    let apps = vec![
        widgets::AppEntry::new("firefox", "Firefox"),
        widgets::AppEntry::new("ghostty", "Ghostty"),
    ];
    let mut nav = 0usize;
    let mut mode = 1usize;
    let mut name = String::from("hello");
    let mut search = String::from("fire");
    let mut selected: Option<String> = None;
    let mut query = String::new();

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let logo = ctx.load_texture(
            "smoke-logo",
            egui::ColorImage::from_rgba_unmultiplied([2, 2], &[0u8; 16]),
            egui::TextureOptions::LINEAR,
        );
        widgets::sidebar(ctx, metrics::SIDEBAR_WIDTH, |ui| {
            widgets::sidebar_header(ui, Some(egui::Image::new(&logo)), "kanso");
            widgets::nav_list(ui, &mut nav, &["A", "B", "C"]);
        });
        widgets::content(ctx, |ui| {
            widgets::section_header(ui, "Smoke");
            widgets::field_label(ui, "Field");
            widgets::padded_text_edit(ui, &mut name);
            // The faithful caption pill painter (backtick → code) — the
            // riskiest widget at runtime.
            widgets::caption(ui, "Press `Super + Space`, then run `vernier --daemon`.");
            widgets::primary_button(ui, "Save");
            widgets::secondary_button(ui, "Cancel");
            widgets::search_field(ui, &mut search, "search");
            widgets::dropdown(ui, "mode", &mut mode, &[(0usize, "Zero"), (1usize, "One")]);
            widgets::app_picker(ui, &apps, &mut selected, &mut query);
            widgets::code(ui, "~/.config/x.toml");
            // The shortcut family must be bound (no panic) even without omarchy.
            ui.label(
                egui::RichText::new("⌃⇧⌥").family(FontFamily::Name(fonts::SHORTCUT_FAMILY.into())),
            );
        });
    });
}

#[test]
fn app_entry_round_trips() {
    let e = widgets::AppEntry::new("id-1", "Display Name");
    assert_eq!(e.id, "id-1");
    assert_eq!(e.name, "Display Name");
}
