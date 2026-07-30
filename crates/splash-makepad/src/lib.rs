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

pub mod kit;

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
        // makepad ships an OpenStreetMap vector-tile renderer with rotation and
        // tilt. A map is a widget here, not a platform view — which is why
        // `google_maps` is portable after all.
        NodeKind::Map => "MapView",
        NodeKind::Shader => "FlutterShader",
        NodeKind::Sdf => "FlutterSdf",
        // No web surface on this backend; the screen branches on st.backend.
        // A real scroll. Safe only once `is_container` stopped deciding
        // child-emission by the mapped widget name — this arm dropped the
        // children of every scroll in the kit before that.
        NodeKind::Scroll => "ScrollYView",
        NodeKind::Web => "View",
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

/// Whether a tap on this node has to be delegated to an overlaid `Button`.
///
/// `on_click` is a `ScriptFnRef` field on `Button`, `CheckBox` and `GlassPanel`
/// and **nowhere else** — a `View` parses the property and silently ignores it,
/// so a row, card or list item carrying `tapto` was completely dead. `Button`
/// in turn takes no children, so the tappable region cannot simply *be* one.
///
/// The way out is an overlay: wrap the container in an `Overlay` view holding
/// the original content plus a transparent, content-sized `Button` on top. The
/// Button owns the hit area and the callback; the content underneath is
/// untouched. Verified on device — a plain row with `tapto` does nothing, the
/// same row under this wrapper navigates.
/// True for a node that holds children.
///
/// A real test on the kind, not `widget_name(kind) == "View"`. That test worked
/// only by accident: every container mapped to the string "View", so comparing
/// the mapped name happened to agree with asking about the kind. It stops
/// agreeing the moment any container maps to something else — mapping `Scroll`
/// to `ScrollYView` silently dropped the children of every scroll in the kit,
/// which is the same failure the RoundedShadowView fix was supposed to have
/// ended. Half-fixed then: the string compare survived behind a comment saying
/// it had been removed.
fn is_container(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Column
            | NodeKind::Row
            | NodeKind::Stack
            | NodeKind::Scroll
            | NodeKind::List
            | NodeKind::Grid
            | NodeKind::Waterflow
            | NodeKind::Refresh
            | NodeKind::Swiper
            | NodeKind::Web
    )
}

fn needs_click_overlay(node: &UiNode) -> bool {
    node.attrs.tapto.is_some() && is_container(node.kind)
}

fn emit(node: &UiNode, out: &mut String, depth: usize) {
    if needs_click_overlay(node) {
        emit_click_overlay(node, out, depth);
        return;
    }
    emit_widget(node, out, depth);
}

/// `Overlay{ <content>, Button{…on_click} }` — see [`needs_click_overlay`].
fn emit_click_overlay(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let inner_ind = "    ".repeat(depth + 1);
    let a = &node.attrs;

    let _ = writeln!(out, "{ind}View {{");
    let _ = writeln!(out, "{inner_ind}flow: Overlay");
    // The wrapper takes over the node's outer size so the Button, which fills
    // it, ends up exactly the size of the content it covers.
    match a.w {
        Some(w) => {
            let _ = writeln!(out, "{inner_ind}width: {w}");
        }
        None => {
            let _ = writeln!(out, "{inner_ind}width: Fill");
        }
    }
    match a.h {
        Some(h) => {
            let _ = writeln!(out, "{inner_ind}height: {h}");
        }
        None => {
            let _ = writeln!(out, "{inner_ind}height: Fit");
        }
    }

    // The content, with `tapto` stripped so it does not re-enter this path.
    let mut content = node.clone();
    content.attrs.tapto = None;
    emit_widget(&content, out, depth + 1);

    // The hit target: an empty ButtonFlatter filling the wrapper.
    //
    // `ButtonFlatter`, not `Button`. A plain Button carries the theme's chrome,
    // so every tappable row in the kit was drawn with a visible outline around
    // it — on the index, every list row, every settings row. ButtonFlatter is
    // upstream's own fully transparent variant: it sets colour and every border
    // colour to `theme.color_u_hidden` rather than injecting properties the
    // shader has no instance for.
    //
    // That last part is why the obvious fix failed before. An earlier version
    // set `draw_bg +: { color: #00000000, border_size: 0.0 }` and the button
    // stopped responding entirely — the themed `draw_bg` shader has no
    // `border_size` instance, and the bad merge takes the whole widget out.
    // One property per line for the same reason: the comma-joined form did not
    // parse.
    let target = a.tapto.as_ref().expect("checked by needs_click_overlay");
    let _ = writeln!(out, "{inner_ind}ButtonFlatter {{");
    let _ = writeln!(out, "{inner_ind}    width: Fill");
    let _ = writeln!(out, "{inner_ind}    height: Fill");
    let _ = writeln!(out, "{inner_ind}    text: \"\"");
    let _ = writeln!(
        out,
        "{inner_ind}    on_click: || {{ ui.nav_signal.set_text({target:?}) }}"
    );
    let _ = writeln!(out, "{inner_ind}}}");
    let _ = writeln!(out, "{ind}}}");
}

fn emit_widget(node: &UiNode, out: &mut String, depth: usize) {
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
    // Only containers carry children — decided by the node's *kind*, and now
    // actually so. This read `widget_name(node.kind) == "View"`, which is a
    // question about the mapped name wearing the comment of a question about the
    // kind. See `is_container`.
    if is_container(node.kind) {
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

    // Map camera. Field names are MapView's own (widgets/src/map/view.rs).
    if node.kind == NodeKind::Map {
        if let Some(v) = a.lat {
            let _ = writeln!(out, "{ind}center_lat: {v}");
        }
        if let Some(v) = a.lon {
            let _ = writeln!(out, "{ind}center_lon: {v}");
        }
        if let Some(v) = a.zoom {
            let _ = writeln!(out, "{ind}zoom: {v}");
        }
        if let Some(v) = a.tilt {
            let _ = writeln!(out, "{ind}tilt: {v}");
        }
        if let Some(v) = a.rotation {
            let _ = writeln!(out, "{ind}rotation: {v}");
        }
    }

    // Control state.
    //
    // `on`, `value` and `total` were declared in `Attrs` and then never emitted,
    // so every `{t:"checkbox", on: 1}` rendered *unchecked* and every
    // `{t:"slider", value: 0.35}` sat at its default — with a caption beside it
    // claiming 0.35. The controls were real native widgets whose declared state
    // never reached them.
    //
    // Field names are makepad's, not invented: `CheckBox` (and `Toggle`, which
    // is a CheckBox variant — see widgets/src/check_box.rs) exposes
    // `active: Option<bool>`; `Slider` has min/max/step/`default` and no `value`.
    if let Some(on) = a.on {
        if matches!(
            node.kind,
            NodeKind::Checkbox | NodeKind::Toggle | NodeKind::Radio
        ) {
            let _ = writeln!(out, "{ind}active: {}", on != 0);
        }
    }
    if let Some(v) = a.value {
        if node.kind == NodeKind::Slider {
            // The kit expresses slider positions as a 0..1 fraction, so pin the
            // range unless the caller gave an explicit `total`.
            let _ = writeln!(out, "{ind}min: 0.0");
            let _ = writeln!(out, "{ind}max: {}", a.total.unwrap_or(1.0));
            let _ = writeln!(out, "{ind}default: {v}");
        }
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
    fn a_raised_container_keeps_its_children() {
        // An elevated card promotes to RoundedShadowView. It is still a
        // container: dropping its children rendered every Material "elevated
        // card with content" as an empty shadow box.
        let ui = to_makepad_ui(&tree(
            r#"{t:"column", bg: 4294901760, radius: 12, elevation: 3, c:[
                   {t:"text", text:"content", h: 20}
               ]}"#,
        ));
        assert!(ui.contains("RoundedShadowView {"), "{ui}");
        assert!(
            ui.contains("text: \"content\""),
            "a raised container must still emit its children:\n{ui}"
        );
    }

    #[test]
    fn control_state_reaches_the_widget() {
        // Declared state used to be dropped on the floor: the attribute existed
        // and nothing emitted it.
        let ui = to_makepad_ui(&tree(r#"{t:"checkbox", on: 1}"#));
        assert!(ui.contains("active: true"), "checkbox keeps its state:\n{ui}");

        let ui = to_makepad_ui(&tree(r#"{t:"toggle", on: 0}"#));
        assert!(ui.contains("active: false"), "toggle keeps its state:\n{ui}");

        let ui = to_makepad_ui(&tree(r#"{t:"slider", value: 0.35}"#));
        assert!(ui.contains("default: 0.35"), "slider keeps its value:\n{ui}");
        assert!(ui.contains("max: 1"), "0..1 range unless total says otherwise:\n{ui}");

        // A container is not a control; `on` there means nothing.
        let ui = to_makepad_ui(&tree(r#"{t:"column", on: 1}"#));
        assert!(!ui.contains("active:"), "containers take no state:\n{ui}");
    }

    #[test]
    fn a_tappable_container_gets_a_button_over_it() {
        // A View ignores `on_click`, so a row carrying `tapto` must be wrapped in
        // an Overlay with a real Button on top or the tap is silently dropped.
        let ui = to_makepad_ui(&tree(
            r#"{t:"row", h: 56, tapto:"date_planner", c:[
                   {t:"text", text:"Date Planner", h: 20}
               ]}"#,
        ));
        assert!(ui.contains("flow: Overlay"), "needs an overlay wrapper:\n{ui}");
        // ButtonFlatter specifically: a plain `Button` carries the theme's
        // chrome and drew a visible outline around every tappable row in the
        // kit. Asserting the transparent variant is what keeps that from
        // coming back.
        assert!(
            ui.contains("ButtonFlatter {"),
            "the hit target must be the transparent Button variant:\n{ui}"
        );
        assert!(
            ui.contains(r#"on_click: || { ui.nav_signal.set_text("date_planner") }"#),
            "the Button carries the handler:\n{ui}"
        );
        // The content survives, and does not itself carry a dead handler.
        assert!(ui.contains("text: \"Date Planner\""), "{ui}");
        assert_eq!(ui.matches("on_click").count(), 1, "exactly one handler:\n{ui}");
    }

    #[test]
    fn a_tappable_button_needs_no_overlay() {
        // Button supports on_click natively — wrapping it would be pure overhead.
        let ui = to_makepad_ui(&tree(r#"{t:"button", label:"Go", tapto:"index"}"#));
        assert!(!ui.contains("flow: Overlay"), "no wrapper needed:\n{ui}");
        assert_eq!(ui.matches("Button {").count(), 1, "{ui}");
        assert!(ui.contains("on_click"), "{ui}");
    }

    #[test]
    fn a_leaf_widget_still_drops_stray_children() {
        // A Button is not a container; children under it are not emitted.
        let ui = to_makepad_ui(&tree(
            r#"{t:"button", label:"Tap", c:[ {t:"text", text:"nope", h: 20} ]}"#,
        ));
        assert!(ui.contains("text: \"Tap\""));
        assert!(!ui.contains("nope"), "leaf widgets carry no children:\n{ui}");
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
