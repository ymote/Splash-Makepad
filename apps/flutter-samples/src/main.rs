//! flutter-samples — the flutter/samples catalog as native makepad widgets.
//!
//! Same shell as `kit-host`, pointed at `components/flutter/` instead of the
//! Material kit: assemble the kit, inject `st`, run the Splash pipeline, feed
//! makepad's dialect into the mounted `Splash` widget.
//!
//! Two pieces of state, because that is all the ports need: the current route
//! and the light/dark flag. A screen navigates by setting `tapto`, which emits
//! an `on_click` writing the target into `nav_signal`; this app reads that
//! label each frame and re-mounts.
//!
//! Hot reload: dropping an assembled kit at `DEVICE_PATH` overrides the baked
//! one, so screens can be edited without a rebuild. Assemble one with
//! `cargo run -p splash-makepad --example assemble -- components/flutter`.
//!
//! The kit is baked by `include_str!` rather than generated into `OUT_DIR`,
//! because `cargo-makepad` compiles the app inside a generated wrapper crate
//! that never runs this crate's build script — `OUT_DIR` is undefined there and
//! the Android build fails to compile. A relative `include_str!` resolves
//! against the source file, so it works identically on desktop and on device.
//! `baked_kit_matches_the_directory` below fails if this list drifts from
//! `components/flutter/`.

pub use makepad_widgets;
use makepad_widgets::*;

app_main!(App);

/// One `.splash` per flutter/samples directory, in the order
/// [`splash_makepad::kit`] fixes: `_kit.splash` first (tokens and helpers),
/// the samples sorted, `_index.splash` last (the index and the router).
macro_rules! kit {
    ($($name:literal),* $(,)?) => {
        concat!($(include_str!(concat!("../../../components/flutter/", $name, ".splash")), "\n"),*)
    };
}

const BAKED: &str = kit![
    "_kit",
    "add_to_app",
    "analysis_defaults",
    "android_splash_screen",
    "animations",
    "asset_transformation",
    "background_isolate_channels",
    "compass_app",
    "cupertino_gallery",
    "date_planner",
    "desktop_photo_search",
    "docs",
    "dynamic_theme",
    "form_app",
    "google_maps",
    "ios_app_clip",
    "material_3_demo",
    "navigation_and_routing",
    "pedometer",
    "platform_channels",
    "platform_design",
    "platform_view_swift",
    "simple_sdf",
    "simple_shader",
    "testing_app",
    "tool",
    "veggieseasons",
    "web_embedding",
    "_index",
];

const DEVICE_PATH: &str = "/data/local/tmp/flutter_samples.splash";

/// A route written here is picked up within a frame or two and mounted. Exists
/// so the visual-QA sweep can drive all 108 screens on a real device without
/// tapping through them:
///
/// ```text
/// adb shell "echo compass_app/booking > /data/local/tmp/flutter_samples.route"
/// adb exec-out screencap -p > booking.png
/// ```
///
/// Appending ` dark` to the line switches the palette for that shot.
const ROUTE_PATH: &str = "/data/local/tmp/flutter_samples.route";

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
                        // Upstream `Splash` always allocates an isolate VM; the
                        // light theme and the shared heap live on the app's main
                        // VM. See the repo README on the one upstream PR.
                        // Fit, not Fill — and this is load-bearing.
                        //
                        // The kit's `{t: "scroll"}` emits a plain View on this
                        // backend, so this ScrollYView is the only scrolling in
                        // the app. A Fill child would exactly match it and
                        // never scroll. See `page()` in `_kit.splash` for the
                        // measurement, and why mapping Scroll to ScrollYView
                        // makes it worse rather than better.
                        host := Splash{ width: Fill, height: Fit }
                    }
                    // The routing signal the mounted kit writes.
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
    route: String,
    #[rust]
    dark: bool,
    /// Viewport in vp, from WindowGeomChange. 0 until the first one arrives.
    #[rust]
    vw: f64,
    #[rust]
    vh: f64,
    #[rust]
    tick: u32,
    #[rust]
    started: bool,
    #[rust]
    last_route_file: String,
    /// Seconds since startup, fed to the kit as `st.t`.
    #[rust]
    clock: f64,
}

impl App {
    fn current_source() -> String {
        std::fs::read_to_string(DEVICE_PATH).unwrap_or_else(|_| BAKED.to_string())
    }

    /// The QA route override, if one has been written. `<route>[ dark]`.
    fn route_override() -> Option<String> {
        let raw = std::fs::read_to_string(ROUTE_PATH).ok()?;
        let line = raw.trim();
        if line.is_empty() {
            return None;
        }
        Some(line.to_string())
    }

    /// Translate + mount the active route.
    fn mount(&mut self, cx: &mut Cx) {
        let src = Self::current_source();
        self.last_src = src.clone();
        let route = if self.route.is_empty() {
            "index"
        } else {
            &self.route
        };
        let full =
            splash_makepad::kit::with_state_sized(
                route, self.dark, self.clock, self.vw, self.vh, &src,
            );
        if let Some(node) = splash_render::build(&full, splash_makepad::kit::register_stub_capabilities) {
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
        // Fork-free themed widgets, against upstream makepad.
        splash_widgets::widgets_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        // The viewport, so a page can name its own height. `Splash` wraps its
        // mount in `View{height:Fit}`, so filling is not available — see
        // `with_state_sized`. inner_size is physical; vp is what the kit's
        // lengths are in.
        if let Event::WindowGeomChange(e) = event {
            let g = &e.new_geom;
            let (vw, vh) = (
                g.inner_size.x / g.dpi_factor,
                g.inner_size.y / g.dpi_factor,
            );
            if (vw - self.vw).abs() > 0.5 || (vh - self.vh).abs() > 0.5 {
                self.vw = vw;
                self.vh = vh;
                self.last_src.clear();
                self.mount(cx);
            }
        }
        if matches!(event, Event::Startup) && !self.started {
            self.started = true;
            // Start on a named screen instead of the index, so one can be opened
            // directly for a look (desktop only — on device, tap through):
            //   SPLASH_ROUTE=date_planner/maya cargo run -p flutter-samples
            self.route = std::env::var("SPLASH_ROUTE").unwrap_or_else(|_| "index".to_string());
            self.next_frame = cx.new_next_frame();
            self.mount(cx);
        }
        if self.next_frame.is_event(event).is_some() {
            self.tick = self.tick.wrapping_add(1);
            let nav_raw = self.ui.widget(cx, ids!(nav_signal)).text();
            let nav = nav_raw.trim();
            if !nav.is_empty() {
                self.ui.widget(cx, ids!(nav_signal)).set_text(cx, "");
                if nav == "theme:toggle" {
                    self.dark = !self.dark;
                } else {
                    self.route = nav.to_string();
                }
                self.mount(cx);
            } else if self.tick % 10 == 0 {
                // A route written to ROUTE_PATH wins over whatever was tapped,
                // so the QA sweep can drive every screen from adb. Checked
                // independently of the source, so pushing an edited kit still
                // hot-reloads while a route override is in place — otherwise the
                // QA loop would need a reinstall for every screen tweak.
                let mut remount = false;
                match Self::route_override() {
                    Some(line) => {
                        if line != self.last_route_file {
                            self.last_route_file = line.clone();
                            let (route, dark) = match line.strip_suffix(" dark") {
                                Some(r) => (r.trim().to_string(), true),
                                None => (line, false),
                            };
                            self.route = route;
                            self.dark = dark;
                            remount = true;
                        }
                    }
                    // Removing the file forgets the last route, so writing the
                    // same route again re-mounts. Without this the QA sweep
                    // silently skipped any screen that matched the previous
                    // run's last route — it photographed whatever was on screen.
                    None => self.last_route_file.clear(),
                }
                if !remount && Self::current_source() != self.last_src {
                    remount = true;
                }
                if remount {
                    self.mount(cx);
                }
            }
            // Animation: a screen can only move if the tree is recomputed
            // against a changing value, so advance the clock and re-mount while
            // an animated route is on screen. Everything else stays static and
            // costs nothing.
            if self.route.starts_with("animations/") {
                self.clock += 1.0 / 60.0;
                self.mount(cx);
            }
            cx.redraw_all();
            self.next_frame = cx.new_next_frame();
        }
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[cfg(test)]
mod tests {
    use super::BAKED;
    use std::path::PathBuf;

    /// The baked list is spelled out above because the Android wrapper crate
    /// cannot run a build script. That means it can drift: add a `.splash` to
    /// `components/flutter/` and forget this list, and the screen is missing
    /// from the app while every test in `splash-makepad` still passes, because
    /// those assemble from the directory. This pins the two together.
    #[test]
    fn baked_kit_matches_the_directory() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../components/flutter");
        let assembled = splash_makepad::kit::concat_kit(&dir).expect("kit assembles");
        // `concat_kit` interleaves `// ---- <file>` markers; the baked const has
        // none. Strip them and the two must be byte-identical.
        let stripped: String = assembled
            .lines()
            .filter(|l| !(l.starts_with("// ---- ") && l.ends_with(".splash")))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_eq!(
            stripped, BAKED,
            "the baked kit in main.rs has drifted from components/flutter/"
        );
    }
}
