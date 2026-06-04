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

/// `true` while fingers rest on the touchpad — the macOS "touch to stop a
/// momentum scroll" signal. Read from egui memory, where our forked egui-winit
/// publishes the Wayland hold gesture (matches `egui_winit::HOLD_GESTURE_ID`).
/// Without the fork the slot is never written, so this is simply always `false`
/// — a graceful no-op, no dependency on the fork.
fn hold_gesture_active(ctx: &Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(Id::new("egui_hold_gesture_active")))
        .unwrap_or(false)
}

/// `true` when this frame's scroll input is coming from a **mouse wheel**
/// (discrete `Line`/`Page` steps) rather than a **trackpad** (pixel-precise
/// `Point` deltas). winit maps a notched *or* free-spinning ("kinetic") mouse
/// wheel to `MouseScrollDelta::LineDelta` and a two-finger trackpad scroll to
/// `PixelDelta`, which egui surfaces as the [`egui::MouseWheelUnit`].
///
/// The kinetic momentum + rubber-band over-scroll in this module is a trackpad
/// affordance — native toolkits (macOS, GTK) only ever coast/over-scroll the
/// trackpad, never the wheel. So both the momentum coast and the owned-offset
/// fling are gated on this returning `false`: a wheel scrolls plainly, and a
/// free-spinning wheel can never pin an over-scroll while it's still turning.
fn scroll_is_wheel(ctx: &Context) -> bool {
    ctx.input(|i| {
        i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line | egui::MouseWheelUnit::Page,
                    ..
                }
            )
        })
    })
}

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

    // A touchpad hold (fingers resting) stops the coast dead — macOS "touch to
    // stop". Treated like zooming: bleed all velocity and stay out of the way.
    let held = hold_gesture_active(ctx);

    // A mouse wheel never coasts (kinetic is trackpad-only; see
    // `scroll_is_wheel`). egui still applies the wheel's raw delta this frame
    // via its own smoothing, so wheel scrolling keeps working — it just gets no
    // momentum tail bolted on.
    let wheel = scroll_is_wheel(ctx);

    let Momentum { mut vel, mut idle } = ctx.data_mut(|d| d.get_temp(id).unwrap_or_default());

    if zooming || held || wheel {
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
/// Snap velocity-seed amplitude — sets both how *fast* the bounce shoots into the
/// overscroll (initial velocity ∝ `v·A`) and how far (peak ≈ `v·A·(P/K)/e`).
/// Raised well above native (0.31) so a flick at the edge snaps out fast and far.
const RB_AMPLITUDE: f32 = 3.0;
/// Snap period; the velocity-bounce peaks at `t* = RB_PERIOD/RB_STIFFNESS` and
/// settles in ~`4·t*`. Set with amplitude on a trackpad against macOS so the edge
/// bounce shoots out fast/far and springs back at the right pace.
const RB_PERIOD: f32 = 1.5;
/// Pull-curve diminishing-returns constant (native); lower resists more / less
/// travel. Sets the *initial* pull slope (independent of the asymptote below), so
/// the first bit of overscroll feels the same regardless of `MAX_PULL`.
const RB_STRETCH_C: f32 = 0.55;
/// Asymptote (px) the manual over-scroll *pull* approaches but never reaches.
/// Native WebKit uses the viewport height, which lets a hold-drag stretch a large
/// fraction of the screen; macOS on a trackpad caps far shorter, so we asymptote
/// at a fixed ~screenful-independent distance. Only caps the manual pull — the
/// flick bounce is velocity-driven (`rb_elastic`) and can still exceed this.
const MAX_PULL: f32 = 50.0;
/// Base fling friction: fraction of velocity kept per millisecond. `0.99601` ≈ a
/// 0.25 s exponential time-constant — the *base* coast; hard flicks stretch this
/// τ via `FLING_GAIN` (super-linear coast). Set on a trackpad against macOS.
const FLING_FRICTION: f32 = 0.99601;
/// Low-pass time constant (s) for the *rendered* offset/stretch — stands in for
/// the egui scroll smoothing we bypassed, so direct scrolling reads smoothly
/// instead of stepping per raw delta. Small enough not to feel laggy.
const SCROLL_RENDER_TAU: f32 = 0.03;
/// Time constant (s) for the velocity-track EMA — the averaging window over the
/// per-frame `raw/dt` that seeds the fling. Frame-rate **independent** (applied
/// as `1 - exp(-dt/τ)`), so the launch velocity, and thus coast/overscroll
/// distance, is the same at any FPS — a debug build and an optimized release
/// build feel identical. Larger = smoother/slower seed; smaller = peakier/faster.
/// Tune against a RELEASE build (that's what ships), e.g.
/// `cargo run --release --example gallery`.
const VEL_TAU: f32 = 0.002;
/// Time constant (s) for spreading a coarse raw scroll burst across frames so a
/// slow drag glides instead of jumping. Small enough that the lag is negligible
/// at flick speed; large enough to de-chunk a slow scroll.
const DRAG_SMOOTH_TAU: f32 = 0.009;
/// Base coast-τ scale at the fling knee. The fling always *launches* at the
/// finger speed (continuous hand-off); this scales the decay τ, not the launch,
/// so a gentle flick (`scale = 1`) decays at the base `FLING_FRICTION` τ.
const FLING_SCALE: f32 = 1.0;
/// Per-(px/s)-over-the-knee added to the τ scale: `scale = FLING_SCALE +
/// FLING_GAIN * (|v| - FLING_KNEE).max(0)`, and the fling decays at
/// `FLING_FRICTION^(1/scale)` — a longer τ for hard flicks, so the coast is
/// *super-linear* in flick speed (macOS-style) without inflating the launch
/// (which keeps the edge bounce honest). The top-end dial.
const FLING_GAIN: f32 = 0.002;
/// Flick speed (px/s) below which the coast stays at the base τ; above it the
/// super-linear τ stretch kicks in, so gentle scrolling is unaffected.
const FLING_KNEE: f32 = 50.0;
/// Minimum lift speed (px/s) to start a fling. Above deliberate slow-scroll
/// release speeds so a controlled scroll just stops where you lift, but low
/// enough that a quick *small* flick still coasts.
const FLING_MIN: f32 = 50.0;
/// A fling settles (stops) below this speed (px/s).
const FLING_STOP: f32 = 20.0;
/// No scroll event for this long (s) is taken as "fingers lifted" — egui drops
/// the trackpad gesture phase, so we infer the lift from a gap in events.
/// Shorter = less on-screen velocity bleeds away before the fling fires (smaller
/// hand-off step); must stay above the inter-event gap to avoid false lifts.
const LIFT_GAP: f32 = 0.045;
/// Once a drag pauses longer than this (s) — past the normal inter-event cadence
/// but before a confirmed lift — the view coasts at the tracked velocity instead
/// of letting the drained position freeze. This makes the drag→fling hand-off
/// seamless (no velocity step / hitch). Must sit above the inter-event gap so
/// normal mid-drag frames keep draining, and below `LIFT_GAP`.
const COAST_GAP: f32 = 0.018;
/// Width (px) reserved at the right edge for the scrollbar.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// Live-tunable scroll-feel knobs. Every field defaults to the matching module
/// constant, so behaviour is unchanged until an app overrides them via
/// [`set_scroll_tuning`]. Stored in egui memory and refreshed into each view per
/// frame, so edits (e.g. the gallery's tuning sliders) take effect immediately.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollTuning {
    /// Manual over-scroll asymptote (px): how far a slow pull can stretch.
    pub max_pull: f32,
    /// Pull-curve diminishing-returns constant; lower resists more.
    pub rb_stretch_c: f32,
    /// Rubber-band stiffness; the velocity bounce peaks at `rb_period/rb_stiffness`.
    pub rb_stiffness: f32,
    /// Rubber-band period; raises both the bounce travel and its time-to-peak.
    pub rb_period: f32,
    /// Velocity-bounce amplitude: how fast/far a flick shoots into the overscroll.
    pub rb_amplitude: f32,
    /// Velocity-seed EMA window (s): the lift speed is averaged over ~this long.
    pub vel_tau: f32,
    /// Drag de-chunk time-constant (s).
    pub drag_smooth_tau: f32,
    /// Pause (s) past which a drag coasts at the tracked velocity (hitch-free).
    pub coast_gap: f32,
    /// No-event gap (s) taken as a finger lift.
    pub lift_gap: f32,
    /// Base launch scale at the fling knee (≈ 1:1 with the lift speed).
    pub fling_scale: f32,
    /// Super-linear coast gain per px/s above the knee (longer τ for hard flicks).
    pub fling_gain: f32,
    /// Flick speed (px/s) above which the super-linear coast kicks in.
    pub fling_knee: f32,
    /// Minimum lift speed (px/s) to start a fling.
    pub fling_min: f32,
    /// Base fling decay (velocity kept per ms): the overall glide length.
    pub fling_friction: f32,
    /// A fling stops below this speed (px/s).
    pub fling_stop: f32,
}

impl Default for ScrollTuning {
    fn default() -> Self {
        Self {
            max_pull: MAX_PULL,
            rb_stretch_c: RB_STRETCH_C,
            rb_stiffness: RB_STIFFNESS,
            rb_period: RB_PERIOD,
            rb_amplitude: RB_AMPLITUDE,
            vel_tau: VEL_TAU,
            drag_smooth_tau: DRAG_SMOOTH_TAU,
            coast_gap: COAST_GAP,
            lift_gap: LIFT_GAP,
            fling_scale: FLING_SCALE,
            fling_gain: FLING_GAIN,
            fling_knee: FLING_KNEE,
            fling_min: FLING_MIN,
            fling_friction: FLING_FRICTION,
            fling_stop: FLING_STOP,
        }
    }
}

fn scroll_tuning_id() -> Id {
    Id::new("kanso_scroll_tuning")
}

/// Read the scroll-feel tuning active for this context (defaults if unset).
pub fn scroll_tuning(ctx: &Context) -> ScrollTuning {
    ctx.data_mut(|d| d.get_temp(scroll_tuning_id()).unwrap_or_default())
}

/// Override the scroll-feel tuning for every [`scroll_view`] in this context.
/// Intended for live tuning (sliders); apps normally rely on the defaults.
pub fn set_scroll_tuning(ctx: &Context, tuning: ScrollTuning) {
    ctx.data_mut(|d| d.insert_temp(scroll_tuning_id(), tuning));
}

/// Pull curve: accumulated raw overscroll -> damped, asymptotic visible stretch
/// (approaches `MAX_PULL`, clamped to the viewport, but never reaches it).
fn rb_pull(raw: f32, dim: f32, tun: &ScrollTuning) -> f32 {
    if dim <= 0.0 {
        return 0.0;
    }
    let cap = tun.max_pull.min(dim);
    raw.signum() * (1.0 - 1.0 / (raw.abs() * tun.rb_stretch_c / cap + 1.0)) * cap
}

/// Inverse of [`rb_pull`] — the raw overscroll that yields a given visible
/// stretch (keeps the unclamped position consistent during a bounce). A
/// velocity-driven bounce can push the stretch past `cap`; the `min(0.999)` keeps
/// the result finite there (the excess is bookkeeping, not rendered).
fn rb_pull_inverse(stretch: f32, dim: f32, tun: &ScrollTuning) -> f32 {
    if dim <= 0.0 {
        return 0.0;
    }
    let cap = tun.max_pull.min(dim);
    let y = (stretch.abs() / cap).min(0.999);
    cap * y / (tun.rb_stretch_c * (1.0 - y))
}

/// Snap-back position over time, seeded with the stretch `x0` and velocity `v0`
/// when the snap began. The same function serves a gentle release and a hard
/// flick — only `v0` differs.
fn rb_elastic(x0: f32, v0: f32, t: f32, tun: &ScrollTuning) -> f32 {
    let damp = (-t * tun.rb_stiffness / tun.rb_period).exp();
    (x0 + (-v0 * t * tun.rb_amplitude)) * damp
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
    /// Per-fling decay factor (velocity kept per ms), set at launch. Hard flicks
    /// get a slower decay (longer τ) so they coast farther *without* launching
    /// faster — the super-linearity lives here, not in the launch speed, so the
    /// hand-off stays continuous and the momentum carried into an edge bounce is
    /// the true finger speed (not an inflated launch).
    fling_friction: f32,
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
    /// Feel knobs, refreshed from the shared [`ScrollTuning`] each frame.
    tuning: ScrollTuning,
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
        // Bound the over-scroll excess. The rubber-band stretch saturates near
        // `cap`, but `pos` would otherwise accumulate without limit under a hard
        // push — and then reversing has to claw all of it back before the content
        // moves (it "sticks"). Cap the excess at ~the saturation point.
        let cap = self.tuning.max_pull.min(dim);
        if cap > 0.0 {
            self.pos = self.pos.clamp(-20.0 * cap, max + 20.0 * cap);
        }
        if self.pos < 0.0 {
            self.offset = 0.0;
            self.stretch = rb_pull(-self.pos, dim, &self.tuning);
        } else if self.pos > max {
            self.offset = max;
            self.stretch = -rb_pull(self.pos - max, dim, &self.tuning);
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
    fn track_velocity(&mut self, instant: f32, dt: f32) {
        if self.state == ScrollState::Dragging {
            // dt-aware EMA: averages `instant` over ~vel_tau seconds regardless
            // of frame rate, so the seed velocity is FPS-independent.
            let alpha = 1.0 - (-dt / self.tuning.vel_tau).exp();
            self.vel_ema += (instant - self.vel_ema) * alpha;
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
    // Refresh the feel knobs from the shared tuning each frame (defaults unless an
    // app/the gallery sliders overrode them) so live edits apply immediately.
    sp.tuning = scroll_tuning(ui.ctx());
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

    // A touchpad hold (fingers resting, no movement) stops the coast dead —
    // macOS "touch to stop", surfaced by the forked egui-winit. This catches the
    // zero-movement rest that emits no scroll event; the raw-delta interrupt
    // below handles a re-touch that also moves a little.
    if hold_gesture_active(ui.ctx()) && sp.state == ScrollState::Flinging {
        sp.velocity = 0.0;
        sp.vel_ema = 0.0;
        sp.state = ScrollState::Idle;
    }

    // A mouse wheel bypasses the whole fling/rubber-band controller: it steps
    // the offset directly and clamps hard at the edges, so a free-spinning
    // ("kinetic") wheel never pins an over-scroll while it's still turning. The
    // kinetic physics below is trackpad-only (see `scroll_is_wheel`). `raw` is
    // already in points (egui scales the wheel's line delta by
    // `line_scroll_speed`), so it applies directly.
    let wheel = scroll_is_wheel(ui.ctx());
    if raw != 0.0 && wheel {
        sp.velocity = 0.0;
        sp.vel_ema = 0.0;
        sp.pending = 0.0;
        sp.stretch = 0.0;
        let max = sp.max_offset(dim);
        sp.offset = (sp.offset - raw).clamp(0.0, max);
        sp.pos = sp.offset;
        sp.state = ScrollState::Idle;
        sp.last_event = now;
    } else if raw != 0.0 {
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
            // seeds from this frame's speed for a responsive flick. Seed from the
            // time since the LAST scroll event, NOT the frame dt: when the render
            // rate outruns the event rate (e.g. vsync off → 1000+ fps), the frame
            // dt shrinks far below the ~8 ms between trackpad events, and `raw/dt`
            // would over-read the speed by that ratio. The inter-event interval is
            // the true span this `raw` delta covers, so the seed is correct at any
            // fps. Clamp out the idle gap before a fresh gesture (and div-by-zero).
            let event_dt = ((now - sp.last_event) as f32).clamp(1.0 / 1000.0, 0.05);
            sp.track_velocity(raw / event_dt, event_dt);
        }
        sp.pending += raw;
        sp.last_event = now;
        sp.state = ScrollState::Dragging;
    }

    // Drain the coarse raw burst into the position a fraction per frame so a slow
    // drag glides. ONLY while actively dragging (gap ≤ COAST_GAP): once the
    // gesture pauses we coast instead (below), and once a fling starts it must
    // move purely at `velocity * dt`, with no stale pending draining on top
    // (which adds a brief over-speed at the hand-off). The sub-pixel residual at
    // lift is discarded below rather than folded into the fling.
    if sp.state == ScrollState::Dragging
        && (now - sp.last_event) as f32 <= sp.tuning.coast_gap
        && sp.pending != 0.0
    {
        let alpha = 1.0 - (-dt / sp.tuning.drag_smooth_tau).exp();
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
            let gap = (now - sp.last_event) as f32;
            if gap <= sp.tuning.lift_gap && gap > sp.tuning.coast_gap {
                // Paused past the event cadence but not yet a confirmed lift:
                // coast at the tracked velocity so the position never freezes,
                // making the fling hand-off seamless (no velocity step / hitch).
                sp.apply(sp.vel_ema * dt, dim);
                ui.ctx().request_repaint();
            }
            if gap > sp.tuning.lift_gap {
                let v = sp.vel_ema;
                // Discard the sub-pixel pending residual so the fling starts clean.
                sp.pending = 0.0;
                // Super-linear *coast*: gentle flicks decay at the base τ, hard
                // flicks decay slower (longer τ) so they travel farther — but the
                // launch speed stays == the finger speed (continuous hand-off, and
                // the velocity carried into an edge bounce is the true momentum).
                let scale = sp.tuning.fling_scale
                    + sp.tuning.fling_gain * (v.abs() - sp.tuning.fling_knee).max(0.0);
                // A flick *away* from a stretched edge (e.g. over-scrolled at the
                // bottom, then thrown up) should fling off the edge: the fling's
                // apply() releases the over-scroll gradually and then coasts. Only
                // bounce when there's no real throw, or the throw is *into* the
                // edge (deepening it) — a bounce seeded with an away velocity just
                // springs back to the edge instead of carrying on.
                let away_flick = sp.stretch != 0.0 && sp.stretch * v < 0.0;
                let fling = (sp.stretch == 0.0 || away_flick) && v.abs() > sp.tuning.fling_min;
                if fling {
                    if sp.stretch != 0.0 {
                        // Flinging away from an over-scroll: release it instantly
                        // (snap pos to the edge) so the throw coasts off the edge
                        // right away; the visible stretch springs back smoothly via
                        // render_stretch rather than gating the coast.
                        sp.pos = if sp.stretch < 0.0 {
                            sp.max_offset(dim)
                        } else {
                            0.0
                        };
                        sp.offset = sp.pos;
                    }
                    sp.velocity = v;
                    sp.fling_friction = sp.tuning.fling_friction.powf(1.0 / scale);
                    sp.state = ScrollState::Flinging;
                } else if sp.stretch != 0.0 {
                    sp.begin_bounce(now, v);
                } else {
                    sp.state = ScrollState::Idle;
                }
            }
        }
        ScrollState::Flinging => {
            sp.velocity *= sp.fling_friction.powf(dt * 1000.0);
            sp.apply(sp.velocity * dt, dim);
            // Bounce only when the fling drives *into* a wall (deepening the
            // stretch — a hard flick bounces harder), or when it stalls while
            // still over-scrolled (spring the residual back). A fling that *exits*
            // an over-scroll (stretch and velocity opposing) keeps coasting so it
            // releases the edge instead of re-bouncing off it.
            let into_edge = sp.stretch != 0.0 && sp.stretch * sp.velocity > 0.0;
            let stalled_overscrolled =
                sp.stretch != 0.0 && sp.velocity.abs() < sp.tuning.fling_stop;
            if into_edge || stalled_overscrolled {
                sp.begin_bounce(now, sp.velocity);
            } else if sp.velocity.abs() < sp.tuning.fling_stop {
                sp.state = ScrollState::Idle;
            }
        }
        ScrollState::Bouncing => {
            let t = (now - sp.snap_start) as f32;
            sp.stretch = rb_elastic(sp.snap_x0, sp.snap_v0, t, &sp.tuning);
            // A release whose velocity points *away* from the stretched edge can
            // drive rb_elastic across zero (the `v·t` term overpowering `x0`).
            // Clamp it to the edge it started on: a rubber-band only springs back
            // toward its own edge, never to the far one — letting it cross flips
            // the pos reconciliation below and teleports the view to the far edge.
            sp.stretch = if sp.snap_x0 >= 0.0 {
                sp.stretch.max(0.0)
            } else {
                sp.stretch.min(0.0)
            };
            if sp.stretch.abs() < 1.0 {
                sp.stretch = 0.0;
                sp.pos = sp.offset;
                sp.state = ScrollState::Idle;
            } else {
                // Keep the unclamped position consistent with the decaying
                // stretch so a new drag resumes smoothly.
                let excess = rb_pull_inverse(sp.stretch.abs(), dim, &sp.tuning);
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
