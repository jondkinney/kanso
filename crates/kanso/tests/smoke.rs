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

#[derive(PartialEq, Clone)]
enum Pick {
    A,
    B,
    C,
}

/// Run every NEW widget (shortcut chips, banners, card, links, about pane,
/// the form controls, and the dirty footer) through one real headless frame
/// so a painter/galley panic fails CI without a GPU.
#[test]
fn new_widgets_render_without_panic() {
    use kanso::widgets::{AboutInfo, Dirty, Severity};

    let ctx = egui::Context::default();
    kanso::Theme {
        fonts: fonts::FontOptions {
            shortcut_family: true,
            ..Default::default()
        },
        ..Default::default()
    }
    .apply(&ctx);

    let mut on = true;
    let mut secret = String::from("hunter2");
    let mut token = String::from("ghp_x");
    let mut tol = 0.4f32;
    let mut delay_ms = 12u32;
    let mut delay_ms2 = 8u32;
    let mut count = 5i32;
    let mut col = egui::Color32::from_rgb(79, 163, 255);
    let mut col2 = egui::Color32::from_rgb(80, 200, 120);
    let mut pick = Pick::A;
    let mut seg = Pick::B;
    let mut settings = Dirty::new(3i32);
    settings.edited = 4; // make it dirty so the footer renders the indicator

    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let logo = ctx.load_texture(
            "smoke-logo2",
            egui::ColorImage::from_rgba_unmultiplied([2, 2], &[0u8; 16]),
            egui::TextureOptions::LINEAR,
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            // shortcut chips (custom glyph painting via SHORTCUT_FAMILY)
            widgets::shortcut_chip(ui, &["SUPER", "SHIFT", "K"]);
            widgets::shortcut_chip_str(ui, "CTRL+ALT+ENTER");
            widgets::shortcut_capture_chip(ui, "CTRL+S", false);
            widgets::shortcut_capture_chip(ui, "", true);
            let _ = widgets::token_glyph("SUPER");
            // banners (hand-painted tinted strips)
            widgets::banner(ui, Severity::Info, "i", "b");
            widgets::banner(ui, Severity::Warn, "w", "b");
            widgets::banner(ui, Severity::Error, "e", "b");
            widgets::banner(ui, Severity::Ok, "o", "b");
            let _ = widgets::banner_action(ui, Severity::Warn, "t", "b", "Do it");
            // card
            widgets::card(ui, |ui| {
                ui.label("inside a card");
            });
            // links
            let _ = widgets::link(ui, "Open Settings");
            let _ = widgets::external_link(ui, "Docs");
            let _ = widgets::link_to(ui, "Repo", "https://example.com");
            // about pane (image + hero)
            widgets::about_pane(
                ui,
                AboutInfo {
                    logo: Some(egui::Image::new(&logo)),
                    name: "kanso",
                    version: "0.1.0",
                    blurb: Some("design system"),
                    links: &[("Home", "https://example.com")],
                },
            );
            // form controls
            widgets::toggle(ui, &mut on, "Launch at login");
            widgets::labeled_toggle(ui, "Review", &mut on, "vim mode", "caption `x`");
            widgets::password_field(ui, &mut secret);
            widgets::password_field_revealable(ui, "tok", &mut token);
            widgets::slider(ui, &mut tol, 0.0..=1.0, "Tolerance");
            widgets::Slider::new(&mut delay_ms, 0..=30)
                .label("Delay")
                .suffix(" ms")
                .show(ui);
            widgets::number_stepper(ui, &mut count, 0..=64, 1);
            widgets::NumberStepper::new(0..=30, 1)
                .suffix(" ms")
                .show(ui, &mut delay_ms2);
            widgets::radio_group(
                ui,
                &mut pick,
                &[(Pick::A, "A"), (Pick::B, "B"), (Pick::C, "C")],
            );
            let _ = widgets::segmented(ui, &mut seg, &[(Pick::A, "A"), (Pick::B, "B")]);
            widgets::color_picker(ui, &mut col, "Primary");
            widgets::color_picker_opaque(ui, &mut col2, "Guide");
            // dirty footer (painter dot + Save/Revert)
            let _ = widgets::dirty_footer(&settings).show(ui);
        });
    });
}
