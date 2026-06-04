# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/jondkinney/kanso/compare/v0.1.1...v0.1.2) - 2026-06-04

### Other

- *(gallery)* roomier scroll-tuning sliders
- seed velocity from the inter-event interval, not the frame dt

## [0.1.1](https://github.com/jondkinney/kanso/compare/v0.1.0...v0.1.1) - 2026-06-03

### Other

- rustfmt the Scroll lab gallery
- *(gallery)* Scroll lab page + live tuning panel
- tune fling/over-scroll to macOS + add ScrollTuning knobs

## [0.1.0](https://github.com/jondkinney/kanso/releases/tag/v0.1.0) - 2026-06-03

### Added

- *(theme)* font-free apply_styles for per-frame use
- *(theme)* default to a solid (space-reserving) scrollbar
- *(metrics)* CONTROL_HEIGHT 28 -> 30 (cohort-canonical)
- *(theme)* public control_visuals for global adoption
- *(widgets)* shortcut chip, banner, card, link, about, dirty-footer + form controls
- *(widgets)* sidebar_header (logo + app name)
- *(theme)* constant-geometry control border + teal primary button
- initial kanso egui design system

### Fixed

- *(widgets)* banner info icon, segmented border, centered footer
- *(theme)* one standard corner RADIUS across all controls + containers
- *(theme)* scope control border to inputs/buttons; faithful code pill

### Other

- stop momentum on a touchpad hold gesture
- keep a swapped or shrunk view from rendering off-screen
- *(dirty_footer)* tighten the Save/Revert and dot/label gaps
- shrink the handle knob 25%
- *(gallery)* demo scroll_view + app_picker_combo
- *(app_picker)* searchable combo picker (app_picker_combo)
- kinetic fling + rubber-band over-scroll view
- *(app_picker)* optional per-entry icons
- add editable_combo (typeable combo box)
- add info_icon (circled-i marker + tooltip)
- add progress bar (determinate + indeterminate)
- *(link)* drop intra-doc link to private EXTERNAL_GLYPH
- use egui's standard font for body, reserve Adwaita Sans for modifier glyphs
- *(gallery)* showcase all widgets + a working dirty footer
- headless one-frame smoke test across all widgets

### Added

- Initial design system: `theme::apply`, bundled Adwaita Sans + optional
  omarchy `SHORTCUT_FAMILY`, named `palette`, layout `metrics`, and the
  settings-pane `widgets` (sidebar/content shell, nav rows, buttons,
  search, dropdown, app picker, faithful inline-code `caption` pills,
  field labels, and a `gallery` example).
