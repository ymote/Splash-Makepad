//! Material 3 components, lowered to primitives.
//!
//! The reference catalog (Splash-Android, rendered with real
//! `com.google.android.material.*` views) states components **semantically** —
//! `{t:"button", variant:"filled"}`, `{t:"fab", variant:"large"}`,
//! `{t:"chip", variant:"filter"}` — and lets the renderer produce the widget.
//! This backend previously had no such vocabulary, so the catalog here hand-drew
//! look-alikes out of boxes and every variant of a family collapsed to one
//! appearance. Feeding it the reference's own screens dropped a third of the
//! nodes on the floor.
//!
//! [`lower`] is the missing half: one Material node in, a tree of primitives out,
//! which the normal emitter then walks. Going through primitives (rather than
//! building dialect strings here) is deliberate — the corner-radius, colour-role
//! and text-state handling in `emit_attrs` is where the Material accuracy lives,
//! and a second path would drift from it.

use splash_render::{Attrs, NodeKind, UiNode};

/// The M3 baseline colour roles, `0xAARRGGBB`.
#[derive(Clone, Copy)]
pub struct Roles {
    pub primary: u32,
    pub on_primary: u32,
    pub primary_container: u32,
    pub on_primary_container: u32,
    pub secondary: u32,
    pub on_secondary: u32,
    pub secondary_container: u32,
    pub on_secondary_container: u32,
    pub tertiary: u32,
    pub on_tertiary: u32,
    pub tertiary_container: u32,
    pub on_tertiary_container: u32,
    pub surface_variant: u32,
    pub error: u32,
    pub on_error: u32,
    pub error_container: u32,
    pub on_error_container: u32,
    pub surface: u32,
    pub on_surface: u32,
    pub on_surface_variant: u32,
    pub outline: u32,
    pub outline_variant: u32,
    pub surf_lowest: u32,
    pub surf_low: u32,
    pub surf_cont: u32,
    pub surf_high: u32,
    pub surf_highest: u32,
    pub inverse_surface: u32,
    pub inverse_on_surface: u32,
}

impl Roles {
    pub const fn light() -> Self {
        Self {
            primary: 0xFF6750A4,
            on_primary: 0xFFFFFFFF,
            primary_container: 0xFFEADDFF,
            on_primary_container: 0xFF21005D,
            secondary_container: 0xFFE8DEF8,
            on_secondary_container: 0xFF1D192B,
            tertiary_container: 0xFFFFD8E4,
            error: 0xFFB3261E,
            on_error: 0xFFFFFFFF,
            error_container: 0xFFF9DEDC,
            on_error_container: 0xFF410E0B,
            surface: 0xFFFEF7FF,
            on_surface: 0xFF1D1B20,
            on_surface_variant: 0xFF49454F,
            outline: 0xFF79747E,
            outline_variant: 0xFFCAC4D0,
            surf_lowest: 0xFFFFFFFF,
            surf_low: 0xFFF7F2FA,
            surf_cont: 0xFFF3EDF7,
            surf_high: 0xFFECE6F0,
            surf_highest: 0xFFE6E0E9,
            inverse_surface: 0xFF322F35,
            inverse_on_surface: 0xFFF5EFF7,
            secondary: 0xFF625B71,
            on_secondary: 0xFFFFFFFF,
            tertiary: 0xFF7D5260,
            on_tertiary: 0xFFFFFFFF,
            on_tertiary_container: 0xFF31111D,
            surface_variant: 0xFFE7E0EC,
        }
    }
    /// The scheme the reference catalog actually renders.
    ///
    /// That device runs Material You, so its roles are generated from the
    /// wallpaper rather than the M3 baseline — a blue family, not the baseline
    /// purple. Every value here is sampled from the reference's own screenshots
    /// (`color`, `elevation` and `button` screens), so the two catalogs can be
    /// compared directly instead of only structurally.
    pub const fn reference_dark() -> Self {
        Self {
            primary: 0xFF96CDF8,
            on_primary: 0xFF00344E,
            primary_container: 0xFF004B70,
            on_primary_container: 0xFFCAE6FF,
            secondary_container: 0xFF384956,
            on_secondary_container: 0xFFD3E5F5,
            tertiary_container: 0xFF4D4162,
            error: 0xFFFFB4AB,
            on_error: 0xFF690005,
            error_container: 0xFF93000A,
            on_error_container: 0xFFFFDAD6,
            surface: 0xFF101417,
            on_surface: 0xFFE1E3E5,
            on_surface_variant: 0xFFC1C7CE,
            outline: 0xFF8B9198,
            outline_variant: 0xFF41484D,
            surf_lowest: 0xFF0E1214,
            surf_low: 0xFF181C20,
            surf_cont: 0xFF1C2024,
            surf_high: 0xFF272B2F,
            surf_highest: 0xFF32363A,
            inverse_surface: 0xFFE1E3E5,
            inverse_on_surface: 0xFF2E3134,
            secondary: 0xFFB7C9D9,
            on_secondary: 0xFF22323F,
            tertiary: 0xFFCFC0E8,
            on_tertiary: 0xFF362B4B,
            on_tertiary_container: 0xFFEBDDFF,
            surface_variant: 0xFF41484D,
        }
    }

    pub const fn dark() -> Self {
        Self {
            primary: 0xFFD0BCFF,
            on_primary: 0xFF381E72,
            primary_container: 0xFF4F378B,
            on_primary_container: 0xFFEADDFF,
            secondary_container: 0xFF4A4458,
            on_secondary_container: 0xFFE8DEF8,
            tertiary_container: 0xFF633B48,
            error: 0xFFF2B8B5,
            on_error: 0xFF601410,
            error_container: 0xFF8C1D18,
            on_error_container: 0xFFF9DEDC,
            surface: 0xFF141218,
            on_surface: 0xFFE6E0E9,
            on_surface_variant: 0xFFCAC4D0,
            outline: 0xFF938F99,
            outline_variant: 0xFF49454F,
            surf_lowest: 0xFF0F0D13,
            surf_low: 0xFF1D1B20,
            surf_cont: 0xFF211F26,
            surf_high: 0xFF2B2930,
            surf_highest: 0xFF36343B,
            inverse_surface: 0xFFE6E0E9,
            inverse_on_surface: 0xFF322F35,
            secondary: 0xFFCCC2DC,
            on_secondary: 0xFF332D41,
            tertiary: 0xFFEFB8C8,
            on_tertiary: 0xFF492532,
            on_tertiary_container: 0xFFFFD8E4,
            surface_variant: 0xFF49454F,
        }
    }
}

/// A Material icon name → a glyph in the theme's icon font (Font Awesome).
/// The reference names its icons rather than spelling codepoints.
pub fn icon_glyph(name: &str) -> &'static str {
    match name {
        "add" => "\u{f067}",
        "home" => "\u{f015}",
        "search" => "\u{f002}",
        "music_note" => "\u{f001}",
        "album" => "\u{f51f}",
        "attach_money" => "\u{f155}",
        "bookmark" => "\u{f02e}",
        "call" => "\u{f095}",
        "chat" => "\u{f075}",
        "check" => "\u{f00c}",
        // f044 is pen-to-square (a pencil in a box); the reference draws the
        // plain angled pencil.
        "edit" => "\u{f303}",
        "favorite" => "\u{f004}",
        "inbox" => "\u{f01c}",
        "mail" => "\u{f0e0}",
        "notifications" => "\u{f0f3}",
        "person" => "\u{f007}",
        "place" => "\u{f3c5}",
        "play_arrow" => "\u{f04b}",
        "repeat" => "\u{f01e}",
        "settings" => "\u{f013}",
        "share" => "\u{f1e0}",
        "shopping_cart" => "\u{f07a}",
        "shuffle" => "\u{f074}",
        "skip_next" => "\u{f051}",
        "skip_previous" => "\u{f048}",
        "star" => "\u{f005}",
        "menu" => "\u{f0c9}",
        "more" => "\u{f142}",
        "format_bold" => "\u{f032}",
        "format_italic" => "\u{f033}",
        "format_underlined" => "\u{f0cd}",
        "format_align_center" => "\u{f037}",
        "drag_handle" => "\u{f7a4}",
        // Field affordances: the reveal eye and the error marker.
        "visibility" => "\u{f06e}",
        "error" => "\u{f06a}",
        _ => "\u{f111}", // a filled dot: visibly "an icon we do not have a glyph for"
    }
}

// ---- small builders --------------------------------------------------------

fn n(kind: NodeKind) -> UiNode {
    UiNode {
        kind,
        attrs: Attrs::default(),
        children: Vec::new(),
    }
}
fn col() -> UiNode {
    n(NodeKind::Column)
}
fn row() -> UiNode {
    n(NodeKind::Row)
}
fn label(s: &str, size: f32, color: u32) -> UiNode {
    let mut t = n(NodeKind::Text);
    t.attrs.text = Some(s.to_string());
    t.attrs.size = Some(size);
    t.attrs.color = Some(color);
    t
}
/// A Label Large (14sp / weight 500) — the type role every Material control uses.
fn label_lg(s: &str, color: u32) -> UiNode {
    let mut t = label(s, 14.0, color);
    t.attrs.weight = Some(500);
    t
}
fn glyph(name: &str, size: f32, color: u32) -> UiNode {
    let mut t = label(icon_glyph(name), size, color);
    t.attrs.icon = Some(1);
    t
}
/// A centred, content-sized box.
fn boxed(bg: Option<u32>, radius: f32) -> UiNode {
    let mut c = col();
    c.attrs.bg = bg;
    c.attrs.radius = Some(radius);
    c.attrs.alignx = Some(0.5);
    c.attrs.aligny = Some(0.5);
    c
}

/// Material's disabled treatment: 38% ink, 12% container. Alpha only, so a role
/// stays recognisably itself.
/// The visible span of a gradient stated for a *diagonal* fill, rendered on a
/// shader that only does vertical. A diagonal spreads its range across both
/// axes, so only part of it shows along y: the reference's tile varies 13 of the
/// gradient's full 41 on green, i.e. 32% — hence stops at 0.34 and 0.66.
fn grad_span(a: u32, b: u32) -> (u32, u32) {
    (mix_rgb(a, b, 0.34), mix_rgb(a, b, 0.66))
}

/// Linear blend between two ARGB colours.
fn mix_rgb(a: u32, b: u32, t: f32) -> u32 {
    let ch = |sh: u32| {
        let (x, y) = (((a >> sh) & 0xff) as f32, ((b >> sh) & 0xff) as f32);
        ((x + (y - x) * t).round() as u32).min(255) << sh
    };
    (a & 0xFF00_0000) | ch(16) | ch(8) | ch(0)
}

pub(crate) fn dim(argb: u32, alpha: f32) -> u32 {
    let a = ((argb >> 24) & 0xff) as f32 * alpha;
    (argb & 0x00FF_FFFF) | ((a as u32) << 24)
}

fn is_enabled(a: &Attrs) -> bool {
    a.enabled.unwrap_or(1) != 0
}

fn variant<'a>(a: &'a Attrs, dflt: &'a str) -> &'a str {
    a.variant.as_deref().unwrap_or(dflt)
}

/// Roughly how wide a lowered node draws, in dp.
///
/// Only needed because `flow` has to be packed here (see [`flow`]); it is an
/// estimate, not a measurement — the renderer does the real layout.
fn measure(node: &UiNode) -> f32 {
    let a = &node.attrs;
    if let Some(w) = a.w {
        return w;
    }
    let pad = a.padx.or(a.pad).unwrap_or(0.0) * 2.0;
    if node.kind == NodeKind::Text {
        // 0.66em is a serviceable average advance for the UI faces in use, and
        // erring wide is the safe direction: a row that wraps one item early
        // still reads correctly, one that wraps late runs off the screen.
        let len = a.text.as_deref().map(str::chars).map(Iterator::count).unwrap_or(0) as f32;
        return pad + len * a.size.unwrap_or(14.0) * 0.66;
    }
    let spacing = a.spacing.unwrap_or(0.0) * (node.children.len().saturating_sub(1)) as f32;
    let kids: f32 = match node.kind {
        // a column is as wide as its widest child; a row is the sum
        NodeKind::Column | NodeKind::Stack => node
            .children
            .iter()
            .map(measure)
            .fold(0.0, f32::max),
        _ => node.children.iter().map(measure).sum(),
    };
    pad + spacing + kids
}

/// The width `flow` packs into. The reference's screens are authored for a
/// phone; a host with a different viewport can say so.
static FLOW_WIDTH: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(340);

/// Set the width, in dp, that a wrapping row packs into.
pub fn set_flow_width(dp: f32) {
    FLOW_WIDTH.store(dp as u32, std::sync::atomic::Ordering::Relaxed);
}

/// A wrapping row, packed into fixed rows here.
///
/// The reference wraps every group of demo widgets, and its screens overflow the
/// viewport without it. makepad's own `Flow::RightWrap` is inert in this
/// revision — verified on device, with and without an explicit width, a
/// `RightWrap` row clips exactly like a plain `Right` one — so the packing is
/// done here instead, off an estimated child width.
fn flow(node: &UiNode, r: &Roles) -> UiNode {
    let limit = FLOW_WIDTH.load(std::sync::atomic::Ordering::Relaxed) as f32;
    let spacing = node.attrs.spacing.unwrap_or(8.0);
    let mut lines = col();
    lines.attrs.spacing = Some(spacing);
    let mut line = row();
    line.attrs.spacing = Some(spacing);
    line.attrs.aligny = Some(0.5);
    let mut used = 0.0f32;
    for kid in &node.children {
        // Inside a wrapping row every item hugs; that is what makes it wrap.
        let mut kid = kid.clone();
        if kid.attrs.fillw.is_none() {
            kid.attrs.fitw = Some(1);
        }
        let kid = &kid;
        // Measure what will actually be drawn, not the semantic node.
        let drawn = lower(kid, r).unwrap_or_else(|| kid.clone());
        let w = measure(&drawn);
        if used > 0.0 && used + spacing + w > limit {
            lines.children.push(std::mem::replace(&mut line, {
                let mut l = row();
                l.attrs.spacing = Some(spacing);
                l.attrs.aligny = Some(0.5);
                l
            }));
            used = 0.0;
        }
        used += if used > 0.0 { spacing + w } else { w };
        line.children.push(drawn);
    }
    if !line.children.is_empty() {
        lines.children.push(line);
    }
    // No trailing gap, at any size. Re-tested under the aligned layout — still much worse
    // (allcomponents 8.7 -> 15.6), so unlike the alignment itself this one does
    // not come good once the layout beneath it is right. On `allcomponents` the
    // reference's chips sit
    // 82px above the field where these sit 43. Adding 13dp after every flow was
    // tried and is far worse (allcomponents 12.0 -> 17.4, button 10.7 -> 15.4):
    // screens with several flows compound it. 4dp -- matching the ~12px each of
    // `button`'s sections actually loses -- is also worse (allcomponents 8.7 ->
    // 15.6, and `button` itself 11.4 -> 11.9). Whatever that gap is, it is not a
    // property of the wrapping row.
    lines
}

/// The reference spells parallel lists comma-separated (`icon:"home,search"`,
/// `badge:",,7,"` — an empty slot means "none"), alongside `;`-separated items.
fn list_at(src: Option<&str>, i: usize) -> Option<&str> {
    src.and_then(|s| s.split(',').nth(i))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn items_of(a: &Attrs) -> Vec<&str> {
    a.items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
        .collect()
}

// ---- the lowering ----------------------------------------------------------

/// One Material node → a primitive subtree, or `None` if `node` is not one.
pub fn lower(node: &UiNode, r: &Roles) -> Option<UiNode> {
    let a = &node.attrs;
    Some(match node.kind {
        NodeKind::Text => text_role(node, r)?,
        // Only a field that states chrome lowers. The composed result contains a
        // bare `Input` of its own, and without this guard that one would lower
        // again, forever.
        NodeKind::Input | NodeKind::Textarea
            if a.variant.is_some()
                || a.hint.is_some()
                || a.helper.is_some()
                || a.error.is_some() =>
        {
            text_field(node, r)
        }
        NodeKind::Button => button(node, r)?,
        NodeKind::Image if a.src.is_some() => shapeable_image(a, r),
        // An indicator states no size in the reference — Android's widget has an
        // intrinsic one. Here it collapsed to zero and the screens drew nothing,
        // so supply the M3 default and keep the animating widget.
        // A linear indicator is a track with a fill, not a circular one. The
        // `variant` was ignored, so "Linear - determinate" drew a spinner.
        NodeKind::Progress if variant(a, "circular") == "linear" => {
            let total = a.total.unwrap_or(100.0).max(1.0);
            let frac = if a.indeterminate.unwrap_or(0) != 0 {
                0.35
            } else {
                (a.value.unwrap_or(0.0) / total).clamp(0.0, 1.0)
            };
            // 354dp: the reference's linear bar spans x45-1034, the full content
            // width, where 340 left it 12dp short.
            const TRACK: f32 = 354.0;
            // M3 Expressive, like the slider: active | 4dp gap | inactive, and a
            // 4dp stop dot at the end when determinate. Measured off the
            // reference's own bar — active 643px, an 11px gap, inactive 325px,
            // then an 11px dot, totalling the 990px content width.
            const GAP: f32 = 4.0;
            const DOT: f32 = 4.0;
            let determinate = a.indeterminate.unwrap_or(0) == 0;
            let seg = |w: f32, c: u32| {
                let mut s = col();
                s.attrs.bg = Some(c);
                s.attrs.h = Some(4.0);
                s.attrs.radius = Some(2.0);
                s.attrs.w = Some(w.max(1.0));
                s
            };
            // The row is taller than the 4dp bar — the reference reserves space
            // below each one, which is why its sections sat further apart than
            // these did. 6, measured: 8 overshoots (progressindicator 9.9 -> 10.3
            // where 6 gives 9.6), so the reserve is ~2dp, not the ~4 the gap
            // below the bar suggested.
            let mut bar = row();
            bar.attrs.h = Some(6.0);
            bar.attrs.aligny = Some(0.0);
            bar.attrs.fillw = Some(1);
            let active = TRACK * frac;
            let tail = TRACK - active - GAP - if determinate { DOT } else { 0.0 };
            bar.children.push(seg(active, r.primary));
            bar.children.push(sl_gap(GAP));
            bar.children.push(seg(tail, r.secondary_container));
            if determinate {
                let mut dot = col();
                dot.attrs.bg = Some(r.primary);
                dot.attrs.w = Some(DOT);
                dot.attrs.h = Some(DOT);
                dot.attrs.radius = Some(DOT * 0.5);
                bar.children.push(dot);
            }
            bar
        }
        // A determinate circular indicator is a 40dp ring in the reference
        // (measured). Upstream's LoadingSpinner sizes its arc off its own theme
        // and drew a sliver beside it, the same way Toggle ignores its rect.
        NodeKind::Progress
            if variant(a, "circular") == "circular"
                && a.value.is_some()
                && a.indeterminate.unwrap_or(0) == 0
                && a.group.is_none() =>
        {
            let mut ring = col();
            ring.attrs.w = Some(40.0);
            ring.attrs.h = Some(40.0);
            ring.attrs.radius = Some(20.0);
            // 3.2 nominal, which the emitter halves to the 1.6 that measured
            // right — that halving was added after this constant was set, and
            // silently thinned the ring to 0.8dp.
            ring.attrs.border = Some(3.2);
            // The ring is the *track*; the progress arc goes over it.
            ring.attrs.bordercolor = Some(r.secondary_container);
            // A partial arc is not expressible with box primitives, but the
            // spinner shader already draws one — stop its rotation and pin its
            // gap to 1 - progress and it becomes a determinate indicator.
            let mut arc = node.clone();
            arc.attrs.group = Some("arc".to_string());
            arc.attrs.w = Some(40.0);
            arc.attrs.h = Some(40.0);
            arc.attrs.bg = Some(a.bg.unwrap_or(r.primary));
            // The row reserves space below the 40dp ring, like the slider and
            // linear bar do: the reference's next section sits 21px further down
            // than a bare 40dp row leaves room for. 53, measured in two steps:
            // 48 closed most of it and left 15px, which is this last 5dp.
            let mut stack = n(NodeKind::Stack);
            stack.attrs.w = Some(40.0);
            stack.attrs.h = Some(53.0);
            stack.attrs.aligny = Some(0.0);
            stack.children.push(ring);
            stack.children.push(arc);
            stack
        }
        NodeKind::Loading | NodeKind::Progress if a.w.is_none() && a.h.is_none() => {
            let mut ind = node.clone();
            let d = if variant(a, "uncontained") == "contained" { 48.0 } else { 40.0 };
            ind.attrs.w = Some(d);
            ind.attrs.h = Some(d);
            ind.attrs.bg = a.bg.or(Some(r.primary));
            ind
        }
        // A slider states its range in the reference; that is the cue to dress it.
        // `accent: 0` marks the invisible native widget this lowering already
        // produced -- the same marker the rest of the module uses. Clearing
        // min/max used to serve as the guard, which meant the native slider
        // could never state a range of its own; it needs one now to carry its
        // index, so the guard has to be the marker rather than a side effect.
        NodeKind::Slider if a.accent != Some(0) && (a.min.is_some() || a.max.is_some()) => {
            slider(node, r)
        }
        NodeKind::Fab => fab(a, r),
        NodeKind::IconButton => icon_button(a, r),
        NodeKind::Flow => flow(node, r),
        NodeKind::Segmented => segmented(a, r),
        NodeKind::Toggle if a.text.is_some() => switch_row(node, r),
        NodeKind::Chip => chip(a, r),
        // A checkbox had no lowering at all, so nothing ever set its tap: it
        // toggled makepad's own visual state and reverted on the next mount,
        // while the DSL's `on:` -- and every caption reading that slot -- stayed
        // put. `CheckBox` does carry an `on_click`, so the tap can sit on the
        // widget itself. The `tapto` guard is what stops this re-entering.
        NodeKind::Checkbox if a.key.is_some() && a.tapto.is_none() => {
            let mut c = node.clone();
            let on = a.on.unwrap_or(0) != 0;
            let k = a.key.as_deref().unwrap_or_default();
            c.attrs.tapto = Some(format!("set:{k}={}", i32::from(!on)));
            c
        }
        NodeKind::Card => card(node, r),
        NodeKind::ListItem => list_item(a, r),
        NodeKind::Divider => divider(a, r),
        NodeKind::Spacer => spacer(a),
        NodeKind::ShapeBox => shape_box(a, r),
        NodeKind::ColorSwatch => color_swatch(a, r),
        // The 11px before a section header is stated on the header now, not as
        // a trailing gap on a group of swatches. They are the same boundary, so
        // only one may state it; `color` is the screen that had both and it
        // drifted 11px per section (4.1 -> 19.5). The swatch-gap arm is deleted
        // rather than made to return the node unchanged -- its guard would still
        // match on re-entry and run away (color 74.8).
        //
        // Mark every section header *except the first*. That exception is the
        // whole difference: a trailing gap falls after the last group, where
        // nothing follows it, but a leading one falls before the first header,
        // where it shifts the entire screen down 11px. On `color`, five
        // sections of uniform 162px bands, that shift alone costs 8 points.
        NodeKind::Column
            if node.children.iter().skip(1).any(|k| {
                is_section_head(k) && k.attrs.group.as_deref() != Some("sechead")
            }) =>
        {
            let mut g = node.clone();
            for k in g.children.iter_mut().skip(1) {
                if is_section_head(k) {
                    k.attrs.group = Some("sechead".to_string());
                }
            }
            g
        }
        NodeKind::BadgeIcon => badge_icon(a, r),
        NodeKind::SearchBar => search_bar(a, r),
        NodeKind::Dropdown => dropdown(a, r),
        NodeKind::Tabs => tabs(a, r),
        NodeKind::NavBar => nav_bar(a, r),
        NodeKind::NavRail => nav_rail(a, r),
        NodeKind::Carousel => carousel(a, r),
        NodeKind::RadioGroup => radio_group(a, r),
        NodeKind::RangeSlider => range_slider(a, r),
        NodeKind::AppBarDemo => app_bar(a, r),
        NodeKind::BottomBarDemo => bottom_bar(r),
        NodeKind::ToolbarDemo => toolbar(a, r),
        NodeKind::AdaptiveDemo => adaptive(a, r),
        NodeKind::TransitionHost => transition_host(a, r),
        NodeKind::WeatherIcon => weather_icon(a, r),
        NodeKind::NavMap => nav_map(a, r),
        NodeKind::GlassPanel => glass_panel(node, r),
        _ => return None,
    })
}

/// The Material type scale: `(size, line-height, weight)`.
///
/// The reference sets a type role rather than a size — `variant:"titleMedium"`,
/// `"bodySmall"`, `"displayLarge"` — so without this every heading, caption and
/// sample on a screen renders at the same default size, and the `font` screen
/// (whose whole subject is the scale) shows fifteen identical lines.
fn type_role(name: &str) -> Option<(f32, i32)> {
    Some(match name {
        "displayLarge" => (57.0, 400),
        "displayMedium" => (45.0, 400),
        "displaySmall" => (36.0, 400),
        "headlineLarge" => (32.0, 400),
        "headlineMedium" => (28.0, 400),
        "headlineSmall" => (24.0, 400),
        "titleLarge" => (22.0, 400),
        "titleMedium" => (16.0, 500),
        "titleSmall" => (14.0, 500),
        "bodyLarge" => (16.0, 400),
        "bodyMedium" => (14.0, 400),
        "bodySmall" => (12.0, 400),
        "labelLarge" => (14.0, 500),
        "labelMedium" => (12.0, 500),
        "labelSmall" => (11.0, 500),
        _ => return None,
    })
}

/// A text node carrying a type role, resolved against the scale.
fn text_role(node: &UiNode, r: &Roles) -> Option<UiNode> {
    let a = &node.attrs;
    let (size, weight) = type_role(a.variant.as_deref()?)?;
    let mut t = node.clone();
    // An explicit size still wins; the role only supplies what was not said.
    t.attrs.size = a.size.or(Some(size));
    t.attrs.weight = a.weight.or(Some(weight));
    t.attrs.color = a.color.or(Some(match a.variant.as_deref() {
        // Captions and supporting lines sit on the variant ink, headings on the
        // main one — that is how the reference reads on screen.
        Some("bodySmall") | Some("labelSmall") | Some("bodyMedium") => r.on_surface_variant,
        _ => r.on_surface,
    }));
    t.attrs.fillw = a.fillw.or(Some(1));
    t.attrs.variant = None; // resolved; do not re-enter
    Some(t)
}

/// The five M3 button variants. A bare `{t:"button"}` with no variant keeps the
/// native widget (the kit uses it for real taps), so only a *stated* variant
/// lowers — otherwise this would silently take over every button in the kit.
fn button(node: &UiNode, r: &Roles) -> Option<UiNode> {
    let a = &node.attrs;
    let v = a.variant.as_deref()?;
    let on = is_enabled(a);
    let (bg, ink, border) = match v {
        "filled" => (Some(r.primary), r.on_primary, None),
        "tonal" => (Some(r.secondary_container), r.on_secondary_container, None),
        "elevated" => (Some(r.surf_low), r.primary, None),
        "outlined" => (None, r.primary, Some(r.outline)),
        "text" => (None, r.primary, None),
        _ => (Some(r.primary), r.on_primary, None),
    };
    let (bg, ink, border) = if on {
        (bg, ink, border)
    } else {
        (
            bg.map(|c| dim(r.on_surface, 0.12) | (c & 0)),
            dim(r.on_surface, 0.38),
            border.map(|_| dim(r.on_surface, 0.12)),
        )
    };

    let mut b = boxed(bg, 20.0);
    b.attrs.h = Some(40.0);
    // Horizontal inset is carried by spacer children, not `padx`: only the
    // *symmetric scalar* padding form lands in this dialect, and on a
    // fixed-height box 21.5dp of it leaves a negative content area, which
    // dropped every label to the bottom edge of its button.
    //
    // The spacer is short of the target inset by the Label's own default
    // padding, which sits inside it — the same correction the vertical wrapper
    // makes. Measured: a 21.5 spacer rendered a 94dp Filled against the
    // reference's 89. (The overshoot is not font width; a glyph run measures
    // 343px here against the reference's 347.)
    let inset = if v == "text" { 16.0 } else { 26.0 };
    // Fill by default: the reference's trigger stacks are full-width bars,
    // because a MaterialButton in a vertical LinearLayout is match_parent. A
    // button inside a `flow` hugs instead — `flow` marks its children.
    if a.fitw == Some(1) {
        b.attrs.fitw = Some(1);
    } else {
        b.attrs.fillw = Some(1);
    }
    b.attrs.spacing = Some(8.0);
    if let Some(bc) = border {
        b.attrs.border = Some(1.0);
        b.attrs.bordercolor = Some(bc);
    }
    if v == "elevated" && on {
        b.attrs.elevation = Some(1.0);
    }
    // `key` + `action` is the reference's way of asking the host to open a real
    // dialog / sheet / picker. Route it through the same tap channel.
    if on {
        if let Some(k) = a.key.as_deref() {
            // `action` names a host behaviour (open this dialog); a bare `tap: 1`
            // just records the press. Requiring both left every plain button on
            // the `button` screen inert — they carry `key` + `tap`, no `action`.
            match a.action.as_deref() {
                Some(act) => b.attrs.tapto = Some(format!("set:{k}={act}")),
                None if a.tap.unwrap_or(0) != 0 => {
                    b.attrs.tapto = Some(format!("set:{k}=1"))
                }
                None => {}
            }
        }
    }
    let text = a.label.clone().or_else(|| a.text.clone()).unwrap_or_default();
    // Spacing is stated per gap, not as a row `spacing`, so the leading and
    // trailing insets stay exactly `inset` rather than inset + spacing.
    let mut inner = row();
    inner.attrs.spacing = Some(0.0);
    inner.attrs.aligny = Some(0.5);
    inner.attrs.fitw = Some(1);
    // M3 tightens the leading inset to 16dp when a label follows an icon. With
    // 21.5 on both sides the three icon buttons overflowed the row and clipped
    // "Save" off the screen edge, where the reference's three fit.
    let icon = a.icon_name.as_deref();
    inner.children.push(sl_gap(if icon.is_some() { 28.0 } else { inset }));
    if let Some(icon) = icon {
        inner.children.push(glyph(icon, 18.0, ink));
    }
    if !text.is_empty() {
        if icon.is_some() {
            inner.children.push(sl_gap(8.0));
        }
        inner.children.push(label_lg(&text, ink));
    }
    inner.children.push(sl_gap(inset));
    b.children.push(inner);
    // Android's MaterialButton carries a 4dp vertical inset by default, which
    // the reference inherits: its stacked triggers sit 16dp apart where the DSL
    // asks for 8. Stated as spacer rows around the button rather than a margin,
    // which this dialect drops; the button's own 40dp height is unchanged.
    let mut outer = col();
    outer.attrs.fillw = if a.fitw == Some(1) { None } else { Some(1) };
    if a.fitw == Some(1) {
        outer.attrs.fitw = Some(1);
    }
    // 4dp. The `button` screen loses ~12px per button row against the reference,
    // but this is not where it comes from: 6 makes things far worse (excess over
    // 9.0 across the sweep 2.3 -> 14.7) because `dialog`'s six stacked buttons
    // compound it, and dialog's button pitch already matches exactly (158).
    let mut pad_top = col();
    pad_top.attrs.h = Some(4.0);
    let mut pad_bot = col();
    pad_bot.attrs.h = Some(4.0);
    // No horizontal counterpart to these spacers, though the reference's
    // three-button rows do measure 282dp against 268 here. Even 2dp a side tips
    // the `button` screen's first row past makepad's wrap threshold and drops
    // its third button to the next line, where the reference keeps three — a
    // worse error than the 14dp of width it buys.
    outer.children.push(pad_top);
    outer.children.push(b);
    outer.children.push(pad_bot);
    Some(outer)
}

fn fab(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "regular");
    let icon = a.icon_name.as_deref().unwrap_or("add");
    // M3 FAB: 40/56/96dp with 12/16/28dp corners; extended is 56dp and hugs.
    let (size, radius, glyph_size) = match v {
        "small" => (40.0, 12.0, 20.0),
        "large" => (96.0, 28.0, 36.0),
        _ => (56.0, 16.0, 24.0),
    };
    // `group` names the container role; the reference's "Colours" row is four
    // FABs that differ only by it, and ignoring it made them all identical.
    let (fill, ink) = match a.group.as_deref() {
        Some("secondary") => (r.secondary_container, r.on_secondary_container),
        Some("tertiary") => (r.tertiary_container, r.on_tertiary_container),
        Some("surface") => (r.surf_high, r.primary),
        _ => (r.primary_container, r.on_primary_container),
    };
    let mut f = boxed(Some(fill), radius);
    f.attrs.h = Some(if v == "extended" { 56.0 } else { size });
    f.attrs.elevation = Some(3.0);
    if v == "extended" {
        f.attrs.padx = Some(16.0);
        f.attrs.fitw = Some(1);
        f.attrs.spacing = Some(12.0);
        let mut inner = row();
        inner.attrs.spacing = Some(12.0);
        inner.attrs.aligny = Some(0.5);
        inner.attrs.fitw = Some(1);
        inner.children.push(glyph(icon, 24.0, ink));
        if let Some(l) = a.label.as_deref() {
            let mut t = label(l, 16.0, ink);
            t.attrs.weight = Some(500);
            inner.children.push(t);
        }
        f.children.push(inner);
    } else {
        f.attrs.w = Some(size);
        f.children
            .push(glyph(icon, glyph_size, r.on_primary_container));
    }
    f
}

/// A section header, as `screens/kit.splash` spells one: the only Text node in
/// any of the 41 screens carrying `marginy: 4`.
fn is_section_head(n: &UiNode) -> bool {
    n.kind == NodeKind::Text && n.attrs.marginy == Some(4.0)
}

fn icon_button(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "standard");
    let selected = a.on.unwrap_or(0) != 0;
    let (bg, ink, border) = match v {
        "filled" => (Some(r.primary), r.on_primary, None),
        "tonal" => (
            Some(r.secondary_container),
            r.on_secondary_container,
            None,
        ),
        // The reference inks an outlined icon button with primary, not the
        // neutral it uses for a standard one (sampled off its gear: #96CDF8).
        "outlined" => (None, r.primary, Some(r.outline)),
        _ => (None, r.on_surface_variant, None),
    };
    // A checkable standard icon button fills with primary when selected.
    let ink = if selected && v == "standard" {
        r.primary
    } else {
        ink
    };
    let mut b = boxed(bg, 20.0);
    b.attrs.w = Some(40.0);
    b.attrs.h = Some(40.0);
    // Android wires a checkable icon button to `onState(key, checked?1:0)` and a
    // plain one to its click. `checkable` is never parsed into `Attrs`, so the
    // presence of `on` is what separates the two -- which is the same thing the
    // DSL means by it.
    if let Some(k) = a.key.as_deref() {
        b.attrs.tapto = Some(if a.on.is_some() {
            format!("set:{k}={}", i32::from(!selected))
        } else {
            format!("set:{k}=1")
        });
    }
    if let Some(bc) = border {
        b.attrs.border = Some(1.0);
        b.attrs.bordercolor = Some(bc);
    }
    // A standard icon button has no container, so its glyph *is* its width:
    // the reference's measures 42px against 55 at 20dp, hence 15. The other
    // variants sit in a 40dp circle that fixes their width either way.
    let glyph_dp = if v == "standard" { 15.0 } else { 20.0 };
    b.children.push(glyph(
        a.icon_name.as_deref().unwrap_or("add"),
        glyph_dp,
        ink,
    ));
    // Android's icon buttons carry the same 4dp inset its MaterialButton does:
    // the reference's sit 56dp apart where the DSL asks for 8dp of spacing on a
    // 40dp control. Stated as side gaps, since margin is dropped by this dialect.
    let mut outer = row();
    outer.attrs.fitw = Some(1);
    outer.attrs.aligny = Some(0.5);
    outer.children.push(sl_gap(4.0));
    outer.children.push(b);
    outer.children.push(sl_gap(4.0));
    // The inset is vertical too, and only the sides of it were being stated.
    // Measured on `button`: the reference's icon-button row starts 13px below
    // its header where a *button* row starts in the right place, and the next
    // header sits 22px further down still -- 4dp above and below, exactly the
    // pair MaterialButton gets.
    let mut vouter = col();
    vouter.attrs.fitw = Some(1);
    let mut t = col();
    t.attrs.h = Some(4.0);
    let mut bm = col();
    bm.attrs.h = Some(4.0);
    vouter.children.push(t);
    vouter.children.push(outer);
    vouter.children.push(bm);
    vouter
}
fn segmented(a: &Attrs, r: &Roles) -> UiNode {
    let sel = a.selected.unwrap_or(0);
    let mut strip = row();
    strip.attrs.h = Some(40.0);
    strip.attrs.radius = Some(20.0);
    strip.attrs.border = Some(1.0);
    strip.attrs.bordercolor = Some(r.outline);
    // A segmented button spans its row and divides evenly; hugging its content
    // made "Default|Pink|Green|Blue" a small clump against the reference's bar.
    strip.attrs.fillw = Some(1);
    let count = a
        .items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
        .count();
    for (i, item) in a
        .items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let on = i as i32 == sel;
        // Square, though the reference's selected end segment fills the
        // container's rounded corner and this leaves grey slivers there. The SDF
        // takes one radius for all four corners (`box_y` splits top/bottom, not
        // left/right), so rounding the fill also rounds its inner edge and loses
        // the divider — measured as the worse of the two.
        let _ = count;
        let mut seg = boxed(if on { Some(r.secondary_container) } else { None }, 0.0);
        // The tap writes the state slot; the DSL is re-evaluated and the strip
        // redraws with the new selection.
        if let Some(k) = a.key.as_deref() {
            seg.attrs.tapto = Some(format!("set:{k}={i}"));
        }
        seg.attrs.h = Some(40.0);
        seg.attrs.padx = Some(12.0);
        seg.attrs.fillw = Some(1);
        seg.attrs.spacing = Some(8.0);
        let ink = if on {
            r.on_secondary_container
        } else {
            r.on_surface_variant
        };
        let mut inner = row();
        inner.attrs.spacing = Some(8.0);
        inner.attrs.aligny = Some(0.5);
        inner.attrs.fitw = Some(1);
        // No leading check: M3 specs one, but the reference's toggle group
        // renders text only, and the extra glyph widened the selected segment.
        inner.children.push(label_lg(item, ink));
        seg.children.push(inner);
        strip.children.push(seg);
        // The joined look: a hairline between segments, not a gap.
        if i > 0 {
            let mut rule = col();
            rule.attrs.w = Some(1.0);
            rule.attrs.bg = Some(r.outline);
            strip.children.insert(strip.children.len() - 1, rule);
        }
    }
    // Same 4dp vertical inset as a button and an icon button: the reference's
    // strip starts 13px below its header, and the header after it sits 22px
    // lower again.
    let mut vouter = col();
    let mut t = col();
    t.attrs.h = Some(4.0);
    let mut bm = col();
    bm.attrs.h = Some(4.0);
    vouter.children.push(t);
    vouter.children.push(strip);
    vouter.children.push(bm);
    vouter
}

fn chip(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "assist");
    let on = a.on.unwrap_or(0) != 0;
    let enabled = is_enabled(a);
    let (bg, ink, border) = if on {
        (
            Some(r.secondary_container),
            r.on_secondary_container,
            None,
        )
    } else {
        (None, r.on_surface_variant, Some(r.outline_variant))
    };
    let (ink, border) = if enabled {
        (ink, border)
    } else {
        (dim(r.on_surface, 0.38), border.map(|_| dim(r.on_surface, 0.12)))
    };
    let mut c = boxed(bg, 8.0);
    if let Some(k) = a.key.as_deref() {
        c.attrs.tapto = Some(format!("set:{k}={}", if on { 0 } else { 1 }));
    }
    c.attrs.h = Some(32.0);
    c.attrs.fitw = Some(1);
    if let Some(bc) = border {
        c.attrs.border = Some(1.0);
        c.attrs.bordercolor = Some(bc);
    }
    // 13dp a side, measured: the reference's chips are ~28px wider on both the
    // `chip` screen (253 against 225) and `allcomponents` (226 against 197).
    // The inset rides on spacer children, not `padx` — see `button`: the
    // only padding form this dialect honours is the symmetric scalar, which on
    // a fixed-height chip adds 12dp top and bottom and swells it past 32dp.
    let mut inner = row();
    inner.attrs.spacing = Some(0.0);
    inner.attrs.aligny = Some(0.5);
    inner.attrs.fitw = Some(1);
    inner.children.push(sl_gap(13.0));
    if on {
        inner.children.push(glyph("check", 16.0, ink));
        inner.children.push(sl_gap(8.0));
    }
    if let Some(icon) = a.icon_name.as_deref() {
        inner.children.push(glyph(icon, 16.0, ink));
        inner.children.push(sl_gap(8.0));
    }
    inner.children.push(label_lg(
        a.label.as_deref().or(a.text.as_deref()).unwrap_or(""),
        ink,
    ));
    // An input chip carries a trailing dismiss affordance.
    if v == "input" {
        inner.children.push(sl_gap(8.0));
        inner.children.push(label("\u{00d7}", 14.0, ink));
    }
    inner.children.push(sl_gap(13.0));
    c.children.push(inner);
    // Android's chip carries vertical margins the DSL does not state: the
    // reference leaves 23px above a chip row and 33px below it (measured on the
    // `chip` screen), making the row 52dp around a 32dp chip. That 56px is also
    // exactly the gap `allcomponents` was missing between its chips and field.
    let mut outer = col();
    outer.attrs.fitw = Some(1);
    // Bottom only. The reference also leaves 23px *above* a chip row, but these
    // chips already sit ~13px low on `allcomponents`, so a top margin compounds
    // there (11.9 -> 14.2) while the bottom margin alone helps both.
    let mut bot = col();
    bot.attrs.h = Some(12.0);
    outer.children.push(c);
    outer.children.push(bot);
    outer
}

fn card(node: &UiNode, r: &Roles) -> UiNode {
    let a = &node.attrs;
    let v = variant(a, "elevated");
    let mut c = col();
    c.attrs.radius = Some(12.0);
    // Only pad when the card supplies its own content. The screens wrap a card
    // around a `col` that already carries `pad: 16`, so adding it here too
    // doubled the inset and made every card ~32dp taller than the reference's.
    if node.children.is_empty() {
        c.attrs.pad = Some(16.0);
    }
    c.attrs.spacing = Some(8.0);
    match v {
        "filled" => c.attrs.bg = Some(r.surf_highest),
        "outlined" => {
            c.attrs.bg = Some(r.surface);
            c.attrs.border = Some(1.0);
            c.attrs.bordercolor = Some(r.outline_variant);
        }
        _ => {
            c.attrs.bg = Some(r.surf_low);
            c.attrs.elevation = Some(1.0);
        }
    }
    if let Some(t) = a.title.as_deref().or(a.label.as_deref()) {
        let mut h = label(t, 16.0, r.on_surface);
        h.attrs.weight = Some(500);
        c.children.push(h);
    }
    if let Some(s) = a.supporting.as_deref().or(a.text.as_deref()) {
        let mut body = label(s, 14.0, r.on_surface_variant);
        body.attrs.fillw = Some(1);
        c.children.push(body);
    }
    c.children.extend(node.children.iter().cloned());
    c
}

fn list_item(a: &Attrs, r: &Roles) -> UiNode {
    let title = a.label.as_deref().or(a.text.as_deref()).unwrap_or("");
    let supporting = a.supporting.as_deref();
    // M3 list rows: 56dp one-line, 72dp two-line, 88dp three-line.
    let lines = a.lines.unwrap_or(if supporting.is_some() { 2 } else { 1 });
    let h = match lines {
        1 => 56.0,
        2 => 72.0,
        _ => 88.0,
    };
    let mut item = row();
    item.attrs.h = Some(h);
    item.attrs.spacing = Some(16.0);
    item.attrs.aligny = Some(0.5);
    if let Some(icon) = a.icon_name.as_deref() {
        // The reference's leading affordance is a small outlined glyph, not the
        // filled 40dp avatar this used to draw.
        let mut lead = boxed(None, 12.0);
        lead.attrs.w = Some(24.0);
        lead.attrs.h = Some(24.0);
        lead.children.push(glyph(icon, 20.0, r.on_surface_variant));
        item.children.push(lead);
    }
    let mut stack = col();
    stack.attrs.spacing = Some(2.0);
    stack.attrs.fillw = Some(1);
    let mut head = label(title, 16.0, r.on_surface);
    head.attrs.fillw = Some(1);
    stack.children.push(head);
    if let Some(s) = supporting {
        let mut body = label(s, 14.0, r.on_surface_variant);
        body.attrs.fillw = Some(1);
        stack.children.push(body);
    }
    item.children.push(stack);
    // `action` names a trailing control; the preferences screen is a column of
    // list rows whose switches were simply absent.
    match a.action.as_deref() {
        Some("switch") => {
            // No Fill spacer here: the text stack already fills, and two Fill
            // children split the row — which squeezed the supporting line onto
            // two rows. The control hugs at the trailing edge instead.
            let mut sw = n(NodeKind::Toggle);
            sw.attrs.on = a.on;
            sw.attrs.accent = Some(r.primary);
            sw.attrs.markcolor = Some(r.on_primary);
            sw.attrs.bordercolor = Some(r.outline);
            sw.attrs.bg = Some(r.surf_highest);
            sw.attrs.fitw = Some(1);
            item.children.push(sw);
        }
        Some("more") => {
            item.children.push(glyph("more", 20.0, r.on_surface_variant));
        }
        _ => {}
    }
    item
}

fn divider(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "full");
    let mut d = col();
    d.attrs.bg = Some(r.outline_variant);
    if v == "vertical" {
        d.attrs.w = Some(1.0);
        d.attrs.h = a.h.or(Some(32.0));
    } else {
        d.attrs.h = Some(1.0);
        if v == "inset" {
            d.attrs.margin = Some(16.0);
        }
    }
    d
}

fn spacer(a: &Attrs) -> UiNode {
    let mut s = col();
    s.attrs.h = a.h.or(Some(8.0));
    s.attrs.w = a.w;
    // A bare spacer between two items is the DSL's way of pushing them apart —
    // the music player's "1:12 … 3:24" row spans 232dp here against the
    // reference's 309 because a zero-width spacer moved nothing.
    if a.w.is_none() {
        s.attrs.fillw = Some(1);
    }
    s
}

/// A shape-scale sample: a filled card carrying its own name, at the stated
/// corner. The reference builds a `MaterialCardView` with a Title Small label
/// and 20dp padding — a full-width band, not a swatch, which is why a row of
/// squares looked nothing like it.
fn shape_box(a: &Attrs, r: &Roles) -> UiNode {
    let mut card = col();
    card.attrs.bg = Some(r.surf_highest);
    card.attrs.fillw = Some(1);
    // The reference's rows measure 59dp tall. This has been re-derived each time
    // the text metrics moved: 16 and 18 while the Label carried its own inset,
    // 22 once that was zeroed, and 20 now that text nodes get a 2dp inset back.
    card.attrs.pad = Some(20.0);
    // `radius: 999` is the reference's "full"; it resolves that to 2 x 28dp.
    let radius = a.radius.unwrap_or(0.0);
    card.attrs.radius = Some(if radius >= 999.0 { 56.0 } else { radius });
    if let Some(t) = a.text.as_deref().or(a.label.as_deref()) {
        let mut l = label(t, 14.0, r.on_surface);
        l.attrs.weight = Some(500);
        l.attrs.fillw = Some(1);
        card.children.push(l);
    }
    card
}

/// The Material role a name refers to, and the ink that reads on it.
fn role_pair(r: &Roles, name: &str) -> (u32, u32) {
    match name {
        "primary" => (r.primary, r.on_primary),
        "onPrimary" => (r.on_primary, r.primary),
        "primaryContainer" => (r.primary_container, r.on_primary_container),
        "onPrimaryContainer" => (r.on_primary_container, r.primary_container),
        "secondary" => (r.secondary, r.on_secondary),
        "onSecondary" => (r.on_secondary, r.secondary),
        "secondaryContainer" => (r.secondary_container, r.on_secondary_container),
        "onSecondaryContainer" => (r.on_secondary_container, r.secondary_container),
        "tertiary" => (r.tertiary, r.on_tertiary),
        "onTertiary" => (r.on_tertiary, r.tertiary),
        "tertiaryContainer" => (r.tertiary_container, r.on_tertiary_container),
        "onTertiaryContainer" => (r.on_tertiary_container, r.tertiary_container),
        "error" => (r.error, r.on_error),
        "onError" => (r.on_error, r.error),
        "errorContainer" => (r.error_container, r.on_error_container),
        "onErrorContainer" => (r.on_error_container, r.error_container),
        "surface" => (r.surface, r.on_surface),
        "onSurface" => (r.on_surface, r.surface),
        "surfaceVariant" => (r.surface_variant, r.on_surface_variant),
        "onSurfaceVariant" => (r.on_surface_variant, r.surface_variant),
        "outline" => (r.outline, r.surface),
        "outlineVariant" => (r.outline_variant, r.on_surface),
        _ => (r.surf_cont, r.on_surface),
    }
}

/// A colour-role band: the reference writes each role's name *on* the role, in
/// the ink that pairs with it — which is what makes the screen a palette rather
/// than a row of squares.
fn color_swatch(a: &Attrs, r: &Roles) -> UiNode {
    let (fill, ink) = match a.group.as_deref() {
        Some(role) => role_pair(r, role),
        None => (a.bg.unwrap_or(r.primary), r.on_surface),
    };
    let mut band = col();
    band.attrs.bg = Some(fill);
    // +1dp on whatever the screen asks for. The reference's swatches butt flush
    // against each other at 158px for a stated 56dp; these render 155 and leave
    // a 3px seam between every pair. Not a global scale — a button's 40dp is
    // 113px in both — so it is corrected here rather than in the emitter.
    band.attrs.h = Some(a.h.unwrap_or(56.0) + 2.0);
    band.attrs.padx = Some(16.0);
    band.attrs.aligny = Some(0.5);
    band.attrs.radius = Some(0.0);
    // `w: 999` in the reference means "as wide as the row".
    if a.w.unwrap_or(0.0) > 500.0 {
        band.attrs.fillw = Some(1);
    } else {
        band.attrs.w = a.w;
    }
    if let Some(name) = a.text.as_deref().or(a.label.as_deref()) {
        let mut t = label(name, 14.0, ink);
        t.attrs.fillw = Some(1);
        band.children.push(t);
    }
    let mut outer = col();
    outer.attrs.fillw = Some(1);
    let mut gap = col();
    gap.attrs.h = Some(4.0);
    outer.children.push(band);
    outer.children.push(gap);
    outer
}
fn badge_icon(a: &Attrs, r: &Roles) -> UiNode {
    let mut b = row();
    b.attrs.spacing = Some(4.0);
    b.attrs.fitw = Some(1);
    b.children.push(glyph(
        a.icon_name.as_deref().unwrap_or("notifications"),
        22.0,
        r.on_surface,
    ));
    // Material caps an overflowing badge rather than printing the number: the
    // reference shows "999+" where the raw value is 1234.
    let count = a.badge.clone().or_else(|| {
        a.count.map(|c| if c > 999 { "999+".to_string() } else { c.to_string() })
    });
    let count = count.map(|c| match c.trim().parse::<i64>() {
        Ok(n) if n > 999 => "999+".to_string(),
        _ => c,
    });
    if let Some(count) = count {
        let mut pill = boxed(Some(r.error), 8.0);
        pill.attrs.h = Some(16.0);
        pill.attrs.padx = Some(4.0);
        pill.attrs.fitw = Some(1);
        let mut t = label(&count, 11.0, r.on_error);
        t.attrs.weight = Some(500);
        pill.children.push(t);
        b.children.push(pill);
    }
    b
}

fn search_bar(a: &Attrs, r: &Roles) -> UiNode {
    let mut s = row();
    s.attrs.bg = Some(r.surf_high);
    s.attrs.radius = Some(28.0);
    s.attrs.h = Some(56.0);
    s.attrs.padx = Some(16.0);
    s.attrs.spacing = Some(16.0);
    s.attrs.aligny = Some(0.5);
    s.children.push(glyph("place", 20.0, r.on_surface_variant));
    s.children.push(label(
        a.hint.as_deref().or(a.text.as_deref()).unwrap_or("Search"),
        16.0,
        r.on_surface_variant,
    ));
    s
}

fn dropdown(a: &Attrs, r: &Roles) -> UiNode {
    let mut d = row();
    d.attrs.radius = Some(4.0);
    d.attrs.border = Some(1.0);
    d.attrs.bordercolor = Some(r.outline);
    d.attrs.h = Some(56.0);
    d.attrs.padx = Some(16.0);
    d.attrs.aligny = Some(0.5);
    d.attrs.spacing = Some(12.0);
    let chosen = a
        .items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .nth(a.selected.unwrap_or(0).max(0) as usize)
        .unwrap_or("");
    let shown = if chosen.is_empty() {
        a.hint.as_deref().unwrap_or("Choose")
    } else {
        chosen
    };
    let mut t = label(shown, 16.0, r.on_surface);
    t.attrs.fitw = Some(1);
    d.children.push(t);
    d.children.push(col()); // spacer pushes the caret to the trailing edge
    d.children.push(label("\u{25BE}", 14.0, r.on_surface_variant));
    d
}

fn tabs(a: &Attrs, r: &Roles) -> UiNode {
    let sel = a.selected.unwrap_or(0);
    // The reference's tab strip spans its row and divides evenly — its three
    // tabs centre on thirds of the full width (measured 207/535/868), where a
    // hugging strip clustered them left at 156/410/659.
    let mut strip = row();
    strip.attrs.fillw = Some(1);
    for (i, item) in items_of(a).into_iter().enumerate() {
        let on = i as i32 == sel;
        let ink = if on { r.primary } else { r.on_surface_variant };
        let mut tab = col();
        if let Some(k) = a.key.as_deref() {
            tab.attrs.tapto = Some(format!("set:{k}={i}"));
        }
        tab.attrs.h = Some(48.0);
        tab.attrs.fillw = Some(1);
        tab.attrs.alignx = Some(0.5);
        tab.attrs.aligny = Some(0.5);
        let mut head = row();
        head.attrs.spacing = Some(6.0);
        head.attrs.aligny = Some(0.5);
        head.attrs.fitw = Some(1);
        if let Some(icon) = list_at(a.icon_name.as_deref(), i) {
            head.children.push(glyph(icon, 18.0, ink));
        }
        let mut t = label_lg(item, ink);
        t.attrs.h = Some(24.0);
        head.children.push(t);
        if let Some(count) = list_at(a.badge.as_deref(), i) {
            let mut dot = boxed(Some(r.error), 8.0);
            dot.attrs.h = Some(16.0);
            dot.attrs.padx = Some(4.0);
            dot.attrs.fitw = Some(1);
            let mut bt = label(count, 11.0, r.on_error);
            bt.attrs.weight = Some(500);
            dot.children.push(bt);
            head.children.push(dot);
        }
        tab.children.push(head);
        // The 3dp active indicator is the whole point of a tab strip.
        let mut ind = col();
        ind.attrs.h = Some(3.0);
        ind.attrs.w = Some(40.0);
        ind.attrs.radius = Some(2.0);
        ind.attrs.bg = Some(if on { r.primary } else { 0 });
        tab.children.push(ind);
        strip.children.push(tab);
    }
    strip
}

fn nav_bar(a: &Attrs, r: &Roles) -> UiNode {
    let sel = a.selected.unwrap_or(0);
    let mut bar = row();
    bar.attrs.bg = Some(r.surf_cont);
    // 105dp, measured off the reference's own bar (293px); M3's 80 left it short.
    bar.attrs.h = Some(105.0);
    bar.attrs.pad = Some(8.0);
    bar.attrs.radius = Some(16.0);
    bar.attrs.fillw = Some(1);
    for (i, item) in items_of(a).into_iter().enumerate() {
        let on = i as i32 == sel;
        let ink = if on {
            r.on_secondary_container
        } else {
            r.on_surface_variant
        };
        // Destinations divide the bar evenly — the reference's four sit at
        // 168/415/673/893 across the full width, where hugging clustered them
        // left at 157/337/517/697.
        let mut dest = col();
        dest.attrs.fillw = Some(1);
        dest.attrs.spacing = Some(4.0);
        dest.attrs.alignx = Some(0.5);
        if let Some(k) = a.key.as_deref() {
            dest.attrs.tapto = Some(format!("set:{k}={i}"));
        }
        // The 64x32 active-indicator pill marks the destination.
        let mut pill = boxed(if on { Some(r.secondary_container) } else { None }, 16.0);
        pill.attrs.w = Some(64.0);
        pill.attrs.h = Some(32.0);
        let icon = list_at(a.icon_name.as_deref(), i).unwrap_or("star");
        // A badge rides the icon, so it has to sit in the pill with it.
        match list_at(a.badge.as_deref(), i) {
            Some(count) => {
                let mut stack = row();
                stack.attrs.spacing = Some(2.0);
                stack.attrs.fitw = Some(1);
                stack.children.push(glyph(icon, 20.0, ink));
                let mut dot = boxed(Some(r.error), 8.0);
                dot.attrs.h = Some(16.0);
                dot.attrs.padx = Some(4.0);
                dot.attrs.fitw = Some(1);
                let mut t = label(count, 11.0, r.on_error);
                t.attrs.weight = Some(500);
                dot.children.push(t);
                stack.children.push(dot);
                pill.children.push(stack);
            }
            None => pill.children.push(glyph(icon, 20.0, ink)),
        }
        dest.children.push(pill);
        // Only the selected destination is labelled. The reference does this on
        // every one of its three bars — including the one captioned "labels
        // visible when inactive" — so it is the behaviour, not the M3 default.
        if on {
            let mut t = label(item, 12.0, ink);
            t.attrs.weight = Some(500);
            dest.children.push(t);
        }
        bar.children.push(dest);
    }
    bar
}

fn nav_rail(a: &Attrs, r: &Roles) -> UiNode {
    let sel = a.selected.unwrap_or(0);
    let mut rail = col();
    rail.attrs.bg = Some(r.surface);
    rail.attrs.w = Some(80.0);
    rail.attrs.pad = Some(8.0);
    rail.attrs.spacing = Some(12.0);
    rail.attrs.alignx = Some(0.5);
    for (i, item) in items_of(a).into_iter().enumerate() {
        let on = i as i32 == sel;
        let ink = if on {
            r.on_secondary_container
        } else {
            r.on_surface_variant
        };
        let mut dest = col();
        if let Some(k) = a.key.as_deref() {
            dest.attrs.tapto = Some(format!("set:{k}={i}"));
        }
        dest.attrs.spacing = Some(4.0);
        dest.attrs.alignx = Some(0.5);
        let mut pill = boxed(if on { Some(r.secondary_container) } else { None }, 16.0);
        pill.attrs.w = Some(56.0);
        pill.attrs.h = Some(32.0);
        pill.children.push(glyph(
            list_at(a.icon_name.as_deref(), i).unwrap_or("star"),
            20.0,
            ink,
        ));
        dest.children.push(pill);
        let mut t = label(item, 12.0, ink);
        t.attrs.weight = Some(500);
        dest.children.push(t);
        rail.children.push(dest);
    }
    rail
}

fn carousel(a: &Attrs, r: &Roles) -> UiNode {
    // The reference's items are 220x200dp with a 20dp corner and a Title Medium
    // white caption at 16dp (420dp tall for the fullscreen strategy, which pages
    // vertically). The strategy then masks them to different widths across the
    // viewport, which is what these per-strategy numbers stand in for.
    let v = variant(a, "hero");
    let full = v == "fullscreen";
    let h = if full { 420.0 } else { 200.0 };
    // Measured off the reference's own render rather than guessed: its
    // CarouselLayoutManager strategies resolve to these item widths at this
    // viewport, and the strategy *is* the set of widths.
    let widths: &[f32] = match v {
        "multibrowse" => &[207.0, 106.0, 40.0, 12.0],
        "uncontained" => &[221.0, 133.0],
        "fullscreen" => &[355.0],
        _ => &[313.0, 40.0, 12.0],
    };
    let count = a.count.unwrap_or(3).max(1) as usize;
    let mut strip = row();
    // The reference's tiles abut — no gap resolves between them at this
    // viewport, and its strips total exactly 365dp. A 6dp spacing left every
    // strategy short and dropped the trailing masked slivers off the edge.
    strip.attrs.spacing = Some(0.0);
    strip.attrs.h = Some(h);
    for (i, w) in widths.iter().enumerate().take(count) {
        // The reference's tile gradient runs diagonally; makepad's shader does
        // vertical or horizontal only (horizontal measured worse — carousel
        // 10.6 -> 11.2). Keep it vertical but compress the stops to the middle
        // half, because a diagonal spreads its range across both axes: the
        // reference varies #818BDA -> #8398E0 down a tile where the full-range
        // stops gave #7F81D6 -> #86A8E7. Calibrated, not guessed: the
        // reference's vertical delta is 13 of the gradient's full 41 on green,
        // so the visible span is 32% of the range — stops at 0.34 and 0.66.
        let (g0, g1) = GRADIENTS[i % GRADIENTS.len()];
        // Solved for the reference's split: its tile varies 32% of the gradient
        // range down and 51% across. With the overlay at alpha a, vertical
        // shows (1-a)*span_v and horizontal a*span_h, so a=0.6 with spans of
        // 0.80 and 0.85 lands both.
        let (from, to) = (mix_rgb(g0, g1, 0.10), mix_rgb(g0, g1, 0.90));
        let (afrom, ato) = (mix_rgb(g0, g1, 0.075), mix_rgb(g0, g1, 0.925));
        // Full-range vertical here, with a half-alpha horizontal pass over it
        // (below) — the two average to the diagonal the reference draws.
        let mut item = col();
        item.attrs.bg = Some(from);
        item.attrs.bg2 = Some(to);
        item.attrs.radius = Some(20.0);
        item.attrs.w = Some(*w);
        item.attrs.h = Some(h);
        item.attrs.pad = Some(16.0);
        item.attrs.aligny = Some(1.0);
        // Half-alpha horizontal pass: mixed over the vertical fill beneath it,
        // the two average to the diagonal gradient the reference draws. The
        // shader does one axis per pass, so a diagonal needs two.
        let mut across = col();
        across.attrs.group = Some("gradh".to_string());
        across.attrs.bg = Some((afrom & 0x00FF_FFFF) | 0x9900_0000);
        across.attrs.bg2 = Some((ato & 0x00FF_FFFF) | 0x9900_0000);
        across.attrs.radius = Some(20.0);
        across.attrs.w = Some(*w);
        across.attrs.h = Some(h);
        item.children.push(across);
        // A masked sliver has no room for its caption; the reference's are bare.
        if *w > 90.0 {
            let mut t = label(
                &if i == 0 { "Item 1".to_string() } else { format!("{}", i + 1) },
                16.0,
                0xFFFFFFFF,
            );
            t.attrs.weight = Some(500);
            item.children.push(t);
        }
        strip.children.push(item);
    }
    let _ = r;
    strip
}

fn radio_group(a: &Attrs, r: &Roles) -> UiNode {
    let sel = a.selected.unwrap_or(0);
    let mut g = col();
    g.attrs.spacing = Some(4.0);
    let count = a
        .items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
        .count();
    for (i, item) in a
        .items
        .as_deref()
        .unwrap_or("")
        .split(';')
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let mut radio = n(NodeKind::Radio);
        radio.attrs.text = Some(item.to_string());
        radio.attrs.color = Some(r.on_surface);
        radio.attrs.accent = Some(r.primary);
        radio.attrs.bordercolor = Some(r.on_surface_variant);
        radio.attrs.bg = Some(0);
        radio.attrs.h = Some(34.0);
        radio.attrs.on = Some((i as i32 == sel) as i32);
        // Android reports the *index* when a radio is checked
        // (`onState(key, String.valueOf(i))`). `RadioButton` has no `on_click`
        // field -- unlike `Button` and `CheckBox` -- so the tap has to sit on a
        // container, which is what gets the click overlay.
        match a.key.as_deref() {
            Some(k) => {
                let mut hit = row();
                hit.attrs.fillw = Some(1);
                hit.attrs.aligny = Some(0.5);
                hit.attrs.tapto = Some(format!("set:{k}={i}"));
                hit.children.push(radio);
                g.children.push(hit);
            }
            None => g.children.push(radio),
        }
    }
    g
}

fn app_bar(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "small");
    // Heights measured off the reference's own bars, not M3's 112/152: it
    // renders medium at 121dp and large at 134dp.
    let (h, size, centre) = match v {
        "medium" => (121.0, 24.0, false),
        "large" => (134.0, 28.0, false),
        "center" => (64.0, 22.0, true),
        _ => (64.0, 22.0, false),
    };
    let title = a.title.as_deref().or(a.text.as_deref()).unwrap_or("Title");
    let mut bar = col();
    bar.attrs.bg = Some(r.surf_cont);
    bar.attrs.h = Some(h);
    bar.attrs.radius = Some(0.0);
    bar.attrs.pad = Some(8.0);
    let mut top = row();
    top.attrs.h = Some(48.0);
    top.attrs.aligny = Some(0.5);
    top.attrs.spacing = Some(8.0);
    top.children.push(glyph("chat", 20.0, r.on_surface));
    if centre {
        top.children.push(col());
    }
    let mut t = label(title, size, r.on_surface);
    t.attrs.fitw = Some(1);
    top.children.push(t);
    top.children.push(col());
    top.children.push(glyph("settings", 20.0, r.on_surface_variant));
    // A medium/large bar carries its title on a second line under the actions.
    if v == "medium" || v == "large" {
        let mut act = row();
        act.attrs.h = Some(48.0);
        act.attrs.aligny = Some(0.5);
        act.attrs.spacing = Some(8.0);
        act.children.push(glyph("chat", 20.0, r.on_surface));
        act.children.push(col());
        act.children.push(glyph("settings", 20.0, r.on_surface_variant));
        bar.children.push(act);
        let mut big = label(title, size, r.on_surface);
        big.attrs.fillw = Some(1);
        bar.children.push(big);
    } else {
        bar.children.push(top);
    }
    bar
}

fn bottom_bar(r: &Roles) -> UiNode {
    // From the reference's `bottomBarDemo`: an *outlined card* 120dp tall with a
    // BottomAppBar along its bottom edge — navigation icon, search and a drag
    // handle — and the FAB cradled at centre, overlapping the bar's top edge.
    // This used to be a filled row of unrelated icons with the FAB inline.
    let mut card = col();
    card.attrs.radius = Some(12.0);
    card.attrs.border = Some(1.0);
    card.attrs.bordercolor = Some(r.outline_variant);
    card.attrs.h = Some(120.0);
    card.attrs.fillw = Some(1);
    // The reference's icon row sits just under the cradled FAB, not on the card's
    // bottom edge — its CoordinatorLayout anchor is never set, so the FAB lands
    // top-left and the bar rides with it. Matching what it renders, not what a
    // BottomAppBar would do with the anchor wired up.
    card.attrs.aligny = Some(0.42);

    let mut bar = row();
    bar.attrs.bg = Some(r.surf_cont);
    bar.attrs.h = Some(64.0);
    bar.attrs.padx = Some(12.0);
    bar.attrs.spacing = Some(20.0);
    bar.attrs.aligny = Some(0.5);
    bar.attrs.fillw = Some(1);
    for i in ["menu", "search", "drag_handle"] {
        bar.children.push(glyph(i, 22.0, r.on_surface_variant));
    }
    card.children.push(bar);

    // The cradled FAB sits over the bar rather than inside its row.
    let mut fab_row = row();
    fab_row.attrs.h = Some(120.0);
    fab_row.attrs.fillw = Some(1);
    fab_row.attrs.alignx = Some(0.0);
    fab_row.attrs.aligny = Some(0.0);
    let mut f = boxed(Some(r.primary_container), 16.0);
    f.attrs.w = Some(56.0);
    f.attrs.h = Some(56.0);
    f.children.push(glyph("add", 24.0, r.on_primary_container));
    fab_row.children.push(f);

    let mut stack = n(NodeKind::Stack);
    stack.attrs.fillw = Some(1);
    stack.attrs.h = Some(120.0);
    stack.children.push(card);
    stack.children.push(fab_row);
    stack
}

fn toolbar(a: &Attrs, r: &Roles) -> UiNode {
    let v = variant(a, "docked");
    // The reference's toolbars hold text-format actions, not a generic icon set.
    let icons = [
        "format_bold",
        "format_italic",
        "format_underlined",
        "format_align_center",
    ];
    let vertical = v == "vertical";
    let mut bar = if vertical { col() } else { row() };
    // All three sit on a container tone in the reference — the floating ones are
    // not filled with primary, which read far louder than the original.
    bar.attrs.bg = Some(r.surf_cont);
    bar.attrs.radius = Some(if v.starts_with("floating") { 28.0 } else { 16.0 });
    bar.attrs.pad = Some(8.0);
    bar.attrs.spacing = Some(8.0);
    bar.attrs.aligny = Some(0.5);
    bar.attrs.alignx = Some(0.5);
    if vertical {
        // The vertical toolbar is a full-width panel with its actions stacked at
        // the leading edge, not a narrow strip.
        bar.attrs.fillw = Some(1);
        bar.attrs.alignx = Some(0.0);
        bar.attrs.padx = Some(16.0);
        bar.attrs.spacing = Some(16.0);
    } else {
        bar.attrs.h = Some(64.0);
        // A docked toolbar spans its row; only the floating ones hug.
        // Docked and floating both span the row in the reference.
        bar.attrs.fillw = Some(1);
        bar.attrs.alignx = Some(0.5);
    }
    let ink = r.primary;
    for i in icons {
        bar.children.push(glyph(i, 20.0, ink));
    }
    // The "floating toolbar with a FAB" pairs the strip with a real FAB.
    if v == "floatingfab" {
        let mut f = boxed(Some(r.primary), 16.0);
        f.attrs.w = Some(48.0);
        f.attrs.h = Some(48.0);
        f.children.push(glyph("add", 22.0, r.on_primary));
        bar.children.push(f);
    }
    bar
}

fn adaptive(a: &Attrs, r: &Roles) -> UiNode {
    // From the reference's `adaptiveDemo`: list/detail and supporting both build
    // an *outlined card of three list items*, and feed a grid of six filled
    // "Card N" tiles 96dp tall. The panes this used to invent shared nothing with
    // it but the section heading.
    let v = variant(a, "listdetail");
    if v == "feed" {
        let mut grid = col();
        grid.attrs.spacing = Some(8.0);
        grid.attrs.fillw = Some(1);
        for line_i in 0..3 {
            let mut line = row();
            line.attrs.spacing = Some(8.0);
            line.attrs.fillw = Some(1);
            for col_i in 0..2 {
                let mut card = col();
                card.attrs.bg = Some(r.surf_highest);
                card.attrs.radius = Some(12.0);
                card.attrs.h = Some(96.0);
                card.attrs.w = Some(170.0);
                card.attrs.pad = Some(12.0);
                let mut t = label(&format!("Card {}", line_i * 2 + col_i + 1), 14.0, r.on_surface);
                t.attrs.weight = Some(500);
                card.children.push(t);
                line.children.push(card);
            }
            grid.children.push(line);
        }
        return grid;
    }

    let mut wrap = col();
    wrap.attrs.spacing = Some(8.0);
    wrap.attrs.fillw = Some(1);

    // The outlined card holding the items.
    let mut list = col();
    list.attrs.radius = Some(12.0);
    list.attrs.border = Some(1.0);
    list.attrs.bordercolor = Some(r.outline_variant);
    list.attrs.fillw = Some(1);
    let items = if v == "supporting" { 2 } else { 3 };
    for i in 1..=items {
        let mut item = col();
        item.attrs.h = Some(72.0);
        item.attrs.padx = Some(16.0);
        item.attrs.aligny = Some(0.5);
        item.attrs.spacing = Some(2.0);
        item.attrs.fillw = Some(1);
        let mut t = label(&format!("Item {i}"), 16.0, r.on_surface);
        t.attrs.fillw = Some(1);
        item.children.push(t);
        let mut sup = label("Supporting text", 14.0, r.on_surface_variant);
        sup.attrs.fillw = Some(1);
        item.children.push(sup);
        list.children.push(item);
    }
    wrap.children.push(list);

    // …and the pane beneath it, which is what the layout is demonstrating.
    let mut pane = col();
    pane.attrs.bg = Some(r.surf_highest);
    pane.attrs.radius = Some(12.0);
    pane.attrs.pad = Some(16.0);
    pane.attrs.fillw = Some(1);
    let text = if v == "supporting" {
        "Supporting pane — secondary content."
    } else {
        "Detail pane — the selected item's content."
    };
    let mut t = label(text, 14.0, r.on_surface);
    t.attrs.fillw = Some(1);
    pane.children.push(t);
    wrap.children.push(pane);
    wrap
}

fn transition_host(a: &Attrs, r: &Roles) -> UiNode {
    // Taken from the reference's `transitionHost`/`containerCollapsed`: the
    // collapsed card is a *filled* MaterialCardView 88dp tall inside a host of
    // `h`, carrying a 56dp grad1 thumbnail at a 12dp corner with 16dp padding.
    let v = variant(a, "stage");
    let mut host = col();
    host.attrs.h = a.h;
    host.attrs.fillw = Some(1);
    if v == "stage" {
        // The reference fills this pane at #313539 — surf_highest, the same
        // tone as its collapsed card — not the dimmer surf_cont.
        host.attrs.bg = Some(r.surf_highest);
        host.attrs.radius = Some(12.0);
        host.attrs.pad = Some(16.0);
        host.attrs.spacing = Some(8.0);
        let mut t = label("Pane 1", 22.0, r.on_surface);
        t.attrs.fillw = Some(1);
        host.children.push(t);
        let mut b = label("Press a motion button above", 14.0, r.on_surface_variant);
        b.attrs.fillw = Some(1);
        host.children.push(b);
        return host;
    }
    let mut card = row();
    // Tapping the collapsed card is what runs the container transform in the
    // reference; without a handler it was a picture of the demo.
    if let Some(k) = a.key.as_deref() {
        card.attrs.tapto = Some(format!("set:{k}=1"));
    }
    card.attrs.bg = Some(r.surf_highest);
    card.attrs.radius = Some(12.0);
    card.attrs.h = Some(88.0);
    card.attrs.pad = Some(16.0);
    card.attrs.spacing = Some(16.0);
    card.attrs.aligny = Some(0.5);
    card.attrs.fillw = Some(1);
    let (g0, g1) = GRADIENTS[0];
    let (from, to) = grad_span(g0, g1);
    let mut thumb = col();
    thumb.attrs.bg = Some(from);
    thumb.attrs.bg2 = Some(to);
    thumb.attrs.w = Some(56.0);
    thumb.attrs.h = Some(56.0);
    thumb.attrs.radius = Some(12.0);
    card.children.push(thumb);
    let mut txt = col();
    txt.attrs.spacing = Some(2.0);
    txt.attrs.fillw = Some(1);
    let mut h = label("Container transform", 16.0, r.on_surface);
    h.attrs.weight = Some(500);
    txt.children.push(h);
    txt.children.push(label("Tap to expand", 14.0, r.on_surface_variant));
    card.children.push(txt);
    host.children.push(card);
    host
}

/// octos-one's weather glyph set, by condition index.
fn weather_icon(a: &Attrs, r: &Roles) -> UiNode {
    const CONDITIONS: [&str; 8] = [
        "star", "star", "chat", "chat", "place", "place", "repeat", "repeat",
    ];
    let i = a.value.unwrap_or(0.0) as usize % CONDITIONS.len();
    let size = a.w.unwrap_or(88.0);
    let mut b = boxed(Some(r.surf_cont), 16.0);
    b.attrs.w = Some(size);
    b.attrs.h = Some(size);
    b.children
        .push(glyph(CONDITIONS[i], size * 0.42, r.primary));
    b
}

fn nav_map(a: &Attrs, r: &Roles) -> UiNode {
    let mut m = boxed(Some(r.surf_high), 16.0);
    m.attrs.h = a.h.or(Some(200.0));
    m.attrs.fillw = Some(1);
    m.children.push(glyph("place", 32.0, r.primary));
    m
}

fn glass_panel(node: &UiNode, r: &Roles) -> UiNode {
    let a = &node.attrs;
    let v = variant(a, "panel");
    let mut p = col();
    p.attrs.radius = a.radius.or(Some(16.0));
    p.attrs.pad = Some(16.0);
    p.attrs.spacing = Some(8.0);
    p.attrs.fillw = Some(1);
    // No backdrop blur in this dialect; approximate the glass with a translucent
    // surface over a hairline, which is what reads as "panel" at this size.
    p.attrs.bg = Some(match v {
        "clear" => (r.surf_highest & 0x00FF_FFFF) | 0x30000000,
        "card" => (r.surf_high & 0x00FF_FFFF) | 0xB0000000,
        _ => (r.surf_cont & 0x00FF_FFFF) | 0x80000000,
    });
    p.attrs.border = Some(1.0);
    p.attrs.bordercolor = Some((r.outline & 0x00FF_FFFF) | 0x60000000);
    p.children.extend(node.children.iter().cloned());
    p
}

/// An M3 text field: floating label, supporting or error line, leading icon,
/// trailing affordance, filled or outlined, in every state the reference shows.
///
/// The native `TextInput` alone draws a plain box — no label, no supporting
/// text, no error treatment — so the chrome is composed around it and the widget
/// keeps only the job it is actually good at, holding and editing the value.
fn text_field(node: &UiNode, r: &Roles) -> UiNode {
    let a = &node.attrs;
    let outlined = variant(a, "filled") == "outlined";
    let enabled = is_enabled(a);
    let has_error = a.error.is_some();
    let value = a.text.clone().unwrap_or_default();
    let filled_in = !value.is_empty();

    // Error wins over every other role; disabled dims whatever is left.
    let edge = if has_error {
        r.error
    } else if outlined {
        r.outline
    } else {
        r.on_surface_variant
    };
    let (edge, ink, label_ink) = if enabled {
        (edge, r.on_surface, if has_error { r.error } else { r.on_surface_variant })
    } else {
        (
            dim(r.on_surface, 0.12),
            dim(r.on_surface, 0.38),
            dim(r.on_surface, 0.38),
        )
    };

    let mut field = col();
    field.attrs.spacing = Some(4.0);
    field.attrs.fillw = Some(1);

    // The label floats above once the field has content; otherwise it sits
    // inside as the placeholder, which is what M3 does.
    if filled_in || has_error {
        if let Some(hint) = a.hint.as_deref() {
            let mut l = label(hint, 12.0, label_ink);
            l.attrs.fillw = Some(1);
            field.children.push(l);
        }
    }

    let mut boxrow = row();
    // Stated a hair under 56 so it lands on 56 after the emitter's DP_SCALE:
    // the field is one of the few controls the reference draws at exactly its
    // nominal size, so it should not take the global correction.
    boxrow.attrs.h = Some(match a.lines.unwrap_or(1) {
        n if n > 1 => 55.65 + (n - 1) as f32 * 23.85,
        _ => 55.65,
    });
    // Spacer, not `padx`: on a fixed-height box the only padding form this
    // dialect honours is the symmetric scalar, which added 16dp *vertically* too
    // and pushed the label to 70% down the box instead of centring it.
    boxrow.attrs.spacing = Some(12.0);
    boxrow.attrs.aligny = Some(0.5);
    boxrow.attrs.fillw = Some(1);
    // Filled sits on surfaceContainerHighest with a bottom indicator; outlined
    // is transparent inside a 1dp ring.
    if outlined {
        boxrow.attrs.radius = Some(4.0);
        boxrow.attrs.border = Some(1.0);
        boxrow.attrs.bordercolor = Some(edge);
    } else {
        boxrow.attrs.radius = Some(4.0);
        boxrow.attrs.bg = Some(if enabled {
            r.surf_highest
        } else {
            dim(r.on_surface, 0.04)
        });
    }
    // 4, not M3's 16: makepad's TextInput carries ~18dp of its own inset that a
    // `padding: 0` on it does not clear, so the gap only tops it up.
    boxrow.children.push(sl_gap(4.0));
    if let Some(icon) = a.icon_name.as_deref() {
        boxrow.children.push(glyph(icon, 20.0, label_ink));
    }
    // A real TextInput, so the field is still editable — the chrome around it is
    // what the native widget cannot draw, not a replacement for it.
    let mut input = n(NodeKind::Input);
    input.attrs.text = (!value.is_empty()).then(|| value.clone());
    input.attrs.placeholder = a.hint.clone();
    // 16sp: the reference's field text measures 34px tall against 28 at the
    // widget's default, which is also 34px of the gap it leaves below a chips
    // row on `allcomponents`.
    input.attrs.size = Some(16.0);
    input.attrs.color = Some(if filled_in { ink } else { label_ink });
    input.attrs.bg = Some(0);
    input.attrs.bordercolor = Some(0);
    input.attrs.border = Some(0.0);
    input.attrs.fillw = Some(1);
    input.attrs.h = Some(40.0);
    input.attrs.enabled = a.enabled;
    boxrow.children.push(input);
    // A password field carries the reveal affordance; the error state its icon.
    if has_error {
        boxrow.children.push(glyph("error", 20.0, r.error));
    } else if a.action.as_deref() == Some("password") {
        boxrow.children.push(glyph("visibility", 20.0, label_ink));
    }
    boxrow.children.push(sl_gap(4.0));
    field.children.push(boxrow);

    // The filled variant's active indicator is a line, not a ring.
    if !outlined {
        let mut rule = col();
        rule.attrs.h = Some(1.0);
        rule.attrs.bg = Some(edge);
        rule.attrs.fillw = Some(1);
        field.children.push(rule);
    }

    if let Some(msg) = a.error.as_deref().or(a.helper.as_deref()).or(a.supporting.as_deref()) {
        // M3 insets supporting text 16dp from the box edge; the reference's
        // starts at x90 against a box at x45.
        let mut line = row();
        line.attrs.fillw = Some(1);
        line.children.push(sl_gap(16.0));
        let mut h = label(msg, 12.0, if has_error { r.error } else { label_ink });
        h.attrs.fillw = Some(1);
        line.children.push(h);
        field.children.push(line);
    }
    field
}


/// The reference's three named gradients, taken from its own `Builder.gradient`
/// (`GradientDrawable`, TL→BR) rather than invented — they are generated the
/// same way on both sides, so they can simply agree.
const GRADIENTS: [(u32, u32); 3] = [
    (0xFF7F7FD5, 0xFF86A8E7),
    (0xFF43C6AC, 0xFF191654),
    (0xFFFF9966, 0xFFFF5E62),
];

/// A switch row: label leading, control trailing.
///
/// makepad's `Toggle` draws its switch *before* the label; Material puts it at
/// the trailing edge of a full-width row, which is what the reference shows. The
/// control stays a real Toggle so the row is still interactive — only the label
/// moves out of it.
/// M3's switch, drawn. Measured off the reference at 52x32dp: selected is a
/// `primary` track with a 24dp `on_primary` thumb inset 24dp; unselected is a
/// `surf_highest` track with a 2dp `outline` border and a 16dp outline thumb
/// inset 8dp. Upstream's `Toggle` sizes its shader from its own theme rather
/// than its rect and renders 20dp tall whatever walk it is given — an explicit
/// 52x32, dropping `fitw`, and zeroing its padding were each tried on device.
/// So it stays underneath for the tap and this carries the visual, exactly as
/// the slider does.
fn drawn_switch(on: bool, enabled: bool, r: &Roles) -> UiNode {
    let mut track = row();
    track.attrs.w = Some(52.0);
    track.attrs.h = Some(32.0);
    track.attrs.radius = Some(16.0);
    track.attrs.aligny = Some(0.5);
    if on {
        track.attrs.bg = Some(if enabled { r.primary } else { dim(r.on_surface, 0.12) });
    } else {
        track.attrs.bg = Some(r.surf_highest);
        track.attrs.border = Some(2.0);
        track.attrs.bordercolor = Some(if enabled { r.outline } else { dim(r.on_surface, 0.12) });
    }
    let (lead, d, thumb) = if on {
        (24.0, 24.0, if enabled { r.on_primary } else { dim(r.on_surface, 0.38) })
    } else {
        (8.0, 16.0, if enabled { r.outline } else { dim(r.on_surface, 0.38) })
    };
    track.children.push(sl_gap(lead));
    let mut knob = col();
    knob.attrs.bg = Some(thumb);
    knob.attrs.w = Some(d);
    knob.attrs.h = Some(d);
    knob.attrs.radius = Some(d * 0.5);
    track.children.push(knob);
    track
}

fn switch_row(node: &UiNode, r: &Roles) -> UiNode {
    let a = &node.attrs;
    let mut row_ = row();
    // 56, though on `allcomponents` the gap from the button row to this one
    // measures 203px against the reference's 191: 52 is worse there (9.4 -> 9.8),
    // so the extra is not in this row's height.
    row_.attrs.h = a.h.or(Some(56.0));
    row_.attrs.aligny = Some(0.5);
    row_.attrs.fillw = Some(1);
    let ink = a.color.unwrap_or(if a.enabled.unwrap_or(1) == 0 {
        dim(r.on_surface, 0.38)
    } else {
        r.on_surface
    });
    let mut lab = label(a.text.as_deref().unwrap_or(""), 16.0, ink);
    lab.attrs.fillw = Some(1);
    row_.children.push(lab);
    row_.children.push(col()); // pushes the control to the trailing edge
    // The drawn track replaces the widget rather than covering it: layered over
    // the native Toggle it swallowed the tap (verified on device — the state
    // never flipped). The press writes the state slot instead, which is how
    // every other stateful control in this kit already works.
    let on = a.on.unwrap_or(0) != 0;
    let mut sw = drawn_switch(on, a.enabled.unwrap_or(1) != 0, r);
    if a.enabled.unwrap_or(1) != 0 {
        if let Some(k) = a.key.as_deref() {
            sw.attrs.tapto = Some(format!("set:{k}={}", i32::from(!on)));
        }
    }
    // The switch draws 20dp tall where the reference's is M3's 32dp, and it is
    // not reachable from here: upstream's Toggle sizes its shader off its own
    // theme, not its rect. Setting an explicit 52x32 walk, dropping `fitw`, and
    // zeroing the widget's `theme.mspace_2` padding were each tried on device
    // and each left it at 20dp. It needs a Toggle-side size property upstream.
    row_.children.push(sw);
    row_
}

/// A shapeable image — the reference's M3 corner treatments over a named
/// gradient. There is no bitmap: the catalog names three gradients, so the
/// renderer supplies them and applies the shape.
fn shapeable_image(a: &Attrs, r: &Roles) -> UiNode {
    let (g0, g1) = GRADIENTS[match a.src.as_deref().unwrap_or("grad1") {
        "grad2" => 1,
        "grad3" => 2,
        _ => 0,
    }];
    let (from, to) = grad_span(g0, g1);
    let w = a.w.unwrap_or(96.0);
    let h = a.h.unwrap_or(96.0);
    let mut img = col();
    img.attrs.bg = Some(from);
    img.attrs.bg2 = Some(to);
    img.attrs.w = Some(w);
    img.attrs.h = Some(h);
    // M3 corner treatments: fully round, a 16dp rounded square, or a cut corner —
    // which this dialect cannot chamfer, so it reads as the tightest radius.
    img.attrs.radius = Some(match variant(a, "rounded") {
        "circle" => w.min(h) / 2.0,
        "cut" => 2.0,
        _ => 16.0,
    });
    let _ = r;
    img
}

/// A Material slider: the native widget, with its handle and ticks drawn over it.
///
/// makepad only paints the handle on hover — `handle_sz = mix(0., handle_size,
/// self.hover)` in `slider.rs` — and a touch device never hovers, so on device
/// the control is a bare track and you cannot see where the value sits. Rather
/// than replace it (which would cost the dragging) the handle and the discrete
/// tick marks are laid over the real widget, which keeps the input.
// ---- M3 Expressive slider -------------------------------------------------
//
// Geometry measured off the reference's own render. makepad lays this device out at
// 2.80 px/dp (measured by rendering a known width and reading it back — the
// track height agrees), so: a 16.4dp track (46px), a 3.6x40.7dp bar handle
// (10x114px) with a 4.3dp gap either side (12px),
// capsule caps, and a 4dp dot inset ~6.5dp from any *unfilled* end. Discrete
// stops are dots inset 6.4dp from both ends. The 4.6dp handle/dot width is
// nominal: a rounded shape this small renders ~2px under its stated size, so
// the constants are trimmed to what actually lands on the reference's 10px., `on_primary` where the track is
// filled and `on_secondary_container` where it is not.
//
// Upstream's Slider draws a thin bevelled track with a round knob — a different
// design language, not a near miss — so it stays underneath, fully transparent,
// and owns only the drag. `accent: 0` is the emitter's cue to pin its whole
// draw_bg colour family; setting `color` alone leaves 20-odd siblings painting.
const TRACK_W: f32 = 325.0;
const TRACK_H: f32 = 16.4;
const HANDLE_W: f32 = 4.6;
const HANDLE_H: f32 = 40.7;
const HANDLE_GAP: f32 = 4.3;
const SDOT: f32 = 4.6;
const DOT_INSET: f32 = 6.1;
const TICK_INSET: f32 = 6.4;
// 50, measured under the aligned layout: 56 leaves `allcomponents` 31px too tall
// between its slider and the bar below (10.9 there), 46 was tuned for the old
// unaligned body and costs `slider` (11.2). At 50 both clear 9.0 — 40 routes
// under, against 39 at 56 and 38 at 46. This value, the swatch-group gap and the
// toolbar alignment all move together; each alone is worse than none.
const SLIDER_ROW: f32 = 50.0;
/// The reference's slider is inset further than the surrounding text — the
/// widget reserves room for the thumb. Measured at 40px past the screen padding.
const LEAD_INSET: f32 = 13.5;

fn sl_gap(w: f32) -> UiNode {
    let mut g = col();
    g.attrs.w = Some(w.max(0.0));
    g
}

fn sl_dot(c: u32) -> UiNode {
    let mut d = col();
    d.attrs.bg = Some(c);
    d.attrs.w = Some(SDOT);
    d.attrs.h = Some(SDOT);
    d.attrs.radius = Some(SDOT * 0.5);
    d
}

fn sl_handle(c: u32) -> UiNode {
    let mut h = col();
    h.attrs.bg = Some(c);
    h.attrs.w = Some(HANDLE_W);
    h.attrs.h = Some(HANDLE_H);
    h.attrs.radius = Some(HANDLE_W * 0.5);
    h
}

/// A capsule spanning `x0..x1` in track-local dp, carrying whichever stops fall
/// inside it plus, when `end_dot` is set, the dot at its outer end.
fn sl_seg(x0: f32, x1: f32, bg: u32, mark: u32, stops: &[f32], end_dot: Option<f32>) -> UiNode {
    let mut seg = row();
    seg.attrs.bg = Some(bg);
    seg.attrs.w = Some((x1 - x0).max(1.0));
    seg.attrs.h = Some(TRACK_H);
    seg.attrs.radius = Some(TRACK_H * 0.5);
    seg.attrs.aligny = Some(0.5);
    let mut marks: Vec<f32> = stops
        .iter()
        .copied()
        .filter(|s| *s >= x0 && *s + SDOT <= x1)
        .collect();
    if let Some(at) = end_dot {
        marks.push(at);
    }
    marks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut cur = 0.0;
    for m in marks {
        let at = (m - x0).max(cur);
        seg.children.push(sl_gap(at - cur));
        seg.children.push(sl_dot(mark));
        cur = at + SDOT;
    }
    seg
}

/// The stop positions of a discrete slider, in track-local dp.
fn sl_stops(a: &Attrs, span: f32) -> Vec<f32> {
    let Some(step) = a.step.filter(|s| *s > 0.0) else {
        return Vec::new();
    };
    let n = (span / step).round() as i32;
    if n <= 1 || n > 20 {
        return Vec::new();
    }
    let usable = TRACK_W - 2.0 * TICK_INSET;
    (0..=n)
        .map(|i| TICK_INSET + usable * i as f32 / n as f32 - SDOT * 0.5)
        .collect()
}

thread_local! {
    static SLIDER_IX: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Reset the per-screen slider numbering. Called once per translation, so the
/// indices a screen emits match the order the host collects them in.
pub fn reset_slider_index() {
    SLIDER_IX.with(|c| c.set(0));
}

fn next_slider_index() -> u32 {
    SLIDER_IX.with(|c| {
        let i = c.get();
        c.set(i + 1);
        i
    })
}

/// The draggable-but-invisible widget under a drawn track.
///
/// `frac` is the handle position as 0..1. It has to be passed in rather than
/// left as the node's own `value`: clearing min/max makes the emitter write a
/// 0..1 range, so a raw value (50 on a 0..100 slider) clamped to the maximum.
/// Invisible while nothing read the widget back, wrong the moment anything did.
fn sl_native(node: &UiNode, frac: f32) -> UiNode {
    let mut native = node.clone();
    // Each slider gets its own unit band -- slider *i* runs i..i+1 -- so the
    // value a `Slide` action carries says which slider moved. The host cannot
    // ask: a widget inside a mounted Splash body is not reachable by id even
    // with `widget_flood` (its actions escape, but a lookup returns nothing),
    // so identity has to travel in the one field that does come out. Bands stay
    // small ints to keep full float resolution on the fraction.
    // Bands are spaced *two* apart, not one: with 0..1 and 1..2 abutting, a
    // slider dragged fully right reports exactly 1.0, and floor(1.0) names the
    // next slider. Spacing them leaves the top of each band unambiguous.
    let idx = next_slider_index();
    let base = (idx * 2) as f32;
    native.attrs.min = Some(base);
    native.attrs.max = Some(base + 1.0);
    native.attrs.value = Some(base + frac);
    // Addressable, so the host can read the drag back out. Without an id the
    // emitted `Slider` is anonymous and its value can never reach the app --
    // which is why these tracked a finger and then snapped back to the DSL's
    // number, i.e. did not slide at all.
    if let Some(k) = node.attrs.key.as_deref() {
        native.attrs.id = Some(format!("sl_{k}"));
    }
    native.attrs.accent = Some(0);
    // makepad's Slider prints its own value ("50.00") beside the track; Material
    // does not, and the reference's screens carry a "Value: N" caption from the
    // DSL instead — so the widget's readout would be a duplicate.
    native.attrs.color = Some(0);
    native
}

fn sl_stack(native: UiNode, bar: UiNode) -> UiNode {
    let mut stack = n(NodeKind::Stack);
    stack.attrs.fillw = Some(1);
    // The track sits at the *top* of its 56dp row, not centred: section pitch
    // already matches the reference at 399px, and centring put the track 22px
    // below its own. Measured — 0.5 landed at y397, 0.30 at y388, 0.0 at y375
    // against the reference's y375.
    // The row is taller than the handle: Android's Slider reserves ~56dp, which
    // is what puts the reference's sections 399px apart against 357 here.
    stack.attrs.h = Some(SLIDER_ROW);
    stack.attrs.aligny = Some(0.0);
    stack.children.push(native);
    stack.children.push(bar);
    stack
}

fn slider(node: &UiNode, r: &Roles) -> UiNode {
    let a = &node.attrs;
    let enabled = is_enabled(a);
    let (lo, hi) = (a.min.unwrap_or(0.0), a.max.unwrap_or(1.0));
    let span = if (hi - lo).abs() < f32::EPSILON { 1.0 } else { hi - lo };
    let frac = ((a.value.unwrap_or(lo) - lo) / span).clamp(0.0, 1.0);

    let (act, inact) = if enabled {
        (a.accent.unwrap_or(r.primary), r.secondary_container)
    } else {
        (dim(r.on_surface, 0.38), dim(r.on_surface, 0.12))
    };
    let stops = sl_stops(a, span);
    let at = TRACK_W * frac;

    let mut bar = row();
    bar.attrs.aligny = Some(0.5);
    bar.attrs.h = Some(HANDLE_H);
    bar.children.push(sl_gap(LEAD_INSET));
    let lead_to = at - HANDLE_W * 0.5 - HANDLE_GAP;
    if lead_to > 1.0 {
        bar.children.push(sl_seg(0.0, lead_to, act, r.on_primary, &stops, None));
    }
    bar.children.push(sl_gap(HANDLE_GAP));
    bar.children.push(sl_handle(act));
    bar.children.push(sl_gap(HANDLE_GAP));
    let tail_from = at + HANDLE_W * 0.5 + HANDLE_GAP;
    if TRACK_W - tail_from > 1.0 {
        // Only a continuous track shows the end dot; a discrete one already has
        // a stop there.
        let dot = stops.is_empty().then_some(TRACK_W - DOT_INSET - SDOT);
        bar.children.push(sl_seg(
            tail_from,
            TRACK_W,
            inact,
            if enabled { r.on_secondary_container } else { r.on_surface },
            &stops,
            dot,
        ));
    }
    sl_stack(sl_native(node, frac), bar)
}

fn range_slider(a: &Attrs, r: &Roles) -> UiNode {
    // Two thumbs on one track. makepad's Slider carries a single value, so the
    // range is drawn rather than driven — stated plainly instead of pretending
    // the widget supports it.
    let (lo, hi) = (a.min.unwrap_or(0.0), a.max.unwrap_or(100.0));
    let span = if (hi - lo).abs() < f32::EPSILON { 1.0 } else { hi - lo };
    let f1 = ((a.value.unwrap_or(lo) - lo) / span).clamp(0.0, 1.0);
    let f2 = ((a.value2.unwrap_or(hi) - lo) / span).clamp(0.0, 1.0);
    let (f1, f2) = if f1 <= f2 { (f1, f2) } else { (f2, f1) };
    let enabled = is_enabled(a);
    let (act, inact) = if enabled {
        (a.accent.unwrap_or(r.primary), r.secondary_container)
    } else {
        (dim(r.on_surface, 0.38), dim(r.on_surface, 0.12))
    };
    let mark = if enabled { r.on_secondary_container } else { r.on_surface };
    let stops = sl_stops(a, span);
    let (x1, x2) = (TRACK_W * f1, TRACK_W * f2);

    let mut bar = row();
    bar.attrs.aligny = Some(0.5);
    bar.attrs.h = Some(HANDLE_H);
    bar.children.push(sl_gap(LEAD_INSET));
    // Both ends are unfilled here, so both carry a dot.
    let lead_to = x1 - HANDLE_W * 0.5 - HANDLE_GAP;
    if lead_to > 1.0 {
        bar.children.push(sl_seg(0.0, lead_to, inact, mark, &stops, Some(DOT_INSET)));
    }
    bar.children.push(sl_gap(HANDLE_GAP));
    bar.children.push(sl_handle(act));
    bar.children.push(sl_gap(HANDLE_GAP));
    let mid_from = x1 + HANDLE_W * 0.5 + HANDLE_GAP;
    let mid_to = x2 - HANDLE_W * 0.5 - HANDLE_GAP;
    if mid_to - mid_from > 1.0 {
        bar.children.push(sl_seg(mid_from, mid_to, act, r.on_primary, &stops, None));
    }
    bar.children.push(sl_gap(HANDLE_GAP));
    bar.children.push(sl_handle(act));
    bar.children.push(sl_gap(HANDLE_GAP));
    let tail_from = x2 + HANDLE_W * 0.5 + HANDLE_GAP;
    if TRACK_W - tail_from > 1.0 {
        bar.children.push(sl_seg(
            tail_from,
            TRACK_W,
            inact,
            mark,
            &stops,
            Some(TRACK_W - DOT_INSET - SDOT),
        ));
    }
    let mut stack = n(NodeKind::Stack);
    stack.attrs.fillw = Some(1);
    stack.attrs.h = Some(SLIDER_ROW);
    stack.attrs.aligny = Some(0.0);
    stack.children.push(bar);
    stack
}
