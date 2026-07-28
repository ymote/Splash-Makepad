//! makepad render backend for Splash.
//!
//! The shared [`splash_render`] core evaluates the Splash DSL (in the
//! renderer-free makepad-script VM) into a backend-agnostic [`UiNode`] tree.
//! This crate is one **render backend**: it turns that tree into makepad's own
//! UI dialect — the `View{…}/Label{…}` component script that makepad's widget
//! system renders natively (see `examples/counter` upstream: the UI is declared
//! in script and rendered by `makepad_widgets`).
//!
//! Translating to makepad's native dialect (rather than reimplementing
//! immediate-mode drawing) is deliberate: it reuses makepad's real widgets,
//! layout, and theming, and keeps this backend small. Splash-OH's ArkUI backend
//! is the sibling that builds native ArkUI nodes instead — same `UiNode`, two
//! backends, one shared VM. That is the whole point of the split: makepad is
//! *one* render backend, not *the* renderer.
//!
//! The last mile — feeding [`to_makepad_ui`]'s output into a live makepad
//! `Window` and calling `render()` — is a thin app shell over `makepad_widgets`
//! (see the module docs on [`wiring`]); this crate keeps the translation pure
//! and unit-tested so it needs no window to verify.

use splash_render::{NodeKind, UiNode};
use std::fmt::Write as _;

/// Translate a `UiNode` tree into makepad component-script UI source.
///
/// Containers ([`NodeKind::Column`]/`Row`/`Stack`/`Scroll`/…) become `View`s
/// with the matching `flow`; `Text` → `Label`, `Button` → `Button`, `Image` →
/// `Image`, text inputs → `TextInput`. Attributes map to makepad props
/// (`bg` → `show_bg`+`draw_bg.color`, `size` → `draw_text` font size, etc.).
pub fn to_makepad_ui(root: &UiNode) -> String {
    let mut out = String::new();
    emit(root, &mut out, 0);
    out
}

/// The makepad widget a kind renders as.
fn widget_name(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Text => "Label",
        NodeKind::Button => "Button",
        NodeKind::Image => "Image",
        NodeKind::Input | NodeKind::Textarea => "TextInput",
        NodeKind::Slider => "Slider",
        NodeKind::Checkbox => "CheckBox",
        NodeKind::Toggle => "Toggle",
        NodeKind::Radio => "RadioButton",
        // A real, continuously-animated Material circular indicator (its shader
        // spins off draw_pass.time). `bg` recolours the arc via draw_bg.color.
        NodeKind::Loading => "LoadingSpinner",
        // The M3 loading indicator: a solid shape that morphs + rotates.
        NodeKind::Progress => "LoadingMorph",
        // every container-ish kind is a View with the right flow.
        _ => "View",
    }
}

/// Layout flow for container kinds.
fn flow(kind: NodeKind) -> Option<&'static str> {
    match kind {
        NodeKind::Row => Some("Right"),
        NodeKind::Stack => Some("Overlay"),
        k if k.is_vertical_stack() => Some("Down"),
        _ => None,
    }
}

fn emit(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let name = widget_for(node);
    // An `id` makes the widget addressable in the mounted tree: `name := Widget{…}`.
    match &node.attrs.id {
        Some(wid) => {
            let _ = writeln!(out, "{ind}{wid} := {name} {{");
        }
        None => {
            let _ = writeln!(out, "{ind}{name} {{");
        }
    }
    emit_attrs(node, out, depth + 1);
    // Only container views carry children.
    if name == "View" || name == "RoundedView" {
        for c in &node.children {
            emit(c, out, depth + 1);
        }
    }
    let _ = writeln!(out, "{ind}}}");
}

/// The concrete makepad widget for a node. A container that carries a background
/// or corner radius must be a `RoundedView` — plain `View` does not paint
/// `draw_bg` in makepad, which renders such containers as (invisible) empty
/// boxes with only their text children showing.
fn widget_for(node: &UiNode) -> &'static str {
    let base = widget_name(node.kind);
    let a = &node.attrs;
    if base == "View" {
        // A raised container casts a drop shadow (Material elevation).
        if a.elevation.is_some() {
            return "RoundedShadowView";
        }
        if a.bg.is_some() || a.radius.is_some() || a.border.is_some() {
            return "RoundedView";
        }
    }
    base
}

fn emit_attrs(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let a = &node.attrs;

    if let Some(f) = flow(node.kind) {
        let _ = writeln!(out, "{ind}flow: {f}");
    }
    // Explicit alignment wins; otherwise a row centres its children vertically
    // (a widget and its label line up), matching how ArkUI lays a row out.
    if a.alignx.is_some() || a.aligny.is_some() {
        let x = a.alignx.unwrap_or(0.0);
        let y = a.aligny.unwrap_or(0.0);
        let _ = writeln!(out, "{ind}align: Align{{x: {x}, y: {y}}}");
    } else if node.kind == NodeKind::Row {
        let _ = writeln!(out, "{ind}align: Align{{y: 0.5}}");
    }
    // makepad containers default to Fill on both axes, which makes a card
    // stretch far past its content. A container with no explicit size hugs its
    // content vertically (`Fit`) and fills horizontally — the ArkUI default.
    let container = flow(node.kind).is_some();
    match a.w {
        Some(w) => {
            let _ = writeln!(out, "{ind}width: {w}");
        }
        None if a.fillw == Some(1) => {
            let _ = writeln!(out, "{ind}width: Fill");
        }
        None if a.fitw == Some(1) => {
            let _ = writeln!(out, "{ind}width: Fit");
        }
        None if container => {
            let _ = writeln!(out, "{ind}width: Fill");
        }
        None => {}
    }
    match a.h {
        Some(h) => {
            let _ = writeln!(out, "{ind}height: {h}");
        }
        None if a.fillh == Some(1) => {
            let _ = writeln!(out, "{ind}height: Fill");
        }
        None if a.fith == Some(1) => {
            let _ = writeln!(out, "{ind}height: Fit");
        }
        None if container => {
            let _ = writeln!(out, "{ind}height: Fit");
        }
        None => {}
    }
    // Padding: asymmetric padx/pady (each overriding pad on its axis) emit a
    // per-side object; otherwise a uniform pad. Enables M3's asymmetric insets
    // (e.g. a button's 24dp horizontal / 6dp vertical padding).
    if a.padx.is_some() || a.pady.is_some() {
        let px = a.padx.or(a.pad).unwrap_or(0.0);
        let py = a.pady.or(a.pad).unwrap_or(0.0);
        let _ = writeln!(
            out,
            "{ind}padding: {{left: {px}, right: {px}, top: {py}, bottom: {py}}}"
        );
    } else if let Some(p) = a.pad {
        let _ = writeln!(out, "{ind}padding: {p}");
    }
    if let Some(sp) = a.spacing {
        let _ = writeln!(out, "{ind}spacing: {sp}");
    }
    // bg / radius / border share one draw_bg block. A RoundedView defaults to a
    // transparent fill, so a border with no bg paints a clean outline (Material
    // "outlined" components); a border needs show_bg so the outline is painted.
    if a.bg.is_some() || a.radius.is_some() || a.border.is_some() || a.elevation.is_some() {
        if a.bg.is_some() || a.border.is_some() || a.elevation.is_some() {
            let _ = writeln!(out, "{ind}show_bg: true");
        }
        let mut parts = Vec::new();
        if let Some(bg) = a.bg {
            parts.push(format!("color: {}", hex_rgba(bg)));
        }
        if let Some(r) = a.radius {
            parts.push(format!("radius: {r}"));
        }
        if let Some(b) = a.border {
            parts.push(format!("border_size: {b}"));
        }
        if let Some(bc) = a.bordercolor {
            parts.push(format!("border_color: {}", hex_rgba(bc)));
        }
        if let Some(e) = a.elevation {
            // Material elevation → a soft drop shadow scaled by the dp value
            // (RoundedShadowView's shadow_* instances).
            let radius = 3.0 + e * 2.0;
            let dy = 1.0 + e * 0.6;
            parts.push("shadow_color: #00000033".to_string());
            parts.push(format!("shadow_radius: {radius}"));
            parts.push(format!("shadow_offset: vec2(0.0, {dy})"));
        }
        // `draw_bg +:` merges onto the widget's draw shader (makepad convention).
        let _ = writeln!(out, "{ind}draw_bg +: {{ {} }}", parts.join(", "));
    }
    // Text goes on both Label and Button.
    if let Some(t) = a.text.as_ref().or(a.label.as_ref()) {
        let _ = writeln!(out, "{ind}text: {t:?}");
    }
    // A placeholder maps to a TextInput's `empty_text` (shown when unfocused/empty).
    if let Some(ph) = a.placeholder.as_ref() {
        let _ = writeln!(out, "{ind}empty_text: {ph:?}");
    }
    // `tapto` wires an on_click that writes the route into the `nav_signal`
    // widget; the host app reads that text and re-mounts the target screen.
    if let Some(target) = a.tapto.as_ref() {
        let _ = writeln!(
            out,
            "{ind}on_click: || {{ ui.nav_signal.set_text({target:?}) }}"
        );
    }
    if let Some(s) = a.size {
        // icon selects the theme's Font-Awesome face (monochrome icons);
        // else weight >= 500 selects the Medium (bold) face — M3's label / title /
        // emphasis weight; else just set the size (Regular). Each swaps the whole
        // text_style for the theme style at this size.
        if a.icon == Some(1) {
            let _ = writeln!(
                out,
                "{ind}draw_text.text_style: mod.theme.font_icons{{ font_size: {s} }}"
            );
        } else if a.weight.unwrap_or(400) >= 500 {
            let _ = writeln!(
                out,
                "{ind}draw_text.text_style: mod.theme.font_bold{{ font_size: {s} }}"
            );
        } else {
            let _ = writeln!(out, "{ind}draw_text.text_style.font_size: {s}");
        }
    }
    if let Some(c) = a.color {
        let _ = writeln!(out, "{ind}draw_text.color: {}", hex_rgba(c));
    }
    if let Some(src) = &a.src {
        let _ = writeln!(out, "{ind}source: {src:?}");
    }
}

/// `0xAARRGGBB` (the Splash colour word) → makepad `#RRGGBBAA`.
fn hex_rgba(argb: u32) -> String {
    let a = (argb >> 24) & 0xff;
    let r = (argb >> 16) & 0xff;
    let g = (argb >> 8) & 0xff;
    let b = argb & 0xff;
    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
}

/// Wiring the translation into a live makepad window.
///
/// [`to_makepad_ui`] yields the `body` of a makepad `View`. A host app embeds it
/// under a `Window`/`Root` and renders it — the shape upstream `examples/counter`
/// uses:
///
/// ```text
/// script_mod! {
///     use mod.prelude.widgets.*
///     startup() do #(App::script_component(vm)) {
///         ui: Root{ main_window := Window{ body +: { /* <to_makepad_ui output> */ } } }
///     }
/// }
/// ```
///
/// Generating that `body` at runtime (rather than inline) is the remaining
/// last-mile step; the translation above is the substantive part of the backend
/// and is what the tests cover.
pub mod wiring {}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(src: &str) -> UiNode {
        splash_render::build(src, |_vm| {}).expect("evaluates")
    }

    #[test]
    fn column_of_text_becomes_view_with_label() {
        let ui = to_makepad_ui(&tree(
            r#"fn argb(a,r,g,b){ return ((a*256+r)*256+g)*256+b }
               {t:"column", bg: argb(255,20,20,20), pad: 12, c:[
                   {t:"text", text:"Hi", size: 20, color: argb(255,255,255,255), w:100, h:28}
               ]}"#,
        ));
        assert!(
            ui.contains("RoundedView {"),
            "filled container must be a RoundedView:\n{ui}"
        );
        assert!(ui.contains("flow: Down"));
        assert!(ui.contains("padding: 12"));
        assert!(ui.contains("show_bg: true"));
        assert!(ui.contains("color: #141414ff"));
        assert!(ui.contains("Label {"));
        assert!(ui.contains("text: \"Hi\""));
        assert!(ui.contains("font_size: 20"));
    }

    #[test]
    fn row_becomes_view_flow_right_and_button_maps() {
        let ui = to_makepad_ui(&tree(
            r#"{t:"row", h: 44, c:[ {t:"button", label:"Tap", w: 80, h: 40} ]}"#,
        ));
        assert!(ui.contains("flow: Right"));
        assert!(ui.contains("Button {"));
        assert!(ui.contains("text: \"Tap\""));
    }

    #[test]
    fn computed_tree_translates() {
        // The tree is produced by a VM loop, then translated — end to end.
        let ui = to_makepad_ui(&tree(
            r#"let k=[]; let i=0; while i<3 { k.push({t:"text", text:"r"+i, h:20}); i=i+1 } {t:"column", c:k}"#,
        ));
        assert_eq!(ui.matches("Label {").count(), 3);
        assert!(ui.contains("text: \"r2\""));
    }
}
