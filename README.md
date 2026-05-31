# kanso

> 簡素 — *simplicity, the elimination of clutter.* A small shared **egui
> design system** for the desktop utility-app cohort.

`kanso` pulls the pieces that vernier and hyprcorrect's preferences
windows were each re-implementing into one dependency: the theme, the
bundled font, a named color palette, layout metrics, and the recurring
settings-pane widgets. New egui utilities get the cohort look on day one.

> **Working name.** `kanso` is free on crates.io but easily renamed —
> nothing is published yet.

## Why egui-only

The four apps split across **two UI toolkits**:

| App | Toolkit | kanso? |
|-----|---------|--------|
| **vernier** | egui / eframe 0.32 | ✅ target |
| **hyprcorrect** | egui / eframe 0.32 | ✅ target |
| mousehop | GTK4 + libadwaita | ✗ different rendering model |
| tensaku | relm4 + gtk4 + femtovg | ✗ different rendering model |

egui and GTK are different rendering models — a GTK app gets nothing from
an egui widget crate. So kanso is deliberately **egui-only**. The things
the GTK apps *do* share with the egui apps are toolkit-neutral, non-UI
logic (chord-string formatting, `.desktop` enumeration, xdg paths) — those
belong in a *separate* crate, never here. See `REPORT.md`.

## What's inside

```rust
kanso::theme::apply(ctx);          // fonts + type scale + spacing (rides egui's dark Visuals)

use kanso::{palette, metrics, widgets};

widgets::sidebar(ctx, metrics::SIDEBAR_WIDTH, |ui| {   // left nav…
    widgets::nav_list(ui, &mut section, &["General", "Shortcuts", "About"]);
});
widgets::content(ctx, |ui| {                            // …right content
    widgets::section_header(ui, "General");
    widgets::setting(ui, |ui| {
        widgets::field_label(ui, "Display name");
        widgets::padded_text_edit(ui, &mut name);
        widgets::caption(ui, "Leave blank to use `$USER`.");   // backtick → inline pill
    });
    widgets::primary_button(ui, "Save");
});
```

- **`theme`** — one call installs the font, text-style scale (heading 21 /
  body 14 / mono 13 / small 12), spacing, and button padding. It *rides
  egui's default dark Visuals* and overrides nothing else, so dropping it
  into an app does **not** restyle any surface.
- **`fonts`** — bundles Adwaita Sans once (the apps each `include_bytes!`'d
  their own copy) and, optionally, builds the `SHORTCUT_FAMILY` for
  keyboard-chip glyphs incl. the omarchy SUPER logo at `OMARCHY_LOGO`.
- **`palette`** — named colors reconciled to the apps' real literals
  (`TEXT`, `TEXT_MUTED`, `CARD`, `CODE_BG`, `ACCENT`, `ERROR`, `WARN`,
  `INFO`, `CAPTURE`, …) so adoption is pixel-faithful.
- **`metrics`** — spacing / sizing / corner-radius constants.
- **`widgets`** — `sidebar`/`content`/`app_shell`, `nav_item`/`nav_list`,
  `primary_button`/`secondary_button`, `search_field`, `dropdown`,
  `app_picker` (searchable "select a running app"), `caption` (inline
  backtick code pills, glyph-metric-hugging), `code`/`code_span`,
  `field_label`, `section_header`, `setting`, `padded_text_edit`.

The egui version is re-exported as `kanso::egui` so consumers pin the
exact egui kanso was built against.

## Gallery

```bash
cargo run --example gallery
```

A live tour of the theme and every widget.

## Adoption

Until `kanso 0.1.0` is published to crates.io, consume it via a **path
dependency** (the consumer `*-ui` crates publish to crates.io, so kanso
must land on the registry *before* any consumer is tagged — otherwise it
hits the cohort's known release-pr/publish race). Adoption swaps each
app's hand-rolled `install_glyph_fonts`/`apply_style`/`sidebar_item`/
caption/`field_label`/`padded_text_edit` for the kanso equivalents and
deletes the duplicated `AdwaitaSans-Regular.ttf`.

## Roadmap

Shipped (wave 1): theme, fonts (+omarchy), palette, metrics, the widgets
above, gallery.

Wave 2 (needs a 2nd real adoption to shape the API): the chord-capture
chip, a `SettingsWindow` save/revert + dirty-state footer, severity
`banner`/`card`/`status_dot`/`about_pane`.

Out of scope (rule-of-three / wrong layer): GTK widgets, single-app
domain UI (vim-edit, word-diff, the annotation canvas), and the
toolkit-neutral non-UI logic crates.

## License

MIT OR Apache-2.0.
