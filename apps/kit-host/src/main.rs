//! kit-host — a generic app shell that renders a Splash **component kit** as
//! native makepad widgets, against **upstream** makepad. It bakes one kit's
//! `.splash` (Material 3 here) and mounts it via the splash pipeline; native
//! controls are themed fork-free by `splash_widgets` (no makepad fork).
//!
//! Swap the baked kit (or push to the device path) to render iOS / liquid-glass.

pub use makepad_widgets;
use makepad_widgets::*;

app_main!(App);

// The Material 3 component library, straight from the repo's components/ dir.
const BAKED: &str = include_str!("../../../components/material/catalog.splash");
const DEVICE_PATH: &str = "/data/local/tmp/kit_host.splash";

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
                        draw_bg +: { color: #fef7ffff }
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
    fn current_source() -> String {
        std::fs::read_to_string(DEVICE_PATH).unwrap_or_else(|_| BAKED.to_string())
    }

    /// Translate + mount the active screen: inject app state as one `st = {…}`
    /// object, run the splash pipeline, feed makepad's dialect into the host.
    fn mount(&mut self, cx: &mut Cx) {
        let src = Self::current_source();
        self.last_src = src.clone();
        let route = if self.screen.is_empty() { "home" } else { &self.screen };
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
        if let Some(node) = splash_render::build(&full, |_vm| {}) {
            let ui = splash_makepad::to_makepad_ui(&node);
            self.ui.widget(cx, ids!(host)).set_text(cx, &ui);
        }
    }
}

impl MatchEvent for App {}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::theme_mod(vm);
        script_eval!(vm, {
            mod.theme = mod.themes.light
        });
        // Fork-free themed widgets (Material 3), against upstream makepad.
        splash_widgets::widgets_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if matches!(event, Event::Startup) && !self.started {
            self.started = true;
            self.screen = "home".to_string();
            self.next_frame = cx.new_next_frame();
            self.mount(cx);
        }
        if self.next_frame.is_event(event).is_some() {
            self.tick = self.tick.wrapping_add(1);
            let nav_raw = self.ui.widget(cx, ids!(nav_signal)).text();
            let nav = nav_raw.trim();
            if !nav.is_empty() {
                self.ui.widget(cx, ids!(nav_signal)).set_text(cx, "");
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
            } else if self.tick % 20 == 0 && Self::current_source() != self.last_src {
                self.mount(cx);
            }
            cx.redraw_all();
            self.next_frame = cx.new_next_frame();
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
