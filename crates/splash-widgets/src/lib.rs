//! splash-widgets — themed native-widget kits for makepad, defined as **external**
//! `script_mod!` variants so **no makepad fork is needed**.
//!
//! ## The fork-free finding (verified on-device)
//! makepad's native widgets (checkbox, switch, radio, slider, text field) are
//! drawn by their own MPSL shaders; their look is *not* reachable from the Splash
//! DSL. But it **is** reachable from an external crate, with one crucial caveat:
//!
//! * A **runtime** `script_eval!` override of `mod.prelude.widgets.*` **drops the
//!   shader** — the widget renders blank. ❌
//! * A **compiled** `script_mod!` that extends the base widget (e.g.
//!   `mod.widgets.CheckBox = mod.widgets.CheckBoxFlat{ draw_bg +: {…} }`) and then
//!   **references** the result into the prelude **keeps the shader**. ✅
//!
//! The difference: the `script_mod!` macro compiles the MPSL at build time; a
//! runtime string is never compiled. So this crate restyles makepad's widgets
//! against **upstream** makepad — no fork, no upstream PR.
//!
//! ## Usage
//! Call [`widgets_mod`] instead of `makepad_widgets::widgets_mod` when setting up
//! the app VM. It registers upstream's widgets, then our themed variants, then
//! re-points the prelude (what a mounted Splash body resolves) at them.
//!
//! Each new theme (iOS, liquid-glass) is just more variants like these.

use makepad_widgets::*;

pub mod tap;

/// Register upstream makepad's widgets, then this crate's Material-3 variants,
/// re-referencing them into the prelude the Splash mount resolves.
pub fn widgets_mod(vm: &mut ScriptVm) {
    makepad_widgets::widgets_mod(vm);
    self::script_mod(vm);
    self::tap::script_mod(vm);
}

script_mod! {
    use mod.prelude.widgets.*

    // ---- Material 3 native controls (M3 light roles) -------------------------
    // primary #6750A4 · on-surface-variant #49454F · surface-variant #E7E0EC ·
    // Each: extend the base (keeps the pixel shader), then reference into the
    // prelude so the runtime Splash mount sees the themed variant.

    // Checkbox: 18dp box, 2dp stroke, transparent + outline -> primary + white tick.
    mod.widgets.CheckBox = mod.widgets.CheckBoxFlat{
        label_walk +: { margin: theme.mspace_h_1{left: 28.} }
        draw_bg +: {
            size: uniform(18.0)
            border_size: uniform(2.0)
            border_radius: uniform(4.0)
            color: #00000000
            color_hover: #00000000
            color_down: #00000000
            color_focus: #00000000
            color_active: #6750A4
            border_color: #49454F
            border_color_hover: #49454F
            border_color_down: #6750A4
            border_color_active: #6750A4
            border_color_focus: #49454F
            mark_color: #00000000
            mark_color_active: #FFFFFF
            mark_color_active_hover: #FFFFFF
        }
    }
    mod.prelude.widgets.CheckBox = mod.widgets.CheckBox

    // Switch: surface-variant track + outline thumb -> primary track + white thumb.
    // (Default pill geometry — makepad's Toggle shader distorts into a lens at
    // larger sizes; only recolour.)
    mod.widgets.Toggle = mod.widgets.ToggleFlat{
        label_walk +: { margin: theme.mspace_h_1{left: 34.} }
        draw_bg +: {
            color: #E7E0EC
            color_hover: #E7E0EC
            color_active: #6750A4
            border_color: #49454F
            border_color_active: #6750A4
            mark_color: #49454F
            mark_color_active: #FFFFFF
            mark_color_active_hover: #FFFFFF
        }
    }
    mod.prelude.widgets.Toggle = mod.widgets.Toggle

    // Radio: 20dp ring (on-surface-variant unselected, primary selected + dot).
    mod.widgets.RadioButton = mod.widgets.RadioButtonFlat{
        label_walk +: { margin: theme.mspace_h_1{left: 30.} }
        draw_bg +: {
            size: uniform(20.0)
            border_size: uniform(2.0)
            color: #00000000
            color_active: #00000000
            border_color: #49454F
            border_color_active: #6750A4
            mark_color: #00000000
            mark_color_active: #6750A4
        }
    }
    mod.prelude.widgets.RadioButton = mod.widgets.RadioButton

    // Slider: primary active track + handle, neutral inactive track.
    mod.widgets.Slider = mod.widgets.SliderFlat{
        draw_bg +: {
            val_color: #6750A4
            val_color_hover: #6750A4
            handle_color: #6750A4
            handle_color_hover: #6750A4
            handle_color_2: #6750A4
            border_color: #CAC4D0
            border_color_2: #CAC4D0
        }
    }
    mod.prelude.widgets.Slider = mod.widgets.Slider

    // Text field: outlined, on-surface-variant border -> primary on focus.
    mod.widgets.TextInput = mod.widgets.TextInputFlat{
        draw_bg +: {
            border_radius: uniform(4.0)
            border_size: uniform(1.0)
            color: #FEF7FF
            color_focus: #FEF7FF
            color_empty: #FEF7FF
            border_color: #49454F
            border_color_focus: #6750A4
            border_color_empty: #49454F
        }
    }
    mod.prelude.widgets.TextInput = mod.widgets.TextInput

    // ---- Custom widgets -----------------------------------------------------
    // LoadingMorph — the M3 shape-morph loading indicator: a solid shape that
    // cycles circle -> square -> pill -> rounded-rect while rotating, off
    // draw_pass.time (the app's redraw loop keeps it animating).
    mod.widgets.LoadingMorph = View{
        width: Fill
        height: Fill
        show_bg: true
        draw_bg +: {
            color: uniform(#6750A4)
            pixel: fn() {
                let center = self.rect_size * 0.5
                let t = self.draw_pass.time
                let a = t * 1.4
                let cs = cos(a)
                let sn = sin(a)
                let rel = self.pos * self.rect_size - center
                let rot = vec2(rel.x * cs - rel.y * sn, rel.x * sn + rel.y * cs) + center
                let sdf = Sdf2d.viewport(rot)
                let base = min(center.x, center.y) * 0.62
                let ph = t * 0.5
                let ph_mod = ph - floor(ph / 4.0) * 4.0
                let k = floor(ph_mod)
                let e = smoothstep(0.0, 1.0, ph_mod - k)
                let mut wa = 1.0
                let mut ha = 1.0
                let mut ca = 1.0
                if k > 0.5 { wa = 1.0  ha = 1.0  ca = 0.30 }
                if k > 1.5 { wa = 1.42 ha = 0.60 ca = 0.60 }
                if k > 2.5 { wa = 1.24 ha = 0.80 ca = 0.22 }
                let mut k2 = k + 1.0
                if k2 > 3.5 { k2 = 0.0 }
                let mut wb = 1.0
                let mut hb = 1.0
                let mut cb = 1.0
                if k2 > 0.5 { wb = 1.0  hb = 1.0  cb = 0.30 }
                if k2 > 1.5 { wb = 1.42 hb = 0.60 cb = 0.60 }
                if k2 > 2.5 { wb = 1.24 hb = 0.80 cb = 0.22 }
                let w = mix(wa, wb, e) * base
                let h = mix(ha, hb, e) * base
                let corner = mix(ca, cb, e) * base
                sdf.box(center.x - w, center.y - h, w * 2.0, h * 2.0, corner)
                return sdf.fill(self.color)
            }
        }
    }
    mod.prelude.widgets.LoadingMorph = mod.widgets.LoadingMorph

    // ---- flutter/samples' two fragment-shader samples ------------------------
    //
    // `simple_shader` and `simple_sdf` were written off as unportable on the
    // grounds that MPSL compiles at build time and no DSL node carries shader
    // source. Both halves are true; the conclusion was not. A *compiled*
    // variant here, selected by name from the DSL, is the way in — the same
    // route this crate already uses to theme makepad's controls without a fork.
    //
    // Both are transcribed from the samples' own GLSL.

    // simple_shader/shaders/simple.frag — a diagonal gradient through Flutter's
    // sky -> blue -> navy.
    mod.widgets.FlutterShader = View{
        width: Fill
        height: Fill
        show_bg: true
        draw_bg +: {
            color: uniform(#0553B1)
            pixel: fn() {
                let sky  = vec3(2.0, 125.0, 253.0) / 255.0
                let blue = vec3(5.0, 83.0, 177.0) / 255.0
                let navy = vec3(4.0, 43.0, 89.0) / 255.0
                let p = (self.pos.x + self.pos.y) * 0.5
                let mut c = mix(sky, blue, p * 2.0)
                if p >= 0.5 { c = mix(blue, navy, p * 2.0 - 1.0) }
                return vec4(c, 1.0)
            }
        }
    }
    mod.prelude.widgets.FlutterShader = mod.widgets.FlutterShader

    // simple_sdf/shaders/SDF.frag — sdHeart, pink on black, smoothstepped.
    mod.widgets.FlutterSdf = View{
        width: Fill
        height: Fill
        show_bg: true
        draw_bg +: {
            color: uniform(#FF69B4)
            pixel: fn() {
                let mut q = (self.pos - vec2(0.5, 0.5)) * 2.0
                q.y = 0.0 - (q.y - 0.5)
                q.x = abs(q.x)
                let mut d = 0.0
                if q.y + q.x > 1.0 {
                    let u = q - vec2(0.25, 0.75)
                    d = sqrt(dot(u, u)) - 1.41421356 / 4.0
                } else {
                    let a = q - vec2(0.0, 1.0)
                    let m = max(q.x + q.y, 0.0)
                    let b = q - 0.5 * m
                    d = sqrt(min(dot(a, a), dot(b, b))) * sign(q.x - q.y)
                }
                let pink = vec3(255.0, 105.0, 180.0) / 255.0
                let c = mix(pink, vec3(0.0, 0.0, 0.0), smoothstep(0.01, 0.02, d))
                return vec4(c, 1.0)
            }
        }
    }
    mod.prelude.widgets.FlutterSdf = mod.widgets.FlutterSdf

    // ---- L0 data visualisations ---------------------------------------------
    //
    // `ui-profile-l0.md` §1.1: six roles are small data visualisations rather
    // than compositions of boxes and text, and a DSL node cannot carry shader
    // SOURCE because MPSL compiles at build time. It can name a shader that was
    // compiled, which is what these are.
    //
    // THREE OF THE FIVE ARE PURE FUNCTIONS OF THEIR PARAMETERS and are drawn
    // here in full. The other two are not: an air-quality field and a price
    // series are DATA, and this backend has no fetch. They draw an explicit
    // empty state rather than a plausible-looking curve — a card that looks
    // right and is fiction is worse than one that says it has nothing, and §4's
    // no-facts rule is the same instinct one layer down.

    // A day's low and high as a segment of the week's range.
    mod.widgets.L0TempBar = View{
        width: Fill
        height: 8
        show_bg: true
        draw_bg +: {
            tlo:  uniform(0.0)
            thi:  uniform(0.0)
            wmin: uniform(0.0)
            wmax: uniform(1.0)
            pixel: fn() {
                let span = max(self.wmax - self.wmin, 0.001)
                let a = clamp((self.tlo - self.wmin) / span, 0.0, 1.0)
                let b = clamp((self.thi - self.wmin) / span, 0.0, 1.0)
                let x = self.pos.x
                // The track, so an unfilled range still reads as a range.
                let track = vec3(1.0, 1.0, 1.0) * 0.10
                if x < a || x > b { return vec4(track, 1.0) }
                // Cold to warm ACROSS THE SEGMENT, so a wide day reads warmer at
                // its top than a narrow one at the same high.
                let t = (x - a) / max(b - a, 0.001)
                let cold = vec3(90.0, 160.0, 240.0) / 255.0
                let warm = vec3(255.0, 170.0, 60.0) / 255.0
                return vec4(mix(cold, warm, t), 1.0)
            }
        }
    }
    mod.prelude.widgets.L0TempBar = mod.widgets.L0TempBar

    // The sun's path, with now marked on it. Hours are fractional: 18.9 is 18:54.
    mod.widgets.L0SunArc = View{
        width: Fill
        height: 90
        show_bg: true
        draw_bg +: {
            rise: uniform(6.0)
            set:  uniform(18.0)
            now:  uniform(12.0)
            pixel: fn() {
                let p = self.pos
                // A half-ellipse across the width, sitting on the baseline.
                let ax = p.x
                let ay = 1.0 - sin(ax * 3.14159265)
                let d = abs(p.y - ay)
                let ink = vec3(1.0, 1.0, 1.0) * 0.35
                let mut c = vec3(0.0, 0.0, 0.0)
                let mut a = 0.0
                if d < 0.035 { c = ink; a = 1.0 - smoothstep(0.02, 0.035, d) }
                // Where the sun is now, clamped so a pre-dawn or post-dusk time
                // sits at the horizon rather than off the arc.
                let span = max(self.set - self.rise, 0.001)
                let t = clamp((self.now - self.rise) / span, 0.0, 1.0)
                let sx = t
                let sy = 1.0 - sin(sx * 3.14159265)
                let sd = length(vec2(p.x - sx, p.y - sy))
                if sd < 0.06 {
                    let sun = vec3(255.0, 200.0, 80.0) / 255.0
                    let sa = 1.0 - smoothstep(0.035, 0.06, sd)
                    c = mix(c, sun, sa)
                    a = max(a, sa)
                }
                return vec4(c, a)
            }
        }
    }
    mod.prelude.widgets.L0SunArc = mod.widgets.L0SunArc

    // A disc with its terminator at `phase`, 0..1 through the cycle.
    mod.widgets.L0MoonPhase = View{
        width: 64
        height: 64
        show_bg: true
        draw_bg +: {
            phase: uniform(0.5)
            pixel: fn() {
                let q = (self.pos - vec2(0.5, 0.5)) * 2.0
                let r = length(q)
                if r > 1.0 { return vec4(0.0, 0.0, 0.0, 0.0) }
                let edge = 1.0 - smoothstep(0.94, 1.0, r)
                let dark = vec3(1.0, 1.0, 1.0) * 0.12
                let lit  = vec3(1.0, 1.0, 1.0) * 0.92
                // The terminator is an ellipse whose width tracks the phase; at
                // 0 and 1 it covers the disc, at 0.5 it is a straight edge.
                let k = cos(self.phase * 6.2831853)
                let tx = q.x - k * sqrt(max(1.0 - q.y * q.y, 0.0))
                let mut c = dark
                if self.phase < 0.5 {
                    if tx < 0.0 { c = lit }
                } else {
                    if tx > 0.0 { c = lit }
                }
                return vec4(c, edge)
            }
        }
    }
    mod.prelude.widgets.L0MoonPhase = mod.widgets.L0MoonPhase

    // An air-quality field. This backend cannot fetch one, so it says so.
    //
    // Drawing a plausible gradient here would be inventing data — the card would
    // look complete and be fiction, which is the failure §1.1 wants a missing
    // visualisation to avoid, and worse than the marker it replaced.
    mod.widgets.L0AqiContour = View{
        width: Fill
        height: 190
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                // A hatched empty state: unmistakably "no data", not a field.
                let s = (self.pos.x + self.pos.y) * 22.0
                let h = abs(fract(s) - 0.5) * 2.0
                let bg = vec3(1.0, 1.0, 1.0) * 0.05
                let ln = vec3(1.0, 1.0, 1.0) * 0.11
                return vec4(mix(ln, bg, smoothstep(0.35, 0.5, h)), 1.0)
            }
        }
    }
    mod.prelude.widgets.L0AqiContour = mod.widgets.L0AqiContour

    // A price series. Same reasoning: no fetch here, so no invented curve.
    mod.widgets.L0StockPlot = View{
        width: Fill
        height: 180
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let s = (self.pos.x + self.pos.y) * 22.0
                let h = abs(fract(s) - 0.5) * 2.0
                let bg = vec3(1.0, 1.0, 1.0) * 0.05
                let ln = vec3(1.0, 1.0, 1.0) * 0.11
                return vec4(mix(ln, bg, smoothstep(0.35, 0.5, h)), 1.0)
            }
        }
    }
    mod.prelude.widgets.L0StockPlot = mod.widgets.L0StockPlot
}

// TODO(kits): Button touch-ripple as a `RippleButton` variant (it modifies the
// base Button shader, so it's a fuller variant, not just colours); iOS and
// liquid-glass control variants alongside the Material ones above.
