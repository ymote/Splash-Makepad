//! kit-host — a generic app shell that renders a Splash **component kit** as
//! native makepad widgets, against **upstream** makepad. It bakes one kit's
//! `.splash` (Material 3 here) and mounts it via the splash pipeline; native
//! controls are themed fork-free by `splash_widgets` (no makepad fork).
//!
//! Swap the baked kit (or push to the device path) to render iOS / liquid-glass.

pub use makepad_widgets;
use makepad_widgets::*;

mod screens;

app_main!(App);

// The route file the sweep writes, and the DSL override for hot-reload.
const DEVICE_PATH: &str = "/data/local/tmp/kit_host.splash";
const ROUTE_PATH: &str = "/data/local/tmp/kit_host.route";

/// Toolbar height. Paired with the 20dp inset above it: the two sum to the
/// 59.6dp that places every screen's body, so they may only move together.
const BAR_H: f32 = 39.6;

/// Adapter: the isolate hook takes a plain `fn(&mut ScriptVm)`, while a
/// `script_mod!` block returns a `ScriptValue`.
fn register_tap_mod(vm: &mut ScriptVm) {
    splash_widgets::tap::script_mod(vm);
}

/// A slider the host drives: its state slot, the DSL's range, and the value
/// last committed (so a drag only re-mounts when the value actually moves).
#[derive(Clone)]
pub struct SliderBind {
    key: String,
    lo: f32,
    hi: f32,
    step: Option<f32>,
    last: f32,
}

/// Collect every keyed slider in a semantic tree.
fn collect_sliders(node: &splash_render::UiNode, out: &mut Vec<SliderBind>) {
    if node.kind == splash_render::NodeKind::Slider {
        if let Some(k) = node.attrs.key.as_deref() {
            let lo = node.attrs.min.unwrap_or(0.0);
            let hi = node.attrs.max.unwrap_or(1.0);
            out.push(SliderBind {
                key: k.to_string(),
                lo,
                hi,
                step: node.attrs.step,
                last: node.attrs.value.unwrap_or(lo),
            });
        }
    }
    for c in &node.children {
        collect_sliders(c, out);
    }
}

script_mod! {
    use mod.prelude.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(440, 1400)
                body +: {
                    flow: Down
                    ScrollYView{
                        width: Fill
                        height: Fill
                        flow: Down
                        show_bg: true
                        draw_bg +: { color: #101417ff }
                        // Upstream `Splash` always allocates an isolate VM. The
                        // material kit needs the mount on the app's MAIN VM (light
                        // theme + shared heap live there), which requires the one
                        // upstream PR: a `Splash` main-VM-mount option (the
                        // `isolate: false` field this fork added). Until then the
                        // kit mounts on an isolate (dark-default theme).
                        host := Splash{ width: Fill, height: Fit }
                    }
                    // Routing signal the mounted kit writes; the app reads it each frame.
                    nav_signal := Label{ text: "" height: 0 draw_text.text_style.font_size: 1 }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    next_frame: NextFrame,
    #[rust]
    last_src: String,
    #[rust]
    screen: String,
    #[rust]
    count: u32,
    #[rust]
    sel_tab: String,
    #[rust]
    sel_seg: String,
    #[rust]
    sel_date: String,
    /// Every slider on the mounted screen: state key, range, and the value last
    /// written. Collected from the semantic tree, so the range is the DSL's, not
    /// the 0..1 the native widget is emitted with.
    #[rust]
    sliders: Vec<SliderBind>,
    /// A slider moved and the screen has not been rebuilt for it yet.
    #[rust]
    slider_dirty: bool,
    /// Frames since the last slider action, so a drag that simply stops still
    /// redraws even if no touch-up arrives.
    #[rust]
    slider_idle: u32,
    #[rust]
    snack: bool,
    #[rust]
    dark: bool,
    #[rust]
    open: String,
    #[rust]
    tick: u32,
    #[rust]
    started: bool,
}

impl App {
    /// The screen's top app bar.
    ///
    /// Every reference screen sits under a real `MaterialToolbar` with a back
    /// arrow and the screen's title. Without it this catalog's content sat ~64dp
    /// higher than the reference's on every single route, which is most of the
    /// residual difference between the two even where the content matches.
    fn with_toolbar(
        body: splash_render::UiNode,
        r: &splash_makepad::material::Roles,
        route: &str,
    ) -> splash_render::UiNode {
        use splash_render::{Attrs, NodeKind, UiNode};
        let n = |kind| UiNode { kind, attrs: Attrs::default(), children: Vec::new() };
        // The reference's own screen titles.
        let title = screens::title_of(route);

        // Aligned to the reference: title 69px against its 67, body 228 against
        // 229. This was wrong until the swatch-group gap and DP_SCALE went in —
        // the 12px offset had been cancelling drift further down the screen, and
        // removing it alone cost two routes. With the drift itself fixed the
        // alignment pays: 39 routes under 9.0 against 38, and `color` crosses.
        //
        // Historical, now superseded — NOT aligned, deliberately — re-tested after the
        // DP_SCALE fix removed the drift this was originally cancelling.
        // A *half* correction (7.5dp spacer, 54.3dp bar) was tried too and is
        // also worse on all three: color 9.4 -> 10.2. Any upward shift costs
        // them, so this is not a matter of finding the right amount.
        //
        // Re-tested twice more, after DP_SCALE and again after tuning it so the
        // swatch pitch matches exactly: aligning still costs two routes (38 under
        // 9.0 -> 36) and `color` 9.4 -> 12.1. Aligning helps the mean and
        // pushes the three routes still over 9.0 *further* out (color 9.8 ->
        // 10.9, allcomponents 12.0 -> 13.0). Those are the longest screens:
        // shifting content up pulls 12px more of the *bottom* into the viewport,
        // and the bottom is where cumulative error is worst. Since the target is
        // per-route, the offset stays.
        // Solving for both (a 9.6dp
        // spacer with a 50dp bar) does put the title at 69px against its 67 and
        // the body at 228 against its 229 — and scores *worse*: `color` 11.5 ->
        // 15.7, routes under 9.0 drop 38 -> 35. The 12px the body sits low is
        // cancelling an opposite drift further down the screen, where a stated
        // 56dp row renders 157px here against the reference's 158. That is a
        // dp->px rounding difference (2.80 vs 2.82) and it accumulates ~8px over
        // a screen of twelve swatches. Until that is addressed the offset is
        // load-bearing, so the spacer stays at zero.
        //
        // Solving from two measurements, not one knob: the title sits 6px above
        // the reference's while the body sits 12px below it, so the bar has to
        // move *down* while the content moves *up*. A bare height change cannot
        // do that — shortening it drags the title into the status bar (at 60dp
        // the title overlaps the system clock) and lengthening it pushes the
        // body further down. A spacer above the bar plus a shorter bar does:
        // title = S + H/2 - textH/2 and body = S + H + pad, so S=3dp, H=57dp.
        let mut top_inset = n(NodeKind::Column);
        // 20 + 39.6, not 9.6 + 50. The sum is what places the body (S + H), so
        // holding it at 59.6 keeps every screen exactly where it was; raising S
        // and shrinking H by the same amount only moves the bar's contents down,
        // by half the change. That is the headroom: the arrow used to span
        // y72-118 against a status bar whose frame reaches y80, so its top 9px
        // were both cramped and untappable (taps there open the shade).
        top_inset.attrs.h = Some(20.0);
        let mut bar = n(NodeKind::Row);
        // 64. Shortening it moves all content up, which the mean likes — 62
        // scores 5.82 against 64's 6.22 — but it pushes the three routes still
        // above 9.0 *further* out (color 11.5 -> 12.4, allcomponents 12.0 ->
        // 12.7), and 60 drops three more routes back over the line entirely.
        // The mean is not the target; per-route is.
        //
        // The gap below this bar runs ~18px wider than the reference's, but
        // that offset is not uniform across routes: taking it off the bar height
        // (58) helps a dozen screens and costs a dozen others — 37 routes under
        // 9.0 drops to 35. The page padding that would absorb it cannot shrink
        // without pulling the horizontal inset along (tried five ways).
        bar.attrs.h = Some(BAR_H);
        bar.attrs.padx = Some(4.0);
        // 10, not 8: with the back target 48dp wide from x4, this lands the
        // title at 62dp -- the reference's x174.
        bar.attrs.spacing = Some(10.0);
        bar.attrs.aligny = Some(0.5);
        bar.attrs.fillw = Some(1);
        let mut back = n(NodeKind::Text);
        back.attrs.text = Some("\u{f060}".to_string()); // arrow-left
        back.attrs.icon = Some(1);
        back.attrs.size = Some(20.0);
        back.attrs.color = Some(r.on_surface);
        back.attrs.margin = Some(12.0);
        back.attrs.pady = Some(0.0);
        // Wrapped, because `needs_click_overlay` only fires for a container --
        // `tapto` on the Text itself is silently ignored, which is why this
        // arrow was decorative. No padding on the wrapper: the arrow's `margin`
        // is already inert here (margin is emitted through the object form the
        // dialect drops), so hugging keeps the bar's geometry exactly as the
        // toolbar alignment was derived against.
        // A real 48dp touch target, not a hug around a 16dp glyph. The wrapper
        // used to hug: 47x56px, about a third of Material's minimum, and its
        // top edge fell under the status bar (which occupies y 0-80 on this
        // device and swallows taps there) -- so back "worked" only if you hit a
        // sliver. Centring the glyph in 48dp also puts it where the reference
        // draws it: measured, the reference's arrow spans x57-101 and its title
        // starts at x174, where these were at x11 and x88. The toolbar's
        // *horizontal* geometry had never been matched -- only its vertical --
        // and a glyph that small cannot move a 120x250 score.
        let mut back_hit = n(NodeKind::Row);
        back_hit.attrs.w = Some(48.0);
        // The bar's own height, not 48: a target taller than the bar overflows
        // it and its centred glyph lands below the title. The whole of it is
        // tappable now that the bar clears the status bar, so 39.6dp of height
        // is worth more than a nominal 48 that misaligns the arrow.
        back_hit.attrs.h = Some(BAR_H);
        back_hit.attrs.alignx = Some(0.5);
        back_hit.attrs.aligny = Some(0.5);
        back_hit.attrs.tapto = Some(screens::INDEX.to_string());
        back_hit.children.push(back);
        bar.children.push(back_hit);
        let mut t = n(NodeKind::Text);
        t.attrs.text = Some(title.to_string());
        t.attrs.size = Some(22.0);
        // Opt out of the backend's per-text vertical inset — that stands in for
        // Android's TextView font padding inside content, but the bar has its own
        // fixed height and the extra pushed `topappbar` from 8.7 to 9.2. An
        // explicit 0 reads as "stated", where None would take the default.
        t.attrs.pady = Some(0.0);
        t.attrs.color = Some(r.on_surface);
        t.attrs.fillw = Some(1);
        bar.children.push(t);

        let mut page = n(NodeKind::Column);
        page.attrs.fillw = Some(1);
        page.children.push(top_inset);
        page.children.push(bar);
        page.children.push(body);
        page
    }

    /// Stack a real modal over the screen when a slot asks for one.
    ///
    /// The reference's screens do not contain their dialogs, sheets or pickers:
    /// a button writes `key=action` and the *host* builds a real
    /// `MaterialAlertDialogBuilder` / `BottomSheetDialog` / `MaterialDatePicker`.
    /// This is that host half — without it the triggers are inert and the
    /// modal-hosting screens are pictures of buttons.
    fn with_modal(
        screen: splash_render::UiNode,
        r: &splash_makepad::material::Roles,
    ) -> splash_render::UiNode {
        use splash_render::{Attrs, NodeKind, UiNode};
        let n = |kind| UiNode { kind, attrs: Attrs::default(), children: Vec::new() };
        let text = |s: &str, size: f32, w: i32, c: u32| {
            let mut t = n(NodeKind::Text);
            t.attrs.text = Some(s.to_string());
            t.attrs.size = Some(size);
            t.attrs.weight = Some(w);
            t.attrs.color = Some(c);
            t.attrs.fillw = Some(1);
            t
        };

        // Which slot is open, and what it asked for.
        let open = ["dlg", "bs", "ss", "menu", "dp", "tp", "snk"]
            .into_iter()
            .find_map(|k| {
                let v = state_get(k);
                (!v.is_empty()).then(|| (k, v))
            });
        let Some((slot, action)) = open else {
            return screen;
        };

        // What each family shows. The reference opens a different real widget per
        // action — six dialog shapes, three sheets, three pickers — so a single
        // generic card would be a picture of a modal rather than the modal.
        let (title, body, choices, bottom, wide) = match (slot, action.as_str()) {
            ("dlg", "icon") => ("Use location?", "This app will access your location while in use.", &[][..], false, false),
            ("dlg", "single") => ("Choose one", "", &["Never", "Every 15 minutes", "Every hour"][..], false, false),
            ("dlg", "multi") => ("Choose any", "", &["Email", "SMS", "Push"][..], false, false),
            ("dlg", "long") => ("Terms", "This message is deliberately long so the dialog scrolls. It repeats to fill the body and demonstrate that a long dialog keeps its actions pinned below the scrolling region.", &[][..], false, false),
            ("dlg", "full") => ("Full-screen dialog", "A full-screen dialog covers the whole surface and carries its own app bar.", &[][..], false, true),
            ("dlg", _) => ("Reset settings?", "This will restore default values for every preference in this app.", &[][..], false, false),
            ("bs", "list") => ("Bottom sheet", "", &["Share", "Get link", "Edit name", "Delete"][..], true, true),
            ("bs", "tall") => ("Expanded sheet", "A tall sheet expanded to most of the screen.", &[][..], true, true),
            ("bs", _) => ("Bottom sheet", "Drag the handle to expand, or tap outside to dismiss.", &[][..], true, true),
            ("ss", _) => ("Side sheet", "A sheet anchored to the edge of the screen.", &[][..], false, false),
            ("menu", _) => ("", "", &["Refresh", "Settings", "Sign out"][..], false, false),
            ("dp", "range") => ("Select range", "1 Mar 2026 – 14 Mar 2026", &[][..], false, false),
            ("dp", "input") => ("Enter date", "03 / 11 / 2026", &[][..], false, false),
            ("dp", _) => ("Select date", "Wed, 11 March 2026", &[][..], false, false),
            ("tp", "input") => ("Enter time", "10 : 30", &[][..], false, false),
            ("tp", _) => ("Select time", "10 : 30", &[][..], false, false),
            _ => ("Message archived", "", &["Undo"][..], true, true),
        };

        let mut card = n(NodeKind::Column);
        card.attrs.bg = Some(r.surf_high);
        card.attrs.radius = Some(28.0);
        card.attrs.pad = Some(24.0);
        card.attrs.spacing = Some(16.0);
        card.attrs.w = Some(if wide { 360.0 } else { 320.0 });
        if !title.is_empty() {
            card.children.push(text(title, 24.0, 400, r.on_surface));
        }
        if !body.is_empty() {
            card.children.push(text(body, 14.0, 400, r.on_surface_variant));
        }
        for choice in choices {
            let mut row = n(NodeKind::Row);
            row.attrs.h = Some(48.0);
            row.attrs.spacing = Some(12.0);
            row.attrs.aligny = Some(0.5);
            row.attrs.fillw = Some(1);
            let mut mark = n(NodeKind::Column);
            mark.attrs.w = Some(20.0);
            mark.attrs.h = Some(20.0);
            mark.attrs.radius = Some(10.0);
            mark.attrs.border = Some(2.0);
            mark.attrs.bordercolor = Some(r.on_surface_variant);
            row.children.push(mark);
            row.children.push(text(choice, 16.0, 400, r.on_surface));
            card.children.push(row);
        }
        let mut actions = n(NodeKind::Row);
        actions.attrs.spacing = Some(8.0);
        actions.attrs.aligny = Some(0.5);
        // Align the pair to the trailing edge rather than pushing them with a
        // Fill spacer — the spacer claimed the whole row and the buttons never
        // appeared.
        actions.attrs.alignx = Some(1.0);
        actions.attrs.h = Some(40.0);
        for (label, keep) in [("Cancel", false), ("OK", true)] {
            let mut b = n(NodeKind::Column);
            b.attrs.h = Some(40.0);
            b.attrs.padx = Some(12.0);
            b.attrs.fitw = Some(1);
            b.attrs.alignx = Some(0.5);
            b.attrs.aligny = Some(0.5);
            b.attrs.radius = Some(20.0);
            // Give the pair a tonal container. Transparent text buttons were not
            // drawing inside the card at all, and a modal you cannot see your way
            // out of is worse than a slightly heavier one.
            b.attrs.bg = Some(if keep { r.primary } else { r.surf_highest });
            // Either button dismisses; OK also records a result, as the
            // reference's "Last result:" caption expects.
            b.attrs.tapto = Some(if keep {
                format!("modal:{slot}=ok")
            } else {
                format!("modal:{slot}=")
            });
            let mut lab = text(
                label,
                14.0,
                500,
                if keep { r.on_primary } else { r.on_surface },
            );
            // A Fill label inside a hug-content button resolves to zero width and
            // the button collapses to a sliver. Let it size to its text.
            lab.attrs.fillw = None;
            lab.attrs.fitw = Some(1);
            b.attrs.padx = Some(20.0);
            b.children.push(lab);
            actions.children.push(b);
        }
        card.children.push(actions);
        let _ = action;

        // Scrim + card. A Fill scrim would collapse under a Fit mount, so it is
        // given the viewport height explicitly — the same trick the kit uses.
        let mut scrim = n(NodeKind::Column);
        scrim.attrs.bg = Some(0x99000000);
        scrim.attrs.fillw = Some(1);
        scrim.attrs.h = Some(820.0);

        let mut layer = n(NodeKind::Column);
        layer.attrs.fillw = Some(1);
        layer.attrs.h = Some(820.0);
        layer.attrs.alignx = Some(0.5);
        layer.attrs.aligny = Some(if bottom { 1.0 } else { 0.5 });
        layer.attrs.pad = Some(16.0);
        // Dismiss on tap. It has to go on the *layer*, not the scrim: the layer
        // is stacked above and was swallowing every tap, so a scrim handler never
        // fired and the dialog had no way out. Both dialog actions dismiss too,
        // so catching them here as well is harmless.
        layer.attrs.tapto = Some(format!("modal:{slot}="));
        layer.children.push(card);

        let mut stack = n(NodeKind::Stack);
        stack.attrs.fillw = Some(1);
        stack.children.push(screen);
        stack.children.push(scrim);
        stack.children.push(layer);
        stack
    }

    /// The route to draw: whatever the sweep last wrote, else the first screen.
    fn current_route() -> String {
        std::fs::read_to_string(ROUTE_PATH)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "button".to_string())
    }

    /// The screen source. A pushed file still wins, so a single screen can be
    /// iterated on without a rebuild.
    ///
    /// In-app navigation takes precedence over the route file. It has to: a tap
    /// sets `self.screen`, but this used to render `current_route()` and so read
    /// the file every time, which meant no tap could ever change the screen --
    /// the back arrow and every other nav target were inert for that reason as
    /// much as for having no handler.
    fn current_source(&self) -> String {
        if let Ok(pushed) = std::fs::read_to_string(DEVICE_PATH) {
            return pushed;
        }
        if screens::has(&self.screen) {
            return screens::source_for(&self.screen);
        }
        screens::source_for(&Self::current_route())
    }

    /// Translate + mount the active screen: inject app state as one `st = {…}`
    /// object, run the splash pipeline, feed makepad's dialect into the host.
    fn mount(&mut self, cx: &mut Cx) {
        let src = self.current_source();
        let route = if self.screen.is_empty() {
            Self::current_route()
        } else {
            self.screen.clone()
        };
        let route = route.as_str();
        let tab = if self.sel_tab.is_empty() { "overview" } else { &self.sel_tab };
        let seg = if self.sel_seg.is_empty() { "day" } else { &self.sel_seg };
        let date = if self.sel_date.is_empty() { "11" } else { &self.sel_date };
        let count = self.count;
        let snack = if self.snack { 1 } else { 0 };
        let dark = if self.dark { 1 } else { 0 };
        let open = self.open.as_str();
        let full = format!(
            "let st = {{ route: {:?}, count: {}, tab: {:?}, seg: {:?}, date: {:?}, snack: {}, dark: {}, open: {:?} }}\n{}",
            route, count, tab, seg, date, snack, dark, open, src
        );
        // `adaptive` prints its window size class from this slot, and `S()` is
        // read *during* evaluation — so this has to be seeded before build(),
        // not after. Unset, the card showed a blank line where the reference
        // reads "Compact (384dp)". Stated as a constant for the same reason the
        // backing layer's 820dp height is: this host is pinned to one device.
        if state_get("win_class").is_empty() {
            state_set("win_class", "Compact (384dp)");
        }
        if let Some(node) = splash_render::build(&full, register_state) {
            // Every reference screen is rooted in its own `scroll`. This shell
            // already provides one, and the mounted `Splash` is height:Fit — so a
            // nested Fill scroll resolves to zero height and the screen renders
            // blank. Unwrap it and let the shell's scroller do the scrolling.
            let node = match node.kind {
                splash_render::NodeKind::Scroll if node.children.len() == 1 => {
                    node.children.into_iter().next().unwrap()
                }
                _ => node,
            };
            // Re-collected on every mount: the set of sliders is a property of
            // the screen, and the committed value has to track what was just
            // drawn or the next frame reads a stale `last` and re-mounts forever.
            self.sliders.clear();
            collect_sliders(&node, &mut self.sliders);
            // Android's theme paints the Material surface behind a screen, so the
            // reference screens carry no background of their own. Here the shell
            // is that theme: wrap the mount in the surface role, or every screen
            // draws its ink onto makepad's default grey.
            let roles = splash_makepad::material::Roles::reference_dark();
            // Toolbar *outside* the padded page, modal outside that. Applying the
            // page's 16dp above the bar pushed it 39px below the reference's
            // (title at y106 against y67) while squeezing the bar-to-content gap
            // to 74px against 101 — two errors that partly cancelled and left
            // every screen's content 12px low.
            let mut page = splash_render::UiNode {
                kind: splash_render::NodeKind::Column,
                attrs: splash_render::Attrs::default(),
                children: vec![node],
            };
            // Keep the wrapper's own padding and rhythm. Removing it on the
            // theory that the screens' own `pad: 16, pady: 8` was being doubled
            // made every route measurably worse (10.2 -> 11.0, `color` 19 -> 33):
            // the offset is not double padding, and the spacing carries weight.
            // SETTLED — do not probe again. Splitting these axes has been tried
            // at 16/8 (root and emitter), 16/12.4, and symmetric 10 and 13. All
            // are worse across the sweep; 16 on both is the optimum. The 10px by
            // which content sits below the reference's is *not* this padding:
            // reducing it moved `carousel` the wrong way (10.8 -> 11.5).
            //
            // The screens' own `pad: 16, pady: 8` never lands — it is asymmetric,
            // so it hits the inert object form and is dropped whole. This
            // wrapper stands in for it, and 16 on both axes is right: the
            // reference evidently drops `pady` as well. Reproducing 16/8 here
            // was tried, at the root and in the emitter, and cost 2 points of
            // mean across the sweep (it fixes `slider` 9.8 -> 3.3 and loses more
            // on nine others), so the reference is not honouring it either.
            //
            // 16 put every label 6dp right of the reference's and 17px below it
            // (measured: text at x65/y245 against x48/y228); 10 lands on both.
            // Keep the *scalar* form — makepad honours `padding: 10` here but
            // not the per-side object `padding: {left: .., top: ..}`.
            //
            // The 6dp is makepad's `Label` default padding, so zeroing that and
            // restoring 16 here looks equivalent and is not: it costs ~2 points
            // of mean pixel difference across the 41 routes (8.78 -> 10.64, and
            // 11.86 with 10). That default is carrying spacing the reference
            // also has. Measured, not reasoned — don't re-derive it.
            // SETTLED. Splitting this into 16dp horizontal / smaller vertical has
            // now been tried five ways — 16/8 (emitter and root), 16/12.4,
            // symmetric 10 and 13, and 10+6 once the toolbar moved outside this
            // padding, which removed the confound that explained the earlier
            // failures. Every one costs 1.5-2 points of mean across the sweep.
            // It reliably fixes `slider` (9.7 -> 4.6) and reliably breaks a dozen
            // others. 16 on both axes is the optimum; do not probe again.
            page.attrs.pad = Some(16.0);
            page.attrs.spacing = Some(16.0);
            page.attrs.fillw = Some(1);
            // The surface has to cover the viewport, not just the content. A Fill
            // height collapses under the Fit-height Splash mount, so the backing
            // layer is given the viewport height explicitly and the content sits
            // on top — otherwise every short screen showed makepad's grey below
            // the last widget where the reference shows unbroken surface.
            let mut backing = splash_render::UiNode {
                kind: splash_render::NodeKind::Column,
                attrs: splash_render::Attrs::default(),
                children: Vec::new(),
            };
            backing.attrs.bg = Some(roles.surface);
            backing.attrs.fillw = Some(1);
            backing.attrs.h = Some(820.0);
            // `route`, not the file: after a tap the two disagree, and the
            // toolbar kept titling the index "Button".
            let page = Self::with_toolbar(page, &roles, route);
            let page = Self::with_modal(page, &roles);
            let mut stack = splash_render::UiNode {
                kind: splash_render::NodeKind::Stack,
                attrs: splash_render::Attrs::default(),
                children: vec![backing, page],
            };
            stack.attrs.fillw = Some(1);
            let node = stack;
            let ui = splash_makepad::to_makepad_ui(&node);
            // Mount on the app's MAIN VM.
            //
            // `Splash::set_text` allocates an isolate that only ever receives
            // makepad's own `script_mod`, which is why this crate's Roboto could
            // not resolve (every label blanked) and why splash-widgets' M3
            // control theming never applied. Evaluating here with `cx.with_vm`
            // and handing the built `View` to the host widget keeps both in
            // reach. Taps survive because the handler now calls the `NAV` global
            // rather than `ui.nav_signal`, which only exists inside an isolate.
            let code = format!("use mod.prelude.widgets.*\nView{{height:Fit, {ui}");
            let script_mod = ScriptMod {
                cargo_manifest_path: env!("CARGO_MANIFEST_DIR").to_string(),
                module_path: module_path!().to_string(),
                file: file!().to_string(),
                line: 1,
                column: 0,
                code: String::new(),
                values: Vec::new(),
            };
            let built = cx.with_vm(|vm| {
                let value = vm.eval_with_append_source(script_mod, &code, ScriptValue::NIL.into());
                (!value.is_err() && !value.is_nil())
                    .then(|| View::script_from_value(vm, value))
            });
            if let Some(view) = built {
                if let Some(mut host) = self.ui.widget(cx, ids!(host)).borrow_mut::<Splash>() {
                    host.view = view;
                }
                cx.redraw_all();
            }
            // Only once the build succeeded. Marking the source seen up front let a
            // half-written file (`adb push` is not atomic) fail to build, count as
            // seen, and never be retried — that route then stayed frozen on the
            // previous screen until the file changed again.
            self.last_src = src;
        }
    }
}

/// The widget state the screens read through `S`/`N` and write through a tap.
///
/// This is what makes the catalog a demo rather than a picture: a tap writes a
/// slot, the DSL is re-evaluated, and the new tree rebuilds the widgets — the
/// DSL decides what the screen says, exactly as the reference does it.
static STATE: std::sync::Mutex<Option<std::collections::HashMap<String, String>>> =
    std::sync::Mutex::new(None);

fn state_get(key: &str) -> String {
    STATE
        .lock()
        .ok()
        .and_then(|m| m.as_ref().and_then(|m| m.get(key).cloned()))
        .unwrap_or_default()
}

fn state_set(key: &str, value: &str) {
    if let Ok(mut m) = STATE.lock() {
        m.get_or_insert_with(Default::default)
            .insert(key.to_string(), value.to_string());
    }
}

/// Taps land here. The mounted body's `on_click` calls `NAV(t: "…")`, a global
/// registered below, instead of reaching through `ui.nav_signal` — `ui` is only
/// injected inside a `Splash` isolate, so the old handler silently did nothing
/// on any other VM.
static TAPS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn take_tap() -> Option<String> {
    TAPS.lock().ok().and_then(|mut q| if q.is_empty() { None } else { Some(q.remove(0)) })
}

/// `S(key)` -> the slot as a string (""" when unset); `N(key, dflt)` -> its number.
fn register_state(vm: &mut ScriptVm) {
    let f_s = splash_render::add_global_fn(vm, &[(live_id!(k), ScriptValue::NIL)], |vm, a| {
        let k = splash_render::string_prop(vm, a, live_id!(k)).unwrap_or_default();
        let v = state_get(&k);
        vm.bx.heap.new_string_from_str(&v)
    });
    vm.set_injected_global(live_id!(S), f_s);

    let f_n = splash_render::add_global_fn(
        vm,
        &[(live_id!(k), ScriptValue::NIL), (live_id!(d), ScriptValue::NIL)],
        |vm, a| {
            let k = splash_render::string_prop(vm, a, live_id!(k)).unwrap_or_default();
            let d = splash_render::num_prop(vm, a, live_id!(d)).unwrap_or(0.0);
            let v = state_get(&k);
            ScriptValue::from_f64(v.trim().parse::<f64>().unwrap_or(d))
        },
    );
    vm.set_injected_global(live_id!(N), f_n);

    let f_nav = splash_render::add_global_fn(vm, &[(live_id!(t), ScriptValue::NIL)], |vm, a| {
        let t = splash_render::string_prop(vm, a, live_id!(t)).unwrap_or_default();
        if let Ok(mut q) = TAPS.lock() {
            q.push(t);
        }
        ScriptValue::NIL
    });
    vm.set_injected_global(live_id!(NAV), f_nav);
}

impl MatchEvent for App {
    /// Drive the drawn track from the invisible native slider under it.
    ///
    /// The value arrives as `<index>.<fraction>`: a widget inside a mounted
    /// Splash body cannot be found by id from out here -- `widget_flood` returns
    /// nothing -- but its actions do escape, so the Material lowering gives each
    /// slider its own unit band and the index rides out in the one field that
    /// travels. See `sl_native`.
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let mut moved = false;
        for a in actions {
            let Some(wa) = a.downcast_ref::<WidgetAction>() else {
                continue;
            };
            let Some(sa) = wa.action.downcast_ref::<SliderAction>() else {
                continue;
            };
            // Never rebuild mid-gesture: re-mounting on every `Slide` does move
            // the drawn handle, but it destroys and recreates the very widget
            // the finger has captured, so the drag dies after one frame --
            // measured, a 480px swipe moved the handle 8px. The state slot is
            // written throughout and the redraw is deferred to touch-up.
            //
            // Not `EndSlide`: it never arrives for a synthetic swipe (measured
            // -- every action in a 700ms drag was a `Slide`), and trusting it
            // meant the handle never moved at all.
            let v = match sa {
                SliderAction::Slide(v) | SliderAction::TextSlide(v) | SliderAction::EndSlide(v) => {
                    *v
                }
                _ => continue,
            };
            // Bands sit two apart -- see `sl_native`.
            let idx = (v / 2.0).floor().max(0.0) as usize;
            let Some(b) = self.sliders.get_mut(idx) else {
                continue;
            };
            let frac = (v - (idx * 2) as f64).clamp(0.0, 1.0) as f32;
            let mut val = b.lo + frac * (b.hi - b.lo);
            if let Some(st) = b.step.filter(|s| *s > 0.0) {
                val = (val / st).round() * st;
            }
            // A tolerance, not equality: the value will not survive the round
            // trip through a text state slot unchanged, and an exact test would
            // re-mount on every frame of a drag that is standing still.
            if (val - b.last).abs() <= (b.hi - b.lo).abs() * 0.001 {
                continue;
            }
            b.last = val;
            // Match the granularity the screen already shows: a 0..100 slider
            // captions its start value as "50", so reporting "82.66" back reads
            // as a different kind of number.
            let whole = b.step.is_some_and(|s| s >= 1.0) || (b.hi - b.lo).abs() >= 20.0;
            let txt = if whole {
                format!("{}", val.round() as i64)
            } else {
                format!("{val:.2}")
            };
            state_set(&b.key, &txt);
            moved = true;
        }
        if moved {
            self.slider_dirty = true;
            self.slider_idle = 0;
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::theme_mod(vm);
        script_eval!(vm, {
            // The body now mounts on this VM, so its theme is the one every
            // native control resolves against. It has to match the surface the
            // host paints — with the light theme the control labels came out
            // near-black on the dark M3 surface.
            mod.theme = mod.themes.dark
        });
        // Fork-free themed widgets (Material 3), against upstream makepad.
        splash_widgets::widgets_mod(vm);
        // `S`/`N`/`NAV` on the **app** VM as well as the build VM. The mounted
        // body evaluates here (`cx.with_vm`), so without this its `on_click`
        // handlers raise "variable NAV not found in scope" and every tap in the
        // catalog is silently dead — which is exactly what was happening.
        register_state(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if matches!(event, Event::Startup) && !self.started {
            self.started = true;
            // The semantic components resolve their Material roles in the
            // backend, so the scheme is chosen there. The reference catalog
            // renders dark; match it.
            // Match the reference's own scheme, not the M3 baseline: that device
            // runs Material You, so a baseline-purple render can only ever be
            // compared structurally.
            // Put `SplashTap` in reach of every mounted body. A Splash isolate
            // receives makepad's own script mods and nothing else, so a widget
            // type defined out here is invisible to a body -- which is why the
            // tap target had to be a `Button`, and why a tappable row starved
            // the scroll: `Button::handle_event` captures the finger on
            // touch-down. `SplashTap` never calls `event.hits`, so the scroll
            // sees the whole gesture. Must run before the first mount, since an
            // isolate takes its mods at allocation.
            makepad_widgets::widget_async::register_splash_isolate_mod(register_tap_mod);
            splash_makepad::set_scheme(splash_makepad::material::Roles::reference_dark());
            // The width a wrapping row packs into. Left at its 340dp default the
            // flow wrapped a row early — the content is 354dp here (measured:
            // x45-1034), and the shortfall is what made the button screen drop
            // its third button to a second line whenever the buttons widened.
            splash_makepad::material::set_flow_width(354.0);
            // The sweep's route, not `home` -- which was never a route at all, so
            // `source_for` fell through to the first screen and drew `adaptive`
            // under the literal title "home".
            self.screen = Self::current_route();
            self.next_frame = cx.new_next_frame();
            self.mount(cx);
        }
        if self.next_frame.is_event(event).is_some() {
            self.tick = self.tick.wrapping_add(1);
            let nav_raw = take_tap().unwrap_or_default();
            let nav = nav_raw.trim();
            if !nav.is_empty() {
                // `set:<key>=<value>` — a widget writing its state slot. This is
                // the round trip the reference verifies: the tap changes state,
                // the DSL is re-evaluated, and the screen says something new.
                if let Some(kv) = nav.strip_prefix("modal:") {
                    if let Some((k, v)) = kv.split_once('=') {
                        state_set(k, v);
                        if !v.is_empty() {
                            state_set(&format!("{k}_result"), v);
                        }
                    }
                    self.mount(cx);
                    self.next_frame = cx.new_next_frame();
                    cx.redraw_all();
                    return;
                }
                if let Some(kv) = nav.strip_prefix("set:") {
                    if let Some((k, v)) = kv.split_once('=') {
                        state_set(k, v);
                    }
                    self.mount(cx);
                    self.next_frame = cx.new_next_frame();
                    cx.redraw_all();
                    return;
                }
                if nav == "act:count" {
                    self.count = self.count.wrapping_add(1);
                } else if let Some(v) = nav.strip_prefix("tab:") {
                    self.sel_tab = v.to_string();
                } else if let Some(v) = nav.strip_prefix("seg:") {
                    self.sel_seg = v.to_string();
                } else if let Some(v) = nav.strip_prefix("date:") {
                    self.sel_date = v.to_string();
                } else if nav == "snack:show" {
                    self.snack = true;
                } else if nav == "snack:hide" {
                    self.snack = false;
                } else if nav == "theme:toggle" {
                    self.dark = !self.dark;
                } else if let Some(v) = nav.strip_prefix("open:") {
                    self.open = v.to_string();
                } else {
                    self.screen = nav.to_string();
                    self.open = String::new();
                }
                self.mount(cx);
            } else if self.slider_dirty && {
                self.slider_idle += 1;
                self.slider_idle > 8
            } {
                // Rebuild once the drag has been still for ~8 frames, rather
                // than on touch-up: `EndSlide` never arrives for a synthetic
                // swipe, and `TouchState` is not re-exported to reach for the
                // raw event. Settling on idle covers a real lift and a stalled
                // drag alike, and never fires while the finger is still moving,
                // which is what would kill the gesture.
                self.slider_dirty = false;
                self.slider_idle = 0;
                self.mount(cx);
            } else if self.tick % 20 == 0 && self.current_source() != self.last_src {
                self.mount(cx);
            }
            cx.redraw_all();
            self.next_frame = cx.new_next_frame();
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
