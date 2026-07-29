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

/// Register upstream makepad's widgets, then this crate's Material-3 variants,
/// re-referencing them into the prelude the Splash mount resolves.
pub fn widgets_mod(vm: &mut ScriptVm) {
    makepad_widgets::widgets_mod(vm);
    self::script_mod(vm);
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
}

// TODO(kits): Button touch-ripple as a `RippleButton` variant (it modifies the
// base Button shader, so it's a fuller variant, not just colours); iOS and
// liquid-glass control variants alongside the Material ones above.
