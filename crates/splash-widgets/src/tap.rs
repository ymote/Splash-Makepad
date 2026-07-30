//! `SplashTap` — a tap target that does not steal the scroll.
//!
//! ## Why this exists
//!
//! The kit needs a whole row to be tappable: `tapto: "date_planner/maya"` on a
//! row has to navigate. The DSL can only attach a handler to a widget that
//! exposes one to script, and in makepad that is `Button` — a `View` has only
//! `on_render`. So the emitter wrapped every tappable container in an Overlay
//! with a transparent `Button` filling it.
//!
//! That made the catalog unusable. `Button::handle_event` calls
//! `event.hits(cx, self.draw_bg.area())`, which *captures* the finger on
//! touch-down. `ScrollBar::handle_touch_based_drag` then calls
//! `event.hits(cx, scroll_area)`, gets nothing because the Button holds the
//! capture, and never enters `ScrollState::Drag`. Since the rows cover nearly
//! the whole screen, a swipe anywhere over the list did nothing at all.
//!
//! Measured on a OnePlus 6T, index screen, three full-height swipes:
//!
//! | swipe at | pixel rows changed of 2340 |
//! |---|---|
//! | x=540, over the rows | 159 |
//! | x=25, clear of the rows | 1608 |
//!
//! The scroll was never broken. The tap targets were eating the gesture.
//!
//! ## How this avoids it
//!
//! It never calls `event.hits`, so it never registers a hit area and never
//! captures. It reads `Event::TouchUpdate` (and the mouse pair, for desktop)
//! and tests the coordinates against its own rect itself. The scroll therefore
//! sees the entire gesture exactly as if the row were inert.
//!
//! The cost of hit-testing by hand is that this widget must decide for itself
//! what counts as a tap rather than a drag, which upstream's capture would have
//! settled. `TRAVEL_SLOP` is that judgement: a touch that moves further than
//! this between down and up was a scroll, not a tap, and fires nothing. Without
//! it every flick would also navigate.

use makepad_widgets::event::TouchState;
use makepad_widgets::makepad_script::ScriptFnRef;
use makepad_widgets::*;

/// How far a touch may travel and still count as a tap, in logical pixels.
///
/// Android's own `ViewConfiguration` touch slop is 8dp; this is deliberately
/// looser because a finger on a list row moves more than a mouse does, and the
/// failure this guards against — a scroll that also navigates — is much worse
/// than a tap that needs repeating.
const TRAVEL_SLOP: f64 = 12.0;

#[derive(Script, ScriptHook, Widget)]
pub struct SplashTap {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    /// Called on a tap that stayed within `TRAVEL_SLOP`.
    #[live]
    on_click: ScriptFnRef,
    /// Present for its area and its redraw, not to paint anything. The derive
    /// needs a `#[redraw]` field, and a draw object is also what gives this
    /// widget a rect to hit-test against. Its colour is fully transparent.
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    /// The touch that started inside this widget: its id, and where it began.
    #[rust]
    start: Option<(u64, DVec2)>,
}

impl SplashTap {
    fn rect(&self, cx: &mut Cx) -> Rect {
        self.draw_bg.area().rect(cx)
    }

    fn fire(&mut self, cx: &mut Cx) {
        let uid = self.widget_uid();
        cx.widget_to_script_call(uid, NIL, self.source.clone(), self.on_click.clone(), &[]);
    }

    /// Common down/up/move bookkeeping, shared by touch and mouse.
    fn press(&mut self, cx: &mut Cx, abs: DVec2, id: u64) {
        if self.rect(cx).contains(abs) {
            self.start = Some((id, abs));
        }
    }

    fn moved(&mut self, abs: DVec2, id: u64) {
        if let Some((sid, from)) = self.start {
            if sid == id && (abs - from).length() > TRAVEL_SLOP {
                // It became a scroll. Forget it, so the release does nothing.
                self.start = None;
            }
        }
    }

    fn release(&mut self, cx: &mut Cx, abs: DVec2, id: u64) {
        let Some((sid, from)) = self.start else {
            return;
        };
        if sid != id {
            return;
        }
        self.start = None;
        if (abs - from).length() <= TRAVEL_SLOP && self.rect(cx).contains(abs) {
            self.fire(cx);
        }
    }
}

impl Widget for SplashTap {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            // Touch, which is the case that matters on a phone. Deliberately
            // reading the raw event rather than `event.hits`: hits would capture
            // the finger and starve the enclosing scroll.
            Event::TouchUpdate(e) => {
                for t in &e.touches {
                    let abs = dvec2(t.abs.x, t.abs.y);
                    match t.state {
                        TouchState::Start => self.press(cx, abs, t.uid),
                        TouchState::Move => self.moved(abs, t.uid),
                        TouchState::Stop => self.release(cx, abs, t.uid),
                        TouchState::Stable => {}
                    }
                }
            }
            // The desktop pair, so the same screens work under `cargo run`.
            // A mouse has no id; 0 stands in for the single pointer.
            Event::MouseDown(e) => self.press(cx, e.abs, 0),
            Event::MouseMove(e) => self.moved(e.abs, 0),
            Event::MouseUp(e) => self.release(cx, e.abs, 0),
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        // Occupy the space. `draw_bg` is transparent, so this covers the content
        // it sits over without hiding it, and leaves behind the area that
        // `rect` hit-tests against.
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

script_mod! {
    use mod.prelude.widgets.*

    // Registered into both, because a mounted Splash body resolves against the
    // prelude while `mod.widgets` is what other script_mod blocks extend.
    mod.widgets.SplashTap = #(SplashTap::register_widget(vm)){
        draw_bg +: { color: #00000000 }
    }
    mod.prelude.widgets.SplashTap = mod.widgets.SplashTap
}
