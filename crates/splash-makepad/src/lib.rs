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
pub mod material;

use splash_render::{Attrs, NodeKind, UiNode};
use std::fmt::Write as _;

/// The Material colour scheme the semantic components resolve against. The
/// reference states components by role, not by colour, so the roles live here;
/// this is how a host says which set to use.
static SCHEME: std::sync::Mutex<Option<material::Roles>> = std::sync::Mutex::new(None);

pub fn set_scheme(roles: material::Roles) {
    if let Ok(mut s) = SCHEME.lock() {
        *s = Some(roles);
    }
}

/// Convenience for the two built-in schemes.
pub fn set_dark(dark: bool) {
    set_scheme(if dark {
        material::Roles::dark()
    } else {
        material::Roles::light()
    });
}

fn theme() -> material::Roles {
    SCHEME.lock().ok().and_then(|s| *s).unwrap_or_else(material::Roles::light)
}

/// Translate a `UiNode` tree into makepad component-script UI source.
///
/// Containers ([`NodeKind::Column`]/`Row`/`Stack`/`Scroll`/…) become `View`s
/// with the matching `flow`; `Text` → `Label`, `Button` → `Button`, `Image` →
/// `Image`, text inputs → `TextInput`. Attributes map to makepad props
/// (`bg` → `show_bg`+`draw_bg.color`, `size` → `draw_text` font size, etc.).
pub fn to_makepad_ui(root: &UiNode) -> String {
    material::reset_slider_index();
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
        // A ring, as the reference's circular progress draws. `LoadingMorph` is
        // splash-widgets' shape-morph blob — it renders nothing on an isolate and
        // a morphing blob on the main VM, neither of which is a progress ring.
        NodeKind::Progress => "LoadingSpinner",
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
    // A Material node is desugared into primitives first, so it picks up the
    // same corner-radius, colour-role and text handling as everything else.
    // Re-enter rather than emit directly: lowering is where a component gains
    // its `tapto`, and going straight to `emit_widget` skipped the click overlay
    // and left every lowered component inert. This terminates because a lowered
    // node is primitive, and a resolved text role clears the `variant`.
    if let Some(lowered) = material::lower(node, &theme()) {
        emit(&lowered, out, depth);
        return;
    }
    if needs_vertical_pad_wrapper(node) {
        emit_vertical_pad(node, out, depth);
        return;
    }
    if needs_click_overlay(node) {
        emit_click_overlay(node, out, depth);
        return;
    }
    emit_widget(node, out, depth);
}

/// Vertical-only padding (`pady` with no `padx`) on a node that has no fixed
/// height.
///
/// This is the shape the screens use for a list row — `padding: {top: 12,
/// bottom: 12}` — and it is exactly the shape this dialect cannot express: the
/// per-side object is inert, and the scalar form would indent the text
/// horizontally by the same 12dp. Both were measured on device.
///
/// So the inset is built structurally instead, as spacer rows above and below.
/// Without this every row in the app rendered at its bare text height, which is
/// the compression that made long screens drift ~24px per row against the
/// reference.
fn needs_vertical_pad_wrapper(node: &UiNode) -> bool {
    let a = &node.attrs;
    let vertical_only = a.padx.or(a.pad).unwrap_or(0.0) == 0.0
        && a.marginx.or(a.margin).unwrap_or(0.0) == 0.0;
    // Margin is emitted through the same inert object form, so a section
    // header's `margin: {top: 4, bottom: 4}` was dropped exactly like the row
    // padding was. Both are made up structurally, and both at once when a node
    // carries them together.
    // A text node with no stated inset still gets one: `padding: 0` above kills
    // makepad's `Label` default on *both* axes, and only the horizontal half of
    // it was wrong. Restoring the vertical half here is what decouples the two —
    // the axes cannot be set independently through the padding property itself.
    // A text node with no stated inset still gets a small one: Android's TextView
    // font padding is what makes the reference's card rows measure 19dp where
    // these measure 15, and `line_spacing` alone cannot close that without
    // costing the toolbars. 2dp, measured: 3 is clearly worse (36 routes under
    // 9.0 drops to 31) because every box whose padding was derived against 2
    // then needs re-deriving, `shape_box` first.
    let text_default = node.kind == NodeKind::Text
        && a.pady.is_none()
        && a.marginy.is_none()
        && a.pad.is_none();
    (a.h.is_none() && vertical_only && (a.pady.unwrap_or(0.0) + a.marginy.unwrap_or(0.0)) > 0.0)
        || text_default
}

fn emit_vertical_pad(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let inner_ind = "    ".repeat(depth + 1);
    // The widget already carries makepad's own `theme.mspace_1` inset (~4dp), so
    // the spacers make up only the remainder. Adding the full `pady` on top
    // overshot the reference's row pitch by exactly twice that default.
    // The Label default (`theme.mspace_1`) already supplies ~4dp inside a text
    // node, so the spacers make up only the remainder. A View has no such
    // default and takes the full inset.
    // No padding beyond what the DSL states. The reference does leave 173px
    // around a section header where this leaves 162 — measured on `color`, whose
    // offset collapses by exactly 11px at each section boundary — but adding
    // 2dp a side to every stated margin is far worse (38 routes under 9.0 drops
    // to 27): screens carry many section headers and it compounds.
    let stated = node.attrs.pady.unwrap_or(0.0) + node.attrs.marginy.unwrap_or(0.0);
    // 4dp per side is what the zeroed `Label` default was contributing. 6 was
    // tried — it closes the slider screen's ~7dp-per-section shortfall but costs
    // six other routes more than it gains, so the shortfall stays.
    let py = if stated > 0.0 { stated } else { 2.0 };
    // Hug by default. Defaulting to Fill collapsed every label that sits inside
    // a hug-content row — the button labels all vanished while the pixel metric
    // barely moved, which is why this is checked on screen and not only scored.
    let width = if node.attrs.fillw == Some(1) { "Fill" } else { "Fit" };
    let _ = writeln!(out, "{ind}View {{");
    let _ = writeln!(out, "{inner_ind}flow: Down");
    let _ = writeln!(out, "{inner_ind}width: {width}");
    let _ = writeln!(out, "{inner_ind}height: Fit");
    // Symmetric. `color` wants 11px more *above* its section headers and the
    // same below, but giving every stated margin an asymmetric top is far worse
    // (38 routes under 9.0 -> 26): the extra belongs to the widget above the
    // header, not to the header.
    // Symmetric. `button`'s section-header blocks do measure 13px shorter than
    // the reference's, but giving a stated margin more above than below is far
    // worse — re-tested under the aligned layout and still 40 routes under 9.0
    // drops to 36, `color` 4.1 -> 19.5. Unlike the alignment itself, this one
    // does not come good once the layout beneath it is right.
    // A section header takes 4dp more above than below -- but only one that
    // material.rs marked, which is every header except the first on its screen.
    // Emitting it here rather than as a sibling matters: a child of the page's
    // `spacing: 16` column would cost 4dp *plus another 16dp*, 56px where 11 is
    // wanted. These spacers sit inside the wrapper's own `flow: Down`, so 4dp
    // is 4dp.
    let top = if node.attrs.group.as_deref() == Some("sechead") {
        py + 4.0
    } else {
        py
    };
    let _ = writeln!(out, "{inner_ind}View {{ height: {top} }}");
    let mut bare = node.clone();
    // Zero rather than clear, so the rule above does not fire again on re-entry.
    bare.attrs.pady = Some(0.0);
    bare.attrs.pad = None;
    bare.attrs.marginy = Some(0.0);
    bare.attrs.margin = None;
    emit(&bare, out, depth + 1);
    let _ = writeln!(out, "{inner_ind}View {{ height: {py} }}");
    let _ = writeln!(out, "{ind}}}");
}

/// `Overlay{ <content>, Button{…on_click} }` — see [`needs_click_overlay`].
///
/// The handler calls `NAV`, a global the host registers, rather than reaching
/// through `ui.nav_signal`. `ui` is injected by `Splash::eval_body`, so a body
/// mounted anywhere else — notably on the app's main VM, which is what gets this
/// crate's fonts and a widget kit's theming into reach — had every tap silently
/// do nothing. A global works on either VM.
fn emit_click_overlay(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let inner_ind = "    ".repeat(depth + 1);
    let a = &node.attrs;

    // A full-width tappable row hands most of itself back to the scroll.
    //
    // The hit target has to be a Button, because that is the only widget the
    // dialect can attach a handler to. A Button captures the finger on
    // touch-down, so a target covering the whole row leaves the enclosing scroll
    // nothing to drag — and list rows cover nearly the whole screen, which made
    // the catalog feel frozen. Measured: a swipe over a row moved 159 pixel rows
    // of 2340, one clear of the rows moved 1608.
    //
    // So a row that fills its width gets a strip at its trailing edge instead of
    // the whole area. That is where the chevron is on every list row in the kit,
    // it stays comfortably past the 48dp a finger needs, and it leaves the rest
    // of the row free to scroll. Anything with its own width — a chip, a button,
    // a small card — is already narrower than a swipe wants to start on, so it
    // keeps a full-size target.
    //
    // `SplashTap` in splash-widgets is the real answer and cannot be reached: a
    // widget this workspace defines does not resolve inside the isolate VM a
    // mounted Splash allocates. See that module.
    // No edge strip, and no need for one. The target below is `SplashTap`,
    // which never calls `event.hits` and so never captures the finger: it
    // hit-tests itself on `TouchUpdate` and ignores a press that travelled more
    // than its slop. The scroll therefore sees the whole gesture even under a
    // full-width target, and a tap and a swipe can share the same pixels.
    //
    // Both earlier settings were wrong, in opposite directions. A strip on
    // everything shrank Material buttons to a 64dp sliver and an audit called 16
    // of 18 interactions inert. A strip on nothing left a screen of full-width
    // rows -- the catalog index -- with no free pixels for the scroll, so the
    // list would not move and the failed swipe opened whatever row it ended on.
    let target = a.tapto.as_ref().expect("checked by needs_click_overlay");
    let _ = writeln!(out, "{ind}View {{");
    let _ = writeln!(out, "{inner_ind}flow: Overlay");
    // The wrapper takes over the node's outer size so the Button, which fills
    // it, ends up exactly the size of the content it covers.
    // A hug-content target has to stay hugging: forcing Fill made three tabs
    // each claim the whole row, and the strip collapsed to nothing.
    match a.w {
        Some(w) => {
            let _ = writeln!(out, "{inner_ind}width: {w}");
        }
        None if a.fitw == Some(1) => {
            let _ = writeln!(out, "{inner_ind}width: Fit");
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
    // `ButtonFlatter`, upstream's fully transparent Button variant. A plain
    // `Button` drew the theme's chrome as an outline round every tappable row,
    // and hiding that with `draw_bg +: { border_size: 0.0 }` took the whole
    // widget out, because the themed shader has no `border_size` instance. One
    // property per line here for the same reason: the comma-joined form did not
    // parse.
    //
    // `SplashTap` — reachable now. A Splash isolate takes makepad's own script
    // mods and nothing else, so this type used to be unnameable from a body and
    // the target had to be a `Button`, which captures on touch-down and starved
    // the scroll. The host registers this crate's mod into every isolate
    // through `register_splash_isolate_mod`, so the right widget can be used.
    let _ = writeln!(out, "{inner_ind}SplashTap {{");
    let _ = writeln!(out, "{inner_ind}    width: Fill");
    let _ = writeln!(out, "{inner_ind}    height: Fill");
    let _ = writeln!(
        out,
        "{inner_ind}    on_click: || {{ NAV(t: {target:?}) }}"
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
        if a.bg.is_some() || a.bg2.is_some() || a.radius.is_some() || a.border.is_some() {
            return "RoundedView";
        }
    }
    base
}

/// Whether this widget's label carries per-state colours (`draw_text.color_*`).
/// A `Label` has only the base colour; every interactive widget mixes.
fn has_text_states(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Button
            | NodeKind::Checkbox
            | NodeKind::Radio
            | NodeKind::Toggle
            | NodeKind::Slider
            | NodeKind::Input
            | NodeKind::Textarea
    )
}

/// The Material state colours for a native control, as `draw_bg` keys.
///
/// A control is drawn by its own shader, so `bg`/`color` alone cannot describe
/// it. They cannot come from a widget kit either: `Splash` mounts its body on an
/// isolate VM that receives only makepad's own `script_mod`, so variants
/// registered on the app VM are absent there and every control fell back to
/// upstream's grey. A per-instance merge *does* arrive, so the roles travel with
/// the node — and default from the active scheme, because the reference states a
/// control semantically and carries no colour at all.
fn control_roles(kind: NodeKind, a: &Attrs) -> Vec<String> {
    let th = theme();
    let error = a.error.is_some() || a.variant.as_deref() == Some("error");
    let accent = a.accent.or(Some(if error { th.error } else { th.primary }));
    let mark = a.markcolor.or(Some(if error { th.on_error } else { th.on_primary }));
    let outline = a.bordercolor.or(Some(if error { th.error } else { th.on_surface_variant }));
    // Disabled is a whole section per component in the reference.
    let (accent, mark, outline) = if a.enabled.unwrap_or(1) == 0 {
        let d = |c: Option<u32>| c.map(|c| (c & 0x00FF_FFFF) | 0x61000000);
        (d(accent), d(mark), d(outline))
    } else {
        (accent, mark, outline)
    };
    let roles: &[(&str, Option<u32>)] = match kind {
        NodeKind::Checkbox => &[
            ("border_color", if a.bordercolor.is_none() { outline } else { None }),
            ("color_active", accent),
            ("border_color_active", accent),
            ("mark_color_active", mark),
        ],
        // M3's radio is a ring: the dot and the selected ring carry the accent.
        NodeKind::Radio => &[
            ("border_color", if a.bordercolor.is_none() { outline } else { None }),
            ("border_color_active", accent),
            ("mark_color_active", accent),
        ],
        NodeKind::Toggle => &[
            ("color_active", accent),
            ("border_color_active", accent),
            ("mark_color", outline),
            ("mark_color_active", mark),
        ],
        NodeKind::Slider => &[
            ("val_color", accent),
            ("val_color_hover", accent),
            ("handle_color", accent),
            ("handle_color_hover", accent),
        ],
        // A field's fill is state-keyed: `color` alone leaves an *empty* field
        // grey, and empty is the state a catalog screenshot actually shows.
        NodeKind::Input | NodeKind::Textarea => &[
            ("color_empty", a.bg),
            ("color_focus", a.bg),
            ("border_color_empty", outline),
            ("border_color_focus", accent),
        ],
        _ => &[],
    };
    let mut p = Vec::new();
    for (k, v) in roles {
        if let Some(c) = v {
            p.push(format!("{k}: {}", hex_rgba(*c)));
        }
    }
    p
}

/// A stated dp renders ~0.7% smaller here than on the reference: its 56dp row
/// measures 158px where this one measures 157 (2.82 px/dp against 2.80). Small
/// per element, but it accumulates — eight pixels down a screen of twelve
/// swatches — so explicit sizes are scaled to match. Sizes only: extending this
/// to padding and spacing costs two routes (38 under 9.0 drops to 36), so
/// makepad evidently resolves those on a different rounding path.
const DP_SCALE: f32 = 157.7 / 157.0;

fn emit_attrs(node: &UiNode, out: &mut String, depth: usize) {
    let ind = "    ".repeat(depth);
    let a = &node.attrs;

    if let Some(f) = flow(node.kind) {
        let _ = writeln!(out, "{ind}flow: {f}");
    }
    // Explicit alignment wins; otherwise a row centres its children vertically
    // (a widget and its label line up), matching how ArkUI lays a row out.
    // `align` is the DSL's cross-axis alignment — 0 start, 1 centre, 2 end. It
    // was parsed into `Attrs` and then never read by this backend, so `align: 1`
    // did nothing: the music player's album art sat left where the reference
    // centres it. Cross axis is horizontal in a column, vertical in a row.
    let cross = a.align.and_then(|v| match v {
        0 => Some(0.0),
        1 => Some(0.5),
        2 => Some(1.0),
        _ => None,
    });
    let is_row = node.kind == NodeKind::Row;
    let ax = a.alignx.or(if is_row { None } else { cross });
    let ay = a.aligny.or(if is_row { cross } else { None });
    if ax.is_some() || ay.is_some() {
        let x = ax.unwrap_or(0.0);
        let y = ay.unwrap_or(if is_row { 0.5 } else { 0.0 });
        let _ = writeln!(out, "{ind}align: Align{{x: {x}, y: {y}}}");
    } else if is_row {
        let _ = writeln!(out, "{ind}align: Align{{y: 0.5}}");
    }
    // makepad containers default to Fill on both axes, which makes a card
    // stretch far past its content. A container with no explicit size hugs its
    // content vertically (`Fit`) and fills horizontally — the ArkUI default.
    let container = flow(node.kind).is_some();
    match a.w {
        Some(w) => {
            let _ = writeln!(out, "{ind}width: {}", w * DP_SCALE);
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
        // A label's `h` is its Material *line height*, a typographic metric, not
        // a box. Pinning the walk to it cropped the font's real line box and
        // sliced descenders across the whole kit. `h: 0` still means hidden.
        Some(h) if node.kind == NodeKind::Text && h > 0.0 => {
            let _ = writeln!(out, "{ind}height: Fit");
        }
        Some(h) => {
            let _ = writeln!(out, "{ind}height: {}", h * DP_SCALE);
        }
        None if a.fillh == Some(1) => {
            let _ = writeln!(out, "{ind}height: Fill");
        }
        None if a.fith == Some(1) => {
            let _ = writeln!(out, "{ind}height: Fit");
        }
        // A scroll fills unless told otherwise — that is what makes it scroll.
        //
        // Fit is right for a card, which hugs its content. It is wrong for a
        // scroll: a Fit scroll is exactly as tall as what it holds, so there is
        // no viewport smaller than the content and nothing to scroll. It read
        // as "the screen is not scrollable", and it was not.
        //
        // Only visible once `page()` took the window height. Before that the
        // page was Fit too, so it overflowed the window and the *host's*
        // ScrollYView did the scrolling for every screen. Clamping the page
        // removed that overflow and left the kit's own scrolls, still Fit, with
        // nothing to do.
        None if node.kind == NodeKind::Scroll => {
            let _ = writeln!(out, "{ind}height: Fill");
        }
        None if container => {
            let _ = writeln!(out, "{ind}height: Fit");
        }
        None => {}
    }
    // Padding. Only the *scalar* form lands in this dialect: the per-side object
    // `{left: .., top: ..}` parses and silently resolves to nothing, as do
    // `Inset{..}` (declared in mod.draw, out of scope in a mounted body) and
    // makepad's own base-with-override `0{left: ..}`. All three were tried on
    // device. That no-op is why every control rendered hugging its label — a
    // button measured 46dp wide against the reference's 89dp.
    //
    // So when the two axes differ, one has to win. A node with a fixed height
    // cannot use vertical padding anyway, which makes the horizontal inset the
    // one that matters: it is what gives a button its 24dp sides. Anything else
    // keeps the (inert) object form rather than gaining a horizontal inset it
    // never asked for, which would indent every list row's text.
    if a.padx.is_some() || a.pady.is_some() {
        let px = a.padx.or(a.pad).unwrap_or(0.0);
        let py = a.pady.or(a.pad).unwrap_or(0.0);
        if (px - py).abs() < 0.01 || (a.h.is_some() && px > 0.0) {
            let _ = writeln!(out, "{ind}padding: {px}");
        } else {
            let _ = writeln!(
                out,
                "{ind}padding: {{left: {px}, right: {px}, top: {py}, bottom: {py}}}"
            );
        }
    } else if let Some(p) = a.pad {
        let _ = writeln!(out, "{ind}padding: {p}");
    } else if node.kind == NodeKind::Text {
        let _ = writeln!(out, "{ind}padding: 0");
    }
    let (mx, my) = (a.marginx.or(a.margin), a.marginy.or(a.margin));
    if mx.is_some() || my.is_some() {
        let (mx, my) = (mx.unwrap_or(0.0), my.unwrap_or(0.0));
        let _ = writeln!(
            out,
            "{ind}margin: {{left: {mx}, right: {mx}, top: {my}, bottom: {my}}}"
        );
    }
    if let Some(sp) = a.spacing {
        let _ = writeln!(out, "{ind}spacing: {sp}");
    }
    // bg / radius / border share one draw_bg block. A RoundedView defaults to a
    // transparent fill, so a border with no bg paints a clean outline (Material
    // "outlined" components); a border needs show_bg so the outline is painted.
    let roles = control_roles(node.kind, a);
    if a.bg.is_some()
        || a.bg2.is_some()
        || a.radius.is_some()
        || a.border.is_some()
        || a.elevation.is_some()
        || !roles.is_empty()
    {
        if a.bg.is_some() || a.border.is_some() || a.elevation.is_some() {
            let _ = writeln!(out, "{ind}show_bg: true");
        }
        let mut parts = Vec::new();
        if let Some(bg) = a.bg {
            parts.push(format!("color: {}", hex_rgba(bg)));
        }
        // A second stop turns the fill into a gradient (`color_2`).
        if let Some(bg2) = a.bg2 {
            parts.push(format!("color_2: {}", hex_rgba(bg2)));
            // `group: "gradh"` asks for the horizontal axis. Layering a
            // half-alpha horizontal gradient over a vertical one averages to a
            // true diagonal, which is what the reference draws and what this
            // shader cannot do in one pass.
            if a.group.as_deref() == Some("gradh") {
                parts.push("gradient_fill_horizontal: 1.0".to_string());
            }
        }
        if let Some(r) = a.radius {
            // makepad calls this `border_radius` — a bare `radius` silently
            // missed, so every corner fell back to its 2.5 default. It is also
            // *half* the corner it draws (`Sdf2d.box` works in `2. * r`), so a
            // Material dp is emitted halved, which also puts a full corner
            // exactly on the SDF's capsule limit instead of past it.
            let mut r = r;
            for side in [a.h, a.w].into_iter().flatten() {
                r = r.min(side * 0.5);
            }
            parts.push(format!("border_radius: {}", (r * 0.5).max(0.0)));
        }
        if let Some(b) = a.border {
            // Halved, like `border_radius` above: the shader draws the stroke to
            // both sides of the edge, so a 1dp border measured 6px against the
            // reference's 3px on the same outlined card.
            parts.push(format!("border_size: {}", b * 0.5));
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
        parts.extend(roles);
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
            "{ind}on_click: || {{ NAV(t: {target:?}) }}"
        );
    }
    if let Some(s) = a.size {
        // The DSL states type sizes in sp, as Material does; makepad's font_size
        // is in points. Measured against the reference the same string came out
        // ~1.33x too tall on every screen (toolbar title 66px vs 47, heading 45
        // vs 34, button label 39 vs 29) — 72/96 exactly.
        const SP_TO_PT: f32 = 0.75;
        let s = s * SP_TO_PT;
        // icon selects the theme's Font-Awesome face (monochrome icons);
        // else weight >= 500 selects the Medium (bold) face — M3's label / title /
        // emphasis weight; else just set the size (Regular). Each swaps the whole
        // text_style for the theme style at this size.
        // Name the host's Roboto, at the weight the type role asks for.
        //
        // The reference renders in Android's Roboto and makepad bundles IBM
        // Plex, so text could not match while the letterforms differed. `self:`
        // resolves against the `cargo_manifest_path` of the script_mod that
        // evaluates this — the host's, now that the body mounts on the main VM
        // rather than a `Splash` isolate (whose empty manifest path blanked
        // every label).
        //
        // The device ships Roboto as a *variable* font with no separate Medium —
        // `fonts.xml` maps weight 500 onto the same file via the `wght` axis — so
        // the emphasis weight comes from `FontMember.weight`, which makepad maps
        // to that axis. Without it every heading and label rendered regular.
        if a.icon != Some(1) {
            // 1.45, not makepad's 1.2: Android's TextView adds font padding that
            // this does not, so a card's two text rows measured 12dp each against
            // the reference's 16.5. Tested at 1.2 / 1.32 / 1.45 on verified
            // sweeps — all within 0.02 of each other on the mean, and 1.45 takes
            // `adaptive` from 10.2 to 9.6. It costs the toolbars a little
            // (topappbar 8.0 -> 8.7), which stays comfortably in range. 1.6 was
            // also tried and is marginally worse without moving `adaptive`.
            let w = a.weight.unwrap_or(400).max(1) as f32;
            let _ = writeln!(
                out,
                "{ind}draw_text.text_style: TextStyle{{ font_family: FontFamily{{ latin := FontMember{{res: crate_resource(\"self:resources/Roboto-Regular.ttf\") asc: -0.1 desc: 0.0 weight: {w}}} }} line_spacing: 1.45 font_size: {s} }}"
            );
        } else if a.icon == Some(1) {
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
        let hex = hex_rgba(c);
        let _ = writeln!(out, "{ind}draw_text.color: {hex}");
        // An interactive widget's label mixes toward the theme on
        // hover/press/focus/select, so setting only the base colour held for an
        // idle control and vanished the moment one was selected. The stated
        // colour is meant to be *the* colour.
        if has_text_states(node.kind) {
            for state in ["color_hover", "color_down", "color_focus", "color_active"] {
                let _ = writeln!(out, "{ind}draw_text.{state}: {hex}");
            }
            // A field's placeholder is a separate role again, and empty is the
            // state a catalog screenshot shows.
            if matches!(node.kind, NodeKind::Input | NodeKind::Textarea) {
                let _ = writeln!(out, "{ind}draw_text.color_empty: {hex}");
            }
        }
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
    let on = a.on.or_else(|| a.indeterminate.map(|i| (i != 0) as i32));
    if let Some(on) = on {
        if matches!(
            node.kind,
            NodeKind::Checkbox | NodeKind::Toggle | NodeKind::Radio
        ) {
            let _ = writeln!(out, "{ind}active: {}", on != 0);
        }
        // `RadioButton` is the one control with no live `active` field — it has
        // only `active()`/`set_active()`, so the line above is dropped and a
        // selected radio drew as unselected. Writing the shader instance does
        // not survive either; the animator rewrites it next frame. Its animator
        // *is* live, so move that block's default. `animator +:` is the merge
        // form makepad's own variants use; plain `animator:` is ignored.
        if node.kind == NodeKind::Radio && on != 0 {
            let _ = writeln!(out, "{ind}animator +: {{ active: {{ default: @on }} }}");
        }
    }
    // A determinate circular indicator: the spinner shader, held still with its
    // gap pinned so it draws a fixed arc rather than an animated one.
    if node.kind == NodeKind::Progress && a.group.as_deref() == Some("arc") {
        let frac = match (a.value, a.total) {
            (Some(v), Some(t)) if t > 0.0 => (v / t).clamp(0.0, 1.0),
            (Some(v), None) => (v / 100.0).clamp(0.0, 1.0),
            _ => 1.0,
        };
        let gap = (1.0 - frac).clamp(0.0, 0.98);
        let _ = writeln!(out, "{ind}draw_bg.rotation_speed: 0.0");
        let _ = writeln!(out, "{ind}draw_bg.max_gap_ratio: {gap}");
        let _ = writeln!(out, "{ind}draw_bg.min_gap_ratio: {gap}");
        let _ = writeln!(out, "{ind}draw_bg.stroke_width: 4.5");
    }
    if node.kind == NodeKind::Slider {
        // makepad's Slider prints its value through a nested `text_input`, so a
        // `draw_text.color` on the slider itself never reaches it. Material shows
        // no inline readout — the reference's screens carry a "Value: N" caption
        // from the DSL instead — so hide the widget's own.
        // Every state, not just the base: focusing the widget (which a drag
        // does) brought its "1.00" readout and a caret back on screen over the
        // drawn track.
        for f in ["", "_hover", "_focus", "_down", "_active", "_empty"] {
            let _ = writeln!(out, "{ind}text_input.draw_text.color{f}: #00000000");
            let _ = writeln!(out, "{ind}text_input.draw_bg.color{f}: #00000000");
        }
        let _ = writeln!(out, "{ind}text_input.draw_cursor.color: #00000000");
        let _ = writeln!(out, "{ind}text_input.draw_selection.color: #00000000");

        // The widget draws its own active/inactive split — `val_color` up to the
        // handle, `color` past it — which is exactly the Material track. Earlier
        // this was covered with drawn bars because upstream's default paints the
        // whole track alike; the real cause was that only `color` had been set,
        // leaving 20-odd sibling state colours (`color_2`, five `border_color*`,
        // the `val_*` and `handle_*` families) still on the bevelled theme. Pin
        // the family and the native split shows through, and it tracks the drag.
        let t = theme();
        let enabled = a.enabled != Some(0);
        // `accent: 0` is how the Material lowering asks for an invisible widget:
        // it draws the Expressive track itself and keeps this one for the drag.
        let (track, val) = if a.accent == Some(0) {
            (0, 0)
        } else if enabled {
            (t.secondary_container, a.accent.unwrap_or(t.primary))
        } else {
            (material::dim(t.on_surface, 0.12), material::dim(t.on_surface, 0.38))
        };
        for f in ["", "_hover", "_focus", "_drag", "_disabled"] {
            let _ = writeln!(out, "{ind}draw_bg.color{f}: {}", hex_rgba(track));
            let _ = writeln!(out, "{ind}draw_bg.color_2{f}: {}", hex_rgba(track));
            let _ = writeln!(out, "{ind}draw_bg.val_color{f}: {}", hex_rgba(val));
            let _ = writeln!(out, "{ind}draw_bg.handle_color{f}: {}", hex_rgba(val));
            // Material's track is flat; upstream's bevel reads as a stray outline.
            let _ = writeln!(out, "{ind}draw_bg.border_color{f}: #00000000");
            let _ = writeln!(out, "{ind}draw_bg.border_color_2{f}: #00000000");
        }
        let _ = writeln!(out, "{ind}draw_bg.border_size: 0.0");
    }
    if let Some(v) = a.value {
        if node.kind == NodeKind::Slider {
            // The kit expresses slider positions as a 0..1 fraction, so pin the
            // range unless the caller gave an explicit one -- which the Material
            // lowering does, to tag each slider with its own band.
            let lo = a.min.unwrap_or(0.0);
            let hi = a.max.unwrap_or_else(|| lo + a.total.unwrap_or(1.0));
            let _ = writeln!(out, "{ind}min: {lo}");
            let _ = writeln!(out, "{ind}max: {hi}");
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
        assert!(ui.contains("font_size: 15"), "sp is converted to points");
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
    fn shape_attrs_use_the_names_makepad_draws_from() {
        // The route sweep proves a screen *translates*; it cannot catch a
        // correctly-shaped block whose keys makepad does not read. `radius` was
        // one: makepad calls it `border_radius` and defaults it to 2.5, so every
        // corner in the kit silently rendered at 2.5dp on device.
        let ui = to_makepad_ui(&tree(
            r#"{t:"column", bg: 4294901760, radius: 28, border: 1, bordercolor: 4278190080}"#,
        ));
        assert!(ui.contains("border_radius: 14"), "halved, and named:\n{ui}");
        assert!(!ui.contains(" radius: "), "no bare `radius` key:\n{ui}");
        assert!(ui.contains("border_size: 0.5"), "{ui}");
    }

    #[test]
    fn a_radius_is_bounded_by_its_box() {
        // makepad's `border_radius` is half the corner it draws (`Sdf2d.box`
        // works in `2. * r`), so a Material dp is emitted halved and a full
        // corner lands exactly on the SDF's capsule limit.
        let ui = to_makepad_ui(&tree(r#"{t:"column", bg: 4294901760, radius: 20, h: 40}"#));
        assert!(ui.contains("border_radius: 10"), "{ui}");
        let ui = to_makepad_ui(&tree(r#"{t:"column", bg: 4294901760, radius: 12, h: 56}"#));
        assert!(ui.contains("border_radius: 6"), "{ui}");
    }

    #[test]
    fn a_control_carries_its_material_roles() {
        // A control's shader colours cannot come from a widget kit: `Splash`
        // mounts its body on an isolate VM that never receives one. They have to
        // travel with the node — and default from the scheme, because the
        // reference's screens carry no colour at all.
        let ui = to_makepad_ui(&tree(r#"{t:"slider"}"#));
        assert!(ui.contains("val_color: #6750a4ff"), "themed by default:\n{ui}");
        let ui = to_makepad_ui(&tree(r#"{t:"checkbox", enabled: 0}"#));
        assert!(ui.contains("color_active: #6750a461"), "dimmed:\n{ui}");
    }

    #[test]
    fn a_selected_radio_says_so_through_its_animator() {
        // `RadioButton` alone has no live `active` field, so the selected state
        // has to come from moving its animator's default — via `animator +:`,
        // the merge form; plain `animator:` is ignored.
        let ui = to_makepad_ui(&tree(r#"{t:"radio", on: 1}"#));
        assert!(ui.contains("animator +: { active: { default: @on } }"), "{ui}");
        let ui = to_makepad_ui(&tree(r#"{t:"radio"}"#));
        assert!(!ui.contains("animator +:"), "{ui}");
        let ui = to_makepad_ui(&tree(r#"{t:"checkbox", on: 1}"#));
        assert!(!ui.contains("animator +:"), "only the radio needs it:\n{ui}");
    }

    #[test]
    fn a_field_keeps_its_input_inside_the_chrome() {
        // A Material field is a label, a supporting or error line and an
        // affordance around an editable box. The native TextInput draws none of
        // that, so the chrome is composed — but must still hold a real Input.
        let ui = to_makepad_ui(&tree(
            r#"{t:"textfield", variant:"outlined", hint:"Email", error:"Not valid", text:"x"}"#,
        ));
        assert!(ui.contains("TextInput {"), "still editable:\n{ui}");
        assert!(ui.contains("Not valid") && ui.contains("Email"), "{ui}");
        let ui = to_makepad_ui(&tree(r#"{t:"input", placeholder:"plain"}"#));
        assert_eq!(ui.matches("TextInput {").count(), 1, "no chrome:\n{ui}");
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
    fn a_tappable_container_gets_a_non_capturing_target_over_it() {
        // A View ignores `on_click`, so a row carrying `tapto` must be wrapped in
        // an Overlay with a real widget on top or the tap is silently dropped.
        let ui = to_makepad_ui(&tree(
            r#"{t:"row", h: 56, tapto:"date_planner", c:[
                   {t:"text", text:"Date Planner", h: 20}
               ]}"#,
        ));
        assert!(ui.contains("flow: Overlay"), "needs an overlay wrapper:\n{ui}");
        // `SplashTap`, not a Button. Any Button captures the finger on
        // touch-down through `event.hits`, which starves the enclosing scroll --
        // a screen of full-width tappable rows would not scroll at all, and the
        // swipe that failed to scroll fired navigation on release. `SplashTap`
        // hit-tests itself and ignores a press that travelled. Asserting the
        // type is what keeps a Button from coming back.
        assert!(
            ui.contains("SplashTap {"),
            "the hit target must not capture the finger:\n{ui}"
        );
        assert!(
            !ui.contains("ButtonFlatter"),
            "a Button target starves the scroll:\n{ui}"
        );
        assert!(
            ui.contains(r#"on_click: || { NAV(t: "date_planner") }"#),
            "the target carries the handler:\n{ui}"
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
