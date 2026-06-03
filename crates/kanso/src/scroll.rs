//! Kinetic ("momentum") scrolling for the mouse wheel / trackpad.
//!
//! egui only synthesizes momentum for content **drag** (press-drag-release
//! coasts; see `ScrollArea`'s `vel`). The mouse wheel and trackpad
//! two-finger scroll get no coast — egui applies the OS deltas and stops the
//! instant you lift. macOS (and GTK, so the cohort's GTK apps) synthesize
//! that coast in the toolkit; egui doesn't, so its windows feel "stickier".
//!
//! This recreates it without touching egui's private drag state: each frame
//! it measures the real scroll **velocity**, and once the user lifts it
//! keeps scrolling by injecting a friction-decayed delta into
//! [`egui::InputState::smooth_scroll_delta`] — the exact field `ScrollArea`
//! reads — so momentum applies to whichever scroll area is hovered, with no
//! per-area wrapping.
//!
//! Call [`scroll_momentum`] once at the **very top** of your
//! `eframe::App::update`, before any panels draw:
//!
//! ```no_run
//! # struct App;
//! # impl App {
//! fn update(&mut self, ctx: &kanso::egui::Context) {
//!     kanso::scroll::scroll_momentum(ctx);
//!     // … draw panels …
//! }
//! # }
//! ```

use std::hash::Hash;

use egui::{Context, Id, Vec2};

/// Tuning for [`scroll_momentum_with`].
#[derive(Clone, Copy, Debug)]
pub struct MomentumConfig {
    /// Exponential ease-out time constant, in seconds: the coast loses ~63%
    /// of its speed every `decay_time`. This sets the **ease feel** —
    /// deceleration is proportional to the current speed (like macOS/iOS), so
    /// it tapers gently with a soft tail. Larger = floatier.
    pub decay_time: f32,
    /// A constant deceleration (px/s²) applied *on top of* the exponential
    /// ease. This caps how far a flick carries and firms up the stop —
    /// **raise it to make the glide travel less** / stop sooner, lower it to
    /// let it carry. With the exponential alone, distance is `velocity ×
    /// decay_time`, which reaches the end of a long panel easily; this trims
    /// that tail without making the high-speed ease feel abrupt.
    pub friction: f32,
    /// Speed (px/s) below which the coast snaps to a stop.
    pub stop_speed: f32,
    /// EMA weight (0..=1) for the velocity estimate while actively
    /// scrolling — higher tracks the *current* flick speed more tightly, so
    /// slowing down before you lift yields less momentum (like macOS).
    pub velocity_smoothing: f32,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            // Exponential ease for the gentle taper, plus a modest constant
            // friction so a normal flick doesn't carry all the way to the end
            // of a long panel. Measured trackpad release velocities run
            // ~300–1800 px/s.
            decay_time: 1.0,
            friction: 0.0,
            stop_speed: 20.0,
            // Track the current flick speed tightly so a fast lift keeps its
            // momentum (low values lag and under-seed the coast).
            velocity_smoothing: 0.8,
        }
    }
}

/// Add kinetic momentum to wheel/trackpad scrolling with the default feel.
/// See the [module docs](self) for placement. Equivalent to
/// [`scroll_momentum_with`] with [`MomentumConfig::default`].
pub fn scroll_momentum(ctx: &Context) {
    scroll_momentum_with(ctx, MomentumConfig::default());
}

/// [`scroll_momentum`] with explicit tuning.
pub fn scroll_momentum_with(ctx: &Context, cfg: MomentumConfig) {
    let id = Id::new("kanso::scroll_momentum");
    let dt = ctx.input(|i| i.stable_dt).max(1.0 / 240.0);
    // Real scroll this frame, and whether the user is zoom-scrolling
    // (ctrl/cmd + scroll) — in which case we stay out of the way.
    let (raw, zooming) =
        ctx.input(|i| (i.raw_scroll_delta, i.modifiers.command || i.modifiers.ctrl));

    let Momentum { mut vel, mut idle } = ctx.data_mut(|d| d.get_temp(id).unwrap_or_default());

    if zooming {
        vel = Vec2::ZERO;
        idle = 0.0;
    } else if raw.length() > ACTIVE_THRESHOLD {
        // Actively scrolling: egui applies the real delta this frame; we just
        // track velocity (px/s) so we know how fast to coast on release.
        let instant = raw / dt;
        let t = cfg.velocity_smoothing.clamp(0.0, 1.0);
        vel += (instant - vel) * t;
        idle = 0.0;
    } else if vel.length() > cfg.stop_speed {
        idle += dt;
        if idle < RELEASE_DELAY {
            // A brief gap *between* scroll events (they don't arrive every
            // frame) — not a lift. Keep gliding at the tracked velocity
            // without bleeding any speed, so the flick survives intact until
            // the real release.
            ctx.input_mut(|i| i.smooth_scroll_delta += vel * dt);
            ctx.request_repaint();
        } else {
            // Released: exponential ease-out (deceleration ∝ speed) for the
            // gentle taper, plus a constant friction term that trims the tail
            // so the flick doesn't carry too far. Then inject the coast into
            // the very field ScrollArea reads so the hovered area scrolls.
            let speed = vel.length();
            let eased = speed * (-dt / cfg.decay_time.max(1e-3)).exp();
            let new_speed = (eased - cfg.friction * dt).max(0.0);
            vel *= new_speed / speed;
            ctx.input_mut(|i| i.smooth_scroll_delta += vel * dt);
            ctx.request_repaint();
        }
    } else {
        vel = Vec2::ZERO;
    }

    ctx.data_mut(|d| d.insert_temp(id, Momentum { vel, idle }));
}

/// Per-frame coast state, persisted in egui memory between frames.
#[derive(Clone, Copy, Default)]
struct Momentum {
    /// Tracked scroll velocity in px/s.
    vel: Vec2,
    /// Seconds since the last real scroll event (gap vs. genuine release).
    idle: f32,
}

/// Below this per-frame scroll magnitude (px) we treat the wheel as idle.
/// Small enough to catch a real lift, large enough to ignore sub-pixel noise.
const ACTIVE_THRESHOLD: f32 = 0.1;
/// Scroll events can skip frames mid-gesture (especially at high refresh).
/// Hold the velocity through gaps shorter than this before treating the lack
/// of events as a genuine lift and starting the friction coast.
const RELEASE_DELAY: f32 = 0.06;

// ---------------------------------------------------------------------------
// Owned-offset scroll view: kinetic fling + rubber-band over-scroll
// ---------------------------------------------------------------------------
//
// libinput deliberately does NOT synthesize kinetic scrolling — it offloads
// that to the toolkit, so unlike macOS nothing hands us momentum, and without
// it a flick never reaches the edge after the fingers lift, so it can never
// bounce. We therefore own the scroll offset ourselves and run one controller
// for the whole motion: drag -> fling (synthesized from a velocity tracker on
// the raw deltas) -> and, when the fling hits a boundary, the leftover velocity
// is absorbed into a rubber-band that releases through WebKit's velocity-seeded
// exponential snap-back. Modelled on WebKit/Chromium's
// `ScrollElasticityController` (the Safari/Finder code path).

/// Snap-back stiffness (native WebKit constant).
const RB_STIFFNESS: f32 = 20.0;
/// Snap velocity-seed amplitude (native) — how far the release velocity carries
/// the stretch out before it decays.
const RB_AMPLITUDE: f32 = 0.31;
/// Snap period (native); larger = looser / slower bounce.
const RB_PERIOD: f32 = 1.6;
/// Pull-curve diminishing-returns constant (native); lower resists more / less
/// travel. Sets the *initial* pull slope (independent of the asymptote below), so
/// the first bit of overscroll feels the same regardless of `MAX_PULL`.
const RB_STRETCH_C: f32 = 0.55;
/// Asymptote (px) the manual over-scroll *pull* approaches but never reaches.
/// Native WebKit uses the viewport height, which lets a hold-drag stretch a large
/// fraction of the screen; macOS on a trackpad caps far shorter, so we asymptote
/// at a fixed ~screenful-independent distance. Only caps the manual pull — the
/// flick bounce is velocity-driven (`rb_elastic`) and can still exceed this.
const MAX_PULL: f32 = 60.0;
/// Fling friction: fraction of velocity kept per millisecond. `0.999` ≈ a 1 s
/// exponential time-constant — the floaty coast the cohort tuned to (the doc's
/// "normal" 0.998 is noticeably snappier; 0.99 = "fast").
const FLING_FRICTION: f32 = 0.999;
/// Low-pass time constant (s) for the *rendered* offset/stretch — stands in for
/// the egui scroll smoothing we bypassed, so direct scrolling reads smoothly
/// instead of stepping per raw delta. Small enough not to feel laggy.
const SCROLL_RENDER_TAU: f32 = 0.03;
/// EMA weight for the per-frame velocity track: higher follows the *current*
/// flick speed more tightly, so the value at lift reflects the peak of an
/// accelerating throw (matches the tuned momentum module's `velocity_smoothing`).
const VEL_SMOOTHING: f32 = 0.8;
/// Time constant (s) for spreading a coarse raw scroll burst across frames so a
/// slow drag glides instead of jumping. Small enough that the lag is negligible
/// at flick speed; large enough to de-chunk a slow scroll.
const DRAG_SMOOTH_TAU: f32 = 0.025;
/// Scale applied to the launch velocity when starting a *fling* (not the
/// rubber-band, which keeps the raw seed). The tuned momentum module's coast was
/// re-smoothed by egui's `ScrollArea`, damping it; driving the offset directly
/// has no such damping, so a hard flick over-carries without this trim. Scaling
/// the seed trims coast distance (`= v0 * tau`) linearly while leaving the decay
/// curve — and thus the glide/ease feel — unchanged.
const FLING_SCALE: f32 = 0.385;
/// Minimum lift speed (px/s) to start a fling. Set *above* deliberate slow-scroll
/// release speeds so controlled scrolling in mid-page just stops where you lift,
/// and only a genuine flick coasts.
const FLING_MIN: f32 = 500.0;
/// A fling settles (stops) below this speed (px/s).
const FLING_STOP: f32 = 20.0;
/// No scroll event for this long (s) is taken as "fingers lifted" — egui drops
/// the trackpad gesture phase, so we infer the lift from a gap in events.
const LIFT_GAP: f32 = 0.06;
/// Width (px) reserved at the right edge for the scrollbar.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// Pull curve: accumulated raw overscroll -> damped, asymptotic visible stretch
/// (approaches `MAX_PULL`, clamped to the viewport, but never reaches it).
fn rb_pull(raw: f32, dim: f32) -> f32 {
    if dim <= 0.0 {
        return 0.0;
    }
    let cap = MAX_PULL.min(dim);
    raw.signum() * (1.0 - 1.0 / (raw.abs() * RB_STRETCH_C / cap + 1.0)) * cap
}

/// Inverse of [`rb_pull`] — the raw overscroll that yields a given visible
/// stretch (keeps the unclamped position consistent during a bounce). A
/// velocity-driven bounce can push the stretch past `cap`; the `min(0.999)` keeps
/// the result finite there (the excess is bookkeeping, not rendered).
fn rb_pull_inverse(stretch: f32, dim: f32) -> f32 {
    if dim <= 0.0 {
        return 0.0;
    }
    let cap = MAX_PULL.min(dim);
    let y = (stretch.abs() / cap).min(0.999);
    cap * y / (RB_STRETCH_C * (1.0 - y))
}

/// Snap-back position over time, seeded with the stretch `x0` and velocity `v0`
/// when the snap began. The same function serves a gentle release and a hard
/// flick — only `v0` differs.
fn rb_elastic(x0: f32, v0: f32, t: f32) -> f32 {
    let damp = (-t * RB_STIFFNESS / RB_PERIOD).exp();
    (x0 + (-v0 * t * RB_AMPLITUDE)) * damp
}

#[derive(Clone, Copy, Default, PartialEq)]
enum ScrollState {
    #[default]
    Idle,
    Dragging,
    Flinging,
    Bouncing,
}

/// Owned scroll state, persisted in egui memory per scroll view.
#[derive(Clone, Default)]
struct ScrollPhysics {
    state: ScrollState,
    /// Unclamped scroll position: the clamped part is the offset, any excess is
    /// the overscroll.
    pos: f32,
    /// Clamped scroll offset (`0..=max_offset`).
    offset: f32,
    /// Visible overscroll (px): `> 0` past the top edge, `< 0` past the bottom.
    stretch: f32,
    /// Fling velocity (px/s).
    velocity: f32,
    /// EMA of the per-frame velocity (`raw/dt`, px/s) while dragging — biased to
    /// the last frames so a flick seeds its fling from the speed *at lift*, not
    /// the gesture average. Seeds both the fling and the rubber-band bounce.
    vel_ema: f32,
    /// Unreleased drag delta: libinput hands us coarse raw bursts, so the drag
    /// position is fed from here a fraction per frame (de-chunked) rather than
    /// the whole burst at once. Velocity/fling still use the full raw.
    pending: f32,
    last_event: f64,
    snap_x0: f32,
    snap_v0: f32,
    snap_start: f64,
    /// Last frame's content height (drives `max_offset`).
    content_h: f32,
    /// Low-pass-smoothed stretch actually rendered (the offset is direct).
    render_stretch: f32,
}

impl ScrollPhysics {
    fn max_offset(&self, dim: f32) -> f32 {
        (self.content_h - dim).max(0.0)
    }

    /// Apply a scroll delta to the unclamped position, deriving the clamped
    /// offset and the rubber-band stretch. Reversing direction tracks the
    /// stretch naturally back toward zero — no hard reset.
    fn apply(&mut self, delta: f32, dim: f32) {
        let max = self.max_offset(dim);
        self.pos -= delta;
        if self.pos < 0.0 {
            self.offset = 0.0;
            self.stretch = rb_pull(-self.pos, dim);
        } else if self.pos > max {
            self.offset = max;
            self.stretch = -rb_pull(self.pos - max, dim);
        } else {
            self.offset = self.pos;
            self.stretch = 0.0;
        }
    }

    /// Track the per-frame velocity (`raw/dt`) as an EMA, biased to the most
    /// recent frames. A fresh gesture seeds from the first frame's speed (no
    /// stale carry-over); a continuing one converges toward the live speed in a
    /// few frames, so the value at lift reflects the *peak* of an accelerating
    /// flick rather than its average — the model the tuned momentum module uses.
    fn track_velocity(&mut self, instant: f32) {
        if self.state == ScrollState::Dragging {
            self.vel_ema += (instant - self.vel_ema) * VEL_SMOOTHING;
        } else {
            self.vel_ema = instant;
        }
    }

    fn begin_bounce(&mut self, now: f64, v: f32) {
        self.state = ScrollState::Bouncing;
        self.pending = 0.0;
        self.snap_x0 = self.stretch;
        // The velocity carries the stretch *out* before it decays; the sign
        // flips per egui's axis convention so it matches the stretch direction.
        self.snap_v0 = -v;
        self.snap_start = now;
    }
}

/// A vertical scroll view with macOS-style kinetic fling + rubber-band
/// over-scroll, owning the scroll offset itself. egui's `ScrollArea` can neither
/// over-scroll nor fling on Linux (libinput hands the toolkit no momentum), so
/// this drives the whole motion from the raw deltas and renders the content
/// translated + clipped.
///
/// Drop-in for a `ScrollArea::vertical()` that fills the available space; pass
/// an `id_salt` unique within the parent `Ui`. Returns the body's value.
///
/// ```no_run
/// # fn demo(ui: &mut kanso::egui::Ui) {
/// kanso::scroll::scroll_view(ui, "content", |ui| {
///     for i in 0..200 { ui.label(format!("row {i}")); }
/// });
/// # }
/// ```
pub fn scroll_view<R>(
    ui: &mut egui::Ui,
    id_salt: impl Hash,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let id = ui.make_persistent_id((id_salt, "kanso_scroll_view"));
    let mut sp: ScrollPhysics = ui.data_mut(|d| d.get_temp(id).unwrap_or_default());
    let now = ui.input(|i| i.time);
    let dt = ui.input(|i| i.stable_dt).min(0.1);

    let viewport = ui.available_rect_before_wrap();
    let dim = viewport.height();

    // Input — driven by the RAW delta (egui's smoothing would fight the
    // physics), and only while the pointer is over the viewport.
    let hovering = ui.rect_contains_pointer(viewport);
    let raw = if hovering {
        ui.input(|i| i.raw_scroll_delta.y)
    } else {
        0.0
    };
    if raw != 0.0 {
        if matches!(sp.state, ScrollState::Flinging | ScrollState::Bouncing) {
            // Re-touching the trackpad mid-coast halts it dead (macOS-style):
            // flipping to Dragging stops the fling, and dropping this contact
            // frame's velocity means a quick tap-to-stop won't re-fling — only a
            // *sustained* new drag rebuilds enough speed (EMA from zero) to throw
            // again. (A pure finger-rest with no movement sends no event, so the
            // earliest we can stop is this first flicker of contact motion.)
            sp.velocity = 0.0;
            sp.vel_ema = 0.0;
        } else {
            // Track velocity *before* flipping to Dragging so a fresh gesture
            // seeds from this frame's speed for a responsive flick.
            sp.track_velocity(raw / dt);
        }
        sp.pending += raw;
        sp.last_event = now;
        sp.state = ScrollState::Dragging;
    }

    // Drain the coarse raw burst into the position a fraction per frame so a slow
    // drag glides. Not during a bounce (the snap owns the position); any
    // remainder at lift simply drains into the fling, so there's no flush hitch.
    if sp.state != ScrollState::Bouncing && sp.pending != 0.0 {
        let alpha = 1.0 - (-dt / DRAG_SMOOTH_TAU).exp();
        let release = if sp.pending.abs() < 0.5 {
            sp.pending
        } else {
            sp.pending * alpha
        };
        sp.pending -= release;
        sp.apply(release, dim);
        if sp.pending.abs() > 0.01 {
            ui.ctx().request_repaint();
        }
    }

    match sp.state {
        ScrollState::Dragging => {
            // No lift event reaches egui, so infer it from a gap in scroll
            // events, then bounce (released stretched), fling (thrown), or stop.
            if (now - sp.last_event) as f32 > LIFT_GAP {
                let v = sp.vel_ema;
                if sp.stretch != 0.0 {
                    sp.begin_bounce(now, v);
                } else if v.abs() > FLING_MIN {
                    sp.velocity = v * FLING_SCALE;
                    sp.state = ScrollState::Flinging;
                } else {
                    sp.state = ScrollState::Idle;
                }
            }
        }
        ScrollState::Flinging => {
            sp.velocity *= FLING_FRICTION.powf(dt * 1000.0);
            sp.apply(sp.velocity * dt, dim);
            if sp.stretch != 0.0 {
                // Momentum hit the wall — absorb the leftover velocity into the
                // bounce (a hard flick bounces harder).
                sp.begin_bounce(now, sp.velocity);
            } else if sp.velocity.abs() < FLING_STOP {
                sp.state = ScrollState::Idle;
            }
        }
        ScrollState::Bouncing => {
            let t = (now - sp.snap_start) as f32;
            sp.stretch = rb_elastic(sp.snap_x0, sp.snap_v0, t);
            if sp.stretch.abs() < 1.0 {
                sp.stretch = 0.0;
                sp.pos = sp.offset;
                sp.state = ScrollState::Idle;
            } else {
                // Keep the unclamped position consistent with the decaying
                // stretch so a new drag resumes smoothly.
                let excess = rb_pull_inverse(sp.stretch.abs(), dim);
                let max = sp.max_offset(dim);
                sp.pos = if sp.stretch > 0.0 {
                    -excess
                } else {
                    max + excess
                };
            }
        }
        ScrollState::Idle => {
            // Content may have shrunk since last frame (e.g. the host swapped in
            // a shorter view), leaving the resting offset past the new bottom —
            // which would render the content off-screen until the next scroll.
            // Clamp back into range when settled so the view can never go blank.
            let max = sp.max_offset(dim);
            if sp.offset > max {
                sp.offset = max;
                sp.pos = max;
            }
        }
    }

    if sp.state != ScrollState::Idle {
        ui.ctx().request_repaint();
    }

    // Drive the offset *directly* (responsive — smoothing it lags the finger
    // and reads as "slow"); the fling already moves it smoothly frame-to-frame.
    // Only low-pass the over-scroll stretch, where sparse-event stepping shows.
    let alpha = 1.0 - (-dt / SCROLL_RENDER_TAU).exp();
    sp.render_stretch += (sp.stretch - sp.render_stretch) * alpha;
    if (sp.render_stretch - sp.stretch).abs() > 0.05 {
        ui.ctx().request_repaint();
    }
    let render_offset = sp.offset;
    let render_stretch = sp.render_stretch;

    // Reserve the viewport in the parent's layout.
    ui.allocate_rect(viewport, egui::Sense::hover());

    // Render the content translated by `(-offset + stretch)`, clipped to the
    // viewport: a positive stretch detaches the content from the top edge (gap
    // above), a negative one from the bottom.
    let content_w = (viewport.width() - SCROLLBAR_WIDTH).max(0.0);
    let content_rect = egui::Rect::from_min_size(
        egui::pos2(
            viewport.left(),
            viewport.top() - render_offset + render_stretch,
        ),
        egui::vec2(content_w, f32::INFINITY),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(viewport.intersect(ui.clip_rect()));
    let r = add(&mut child);
    sp.content_h = child.min_rect().height();

    // Thin scrollbar — a position indicator (no drag handle yet).
    let max = sp.max_offset(dim);
    if max > 0.5 && sp.content_h > 0.0 {
        let bar_h = (dim / sp.content_h * dim).clamp(24.0, dim);
        let bar_y = viewport.top() + (render_offset.clamp(0.0, max) / max) * (dim - bar_h);
        let bar = egui::Rect::from_min_size(
            egui::pos2(viewport.right() - SCROLLBAR_WIDTH + 1.0, bar_y),
            egui::vec2(SCROLLBAR_WIDTH - 3.0, bar_h),
        );
        let color = if hovering {
            crate::palette::BORDER
        } else {
            crate::palette::CHIP
        };
        ui.painter()
            .rect_filled(bar, egui::CornerRadius::same(3), color);
    }

    ui.data_mut(|d| d.insert_temp(id, sp));
    r
}

/// Reset a [`scroll_view`] (identified by the same `id_salt`, on the same `ui`)
/// back to the top, clearing its kinetic and over-scroll state. Call right
/// before `scroll_view` when the content is swapped wholesale — e.g. changing
/// which section a single scroll area renders — so the new content opens at the
/// top instead of inheriting the previous offset.
pub fn scroll_view_reset(ui: &egui::Ui, id_salt: impl Hash) {
    let id = ui.make_persistent_id((id_salt, "kanso_scroll_view"));
    ui.data_mut(|d| d.insert_temp(id, ScrollPhysics::default()));
}
