# UI abstraction across the utility-app cohort — audit & plan

*Prepared overnight. Scope: vernier, mousehop, tensaku, hyprcorrect — and
the apps you'll add next.*

## TL;DR

- The four apps split across **two UI toolkits**, and that split decides
  everything that follows:
  - **egui / eframe 0.32** — **vernier**, **hyprcorrect**. Heavy, nearly
    identical preferences windows (vernier `prefs.rs` 3.3k lines,
    hyprcorrect `hyprcorrect-ui` ~8k). This is where the duplication is.
  - **GTK4 / libadwaita** — **mousehop** (plain `adw`), **tensaku**
    (relm4 + femtovg, a satty-style annotation tool). Different rendering
    model; an egui widget crate gives them **nothing**.
- I built and verified a shared egui design system, **`kanso`**, next to
  the other repos (`~/Projects/kanso`), implementing the whole wishlist
  (theme, fonts, dropdowns, search, select-running-app, buttons, backtick
  monospace highlights, left-sidebar + right-content shell), and **proved
  it adopts cleanly** into hyprcorrect (compiles).
- The toolkit-neutral duplication that *also* affects the GTK apps
  (chord-string formatting, `.desktop` enumeration, xdg paths) is real but
  belongs in **separate non-UI crates**, never in kanso.

Everything below is grounded in the code (10-agent audit + my own reads);
nothing was pushed; the hyprcorrect trial is on a branch.

---

## 1. The decisive finding: two toolkit families

| App | Toolkit | Shareable surface |
|-----|---------|-------------------|
| **vernier** | egui/eframe 0.32 | ✅ egui widgets + tokens + logic |
| **hyprcorrect** | egui/eframe 0.32 | ✅ egui widgets + tokens + logic |
| **mousehop** | GTK4 + libadwaita | tokens (in spirit) + non-UI logic only |
| **tensaku** | relm4 + gtk4 + femtovg | almost nothing (annotation canvas is irreducible) |

A shared **egui** component crate serves vernier + hyprcorrect + every
future egui utility. The GTK apps can't consume egui widgets — their only
overlap with the egui apps is **non-UI logic** and design *values*
(colors, the Adwaita font), not widgets.

**Implication:** don't chase "one UI library for all four." Build the egui
design system (big, immediate win for 2 + future apps) and, separately,
extract the toolkit-neutral logic (smaller, serves all four).

---

## 2. Where the duplication actually is

### 2a. egui chrome — vernier ↔ hyprcorrect (the main event)

The two preferences windows re-implement the same things, often
**byte-for-byte** (hyprcorrect's source literally says "Mirrors vernier"):

| Duplicated thing | vernier | hyprcorrect | Verdict |
|---|---|---|---|
| Font install (bundled AdwaitaSans + omarchy SUPER glyph) | `prefs.rs:975` | `prefs.rs:2882` | identical pattern |
| Theme/style (text scale + spacing) | `prefs.rs:930` | `prefs.rs:3062` | identical |
| Left-aligned sidebar nav row | `prefs.rs:1041` | `prefs.rs:2626` | **byte-identical** |
| Sidebar + content + footer shell | `prefs.rs:548` | (same shape) | identical shape |
| Inline backtick → code pill caption | `prefs.rs:3056` | `prefs.rs:2740` | near-identical (90+ lines each) |
| `field_label` / `padded_text_edit` / `setting` spacer | `:3047`/`:3203`/`:3001` | `:2690`/`:2667`/… | identical-ish (cosmetic drift) |
| Bundled `AdwaitaSans-Regular.ttf` | `vernier-ui/assets` | `hyprcorrect-ui/assets` | same file, also in mousehop (~2.5 MB triplicated) |
| Ad-hoc `Color32::from_gray(N)` palette | scattered | scattered | same values, no token module |

### 2b. Toolkit-neutral logic — spans the egui *and* GTK apps

| Logic | Where | Apps |
|---|---|---|
| Canonical `CTRL/SHIFT/ALT/SUPER` chord string formatter/parser | vernier `chord_capture.rs`, hyprcorrect `capture.rs` (**byte-identical**), mousehop chord chip | vernier, hyprcorrect, mousehop |
| `.desktop` / running-app enumeration → {id, name, icon} | hyprcorrect `apps.rs`/`docker.rs`, mousehop `desktop_entries.rs`/`frontmost_app.rs` | hyprcorrect, mousehop |
| xdg paths / single-instance / hyprctl-ipc / ksni tray wrappers | several | all four |

### 2c. Genuinely app-specific — do **not** share (rule-of-three)

vim-edit overlay, word-diff/squiggle review renderer, docker/n-gram/LLM
provider UIs (hyprcorrect); tick-slider, color-picker, Figma card
(vernier); the femtovg annotation canvas + tool geometry (tensaku).

---

## 3. What I built: `kanso` (egui design system)

`~/Projects/kanso` — a single-crate Cargo workspace mirroring the cohort's
conventions (edition 2024, rust 1.85, MIT OR Apache-2.0, release-plz + CI,
**with today's `needs: release` gate already baked in**). Status:
**builds clean, clippy `-D warnings` clean, example builds, doctests
pass.** git-initialised on `main`; **nothing pushed**.

Implemented (wave 1 — the whole wishlist):

- **`theme::apply(ctx)`** — fonts + type scale + spacing. Rides egui's
  default dark `Visuals` and overrides nothing else, exactly like the apps
  → **drop-in, no restyle**.
- **`fonts`** — bundles AdwaitaSans once; optional `SHORTCUT_FAMILY` with
  the omarchy SUPER logo (`OMARCHY_LOGO`, `omarchy_available()`).
- **`palette`** — named colors **reconciled to the apps' real literals**
  (`TEXT`, `TEXT_MUTED`, `CARD`, `CODE_BG=gray(48)`, `ACCENT`, `ERROR`,
  `WARN`, `INFO`, `CAPTURE`, …) so adoption is pixel-faithful.
- **`metrics`** — spacing / sizing / radius constants.
- **`widgets`** — `sidebar`/`content`/`app_shell`, `nav_item`/`nav_list`,
  `primary_button`/`secondary_button`, `search_field`, `dropdown`,
  `app_picker` (searchable "select a running app"), **`caption`** (the
  faithful glyph-metric inline-code pill painter, ported from vernier),
  `code`/`code_span`, `field_label`, `section_header`, `setting`,
  `padded_text_edit`.
- **`examples/gallery.rs`** — `cargo run --example gallery` shows the
  theme + every widget (incl. the omarchy chip when the font is present).

> ⚠️ Verified by compiler, not by eye. I can't see the rendered window
> from here — please run the gallery and eyeball pixel parity before broad
> migration. The faithful `caption` port and reconciled palette are
> designed for parity, but the line-height was unified to vernier's 22
> (hyprcorrect used 20); confirm that reads fine.

## 4. Adoption proof (hyprcorrect)

On branch **`hyprcorrect/try/adopt-kanso`** (not pushed): added `kanso` as
a path dep to `hyprcorrect-ui` and delegated `sidebar_item` →
`kanso::widgets::nav_item` (a byte-for-byte port → identical rendering).
**`cargo build -p hyprcorrect-ui` succeeds (26 s).** This proves the
"adopt behind your existing helper API" path: swap a helper's *body* for a
kanso call, leave call sites untouched, ship the dedup incrementally.

---

## 5. Recommended architecture

```
~/Projects/
  kanso/                 ← egui design system (DONE, this report)
    crates/kanso         theme · fonts · palette · metrics · widgets
  <neutral-logic>/       ← NEW, separate: toolkit-agnostic, serves all 4
    crates/chordfmt      canonical CTRL/SHIFT/ALT/SUPER format + parse
    crates/desktop-apps  .desktop / running-app enumeration → {id,name,icon}
    crates/xdg-util      paths · single-instance · hyprctl-ipc · tray
```

- **`kanso` stays egui-only.** No GTK, no cross-toolkit shims, no non-UI
  logic. egui's API is its only runtime dep (re-exported so consumers pin
  the same version).
- The **neutral-logic** crates are plain Rust (no UI dep) so mousehop and
  tensaku can use them too. These are a *separate parallel effort* — I did
  not build them tonight (the report-vs-build line had to fall somewhere).
- A GTK design-system crate is **not** justified yet (only 2 of 4 apps are
  GTK and they use divergent architectures — relm4 vs plain adw). Defer
  until a genuine 3rd GTK consumer (rule-of-three).

## 6. Roadmap

**Wave 1 (done):** kanso theme/fonts/palette/metrics + the widget set +
gallery + hyprcorrect adoption proof.

**Wave 2 (after a 2nd real adoption shapes the API):**
- chord-capture **chip** widget (needs the omarchy font wiring, which is
  in; unify vernier's per-segment vs hyprcorrect's centered painting).
- `SettingsWindow` shell: save/revert + dirty-state footer (let vernier's
  adoption shape the dirty-tracking API rather than guessing now).
- `banner` (severity callout) / `card` / `status_dot` / `about_pane`.

**Parallel track:** the three neutral-logic crates (§5).

## 7. Migration plan & the one hard constraint

1. Keep apps on a **path dep** to kanso while iterating.
2. Reconcile the cosmetic divergences as deliberate token decisions:
   `field_label` 15→14, caption line-height 20→22, and decide whether
   kanso's `padded_text_edit` should adopt hyprcorrect's always-on border
   (recommend yes — it reads better).
3. Delegate helpers behind existing APIs (as the hyprcorrect POC does),
   one widget at a time; delete the per-app `AdwaitaSans-Regular.ttf`.
4. Prove against hyprcorrect, then vernier, then make it the default for
   the next egui app.
5. **Release ordering (hard):** the consumer `*-ui` crates publish to
   crates.io, so **kanso must be published to crates.io *before* any
   consumer is tagged**, or you hit the exact release-pr/publish race we
   fixed across the cohort earlier today. Until kanso 0.1.0 is on the
   registry, consumers stay on the path dep.

## 8. Non-goals (explicit)

- No new repo beyond kanso + the neutral-logic repo; no restart.
- No GTK/cross-toolkit widgets in kanso.
- No light theme (every app is dark-only and rides egui's default dark).
- No single-app domain widgets in kanso (rule-of-three).
- Don't let `FontOptions`/theme balloon into per-app config soup — cap to
  the few real divergences; pick one canonical value for cosmetic nits.

## 9. Morning checklist

- [ ] `cd ~/Projects/kanso && cargo run --example gallery` — eyeball it.
- [ ] Review `hyprcorrect` branch `try/adopt-kanso` (one-commit diff).
- [ ] Decide the canonical values for the 3 divergences (§7.2).
- [ ] Bless the name `kanso` (or rename — trivial, nothing pushed).
- [ ] Green-light: finish hyprcorrect delegation (caption/field_label/
      padded_text_edit), then vernier, then scaffold the neutral-logic repo.
