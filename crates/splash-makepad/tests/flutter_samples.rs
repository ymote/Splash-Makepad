//! Sweep every route in the flutter/samples kit.
//!
//! This is the repo's answer to `flutter/samples/tool`, which walks every sample
//! and runs `flutter analyze` over it. Here the walk is over routes, and the
//! check is that each one evaluates and translates to the screen it claims to
//! be.
//!
//! The second half matters more than the first. The router falls through to the
//! index for any route it does not recognise, so "it rendered" proves nothing —
//! a typo'd route renders the index and looks perfectly healthy. Every case
//! below therefore asserts on a string only that screen emits. Splash-OH learned
//! this the hard way: two of its 28 catalog screens were reachable and wrong.

use splash_makepad::{kit, to_makepad_ui};
use std::path::PathBuf;

fn kit_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../components/flutter")
}

/// Assemble once; every case re-uses the same script with a different `st`.
fn assembled() -> String {
    kit::concat_kit(&kit_dir()).expect("components/flutter assembles")
}

/// Translate one route, or panic with the route name attached.
fn render(kit: &str, route: &str) -> String {
    let src = kit::with_state(route, false, kit);
    let tree = splash_render::build(&src, kit::register_stub_capabilities)
        .unwrap_or_else(|| panic!("route {route:?} evaluated to nil"));
    assert!(
        tree.count() > 3,
        "route {route:?} produced a {}-node tree — too small to be a screen",
        tree.count()
    );
    to_makepad_ui(&tree)
}

/// (route, a string only this screen emits)
fn cases() -> Vec<(String, &'static str)> {
    let mut v: Vec<(String, &'static str)> = [
        // The index itself.
        ("index", "flutter/samples"),
        // material_3_demo — four screens behind a NavigationBar.
        ("material_3_demo", "Common buttons"),
        ("material_3_demo/color", "Key colors"),
        ("material_3_demo/typography", "Display Large"),
        ("material_3_demo/elevation", "Level 5"),
        // cupertino_gallery — tabs plus 21 widget pages.
        ("cupertino_gallery", "Sliding Segmented Control"),
        ("cupertino_gallery/settings", "Dark Mode"),
        ("cupertino_gallery/action_sheet", "Action One"),
        ("cupertino_gallery/activity", "Activity Indicator"),
        ("cupertino_gallery/alert", "This is a sample alert dialog."),
        ("cupertino_gallery/button", "CupertinoButton.filled widget"),
        ("cupertino_gallery/checkbox", "Checkbox"),
        ("cupertino_gallery/context_menu", "Long press to activate context menu:"),
        ("cupertino_gallery/date_picker", "Date Picker"),
        ("cupertino_gallery/list_tile", "Subtitle"),
        ("cupertino_gallery/picker", "Picker"),
        ("cupertino_gallery/popup", "This is a popup surface."),
        ("cupertino_gallery/radio", "Radio"),
        ("cupertino_gallery/scrollbar", "Item 23"),
        ("cupertino_gallery/search", "Search"),
        ("cupertino_gallery/segmented", "Three"),
        ("cupertino_gallery/sheet", "This is a sheet"),
        ("cupertino_gallery/slider", "Slider"),
        ("cupertino_gallery/sliding", "Sliding Segmented Control"),
        ("cupertino_gallery/switch", "Switch"),
        ("cupertino_gallery/text_field", "Enter text"),
        ("cupertino_gallery/text_theme", "This is the picker text style"),
        ("cupertino_gallery/time_picker", "Time Picker"),
        // date_planner — the list plus all nine events.
        ("date_planner", "NEXT 7 DAYS"),
        ("date_planner/pagliacci", "Pick up Carmen at the airport and bring her to the show"),
        ("date_planner/camping", "Find a sleeping bag"),
        ("date_planner/game_night", "Bring a desert to share"),
        ("date_planner/doctor", "Record heart rate data"),
        ("date_planner/sayulita", "Get a new bathing suit"),
        ("date_planner/maya", "Guava kombucha"),
        ("date_planner/school", "First day of school outfit"),
        ("date_planner/book_launch", "Send draft to editor"),
        ("date_planner/wwdc", "Learn about Create ML"),
        // platform_design — four Material tabs plus the iOS chrome.
        ("platform_design", "Silent Harbor"),
        ("platform_design/news", "Golden Thread releases Velvet Signal"),
        ("platform_design/profile", "My neighbor hates me"),
        ("platform_design/settings", "Auto-transition playback to cast devices"),
        ("platform_design/ios", "iOS chrome"),
        // animations — the index and all twenty demos.
        ("animations", "AnimatedSwitcher"),
        // form_app — index plus four demos.
        ("form_app", "Form Samples"),
        ("form_app/signin_http", "mock http.Client"),
        ("form_app/autofill", "Enter your street address"),
        ("form_app/form_widgets", "Brushed teeth"),
        ("form_app/validation", "Not a valid adjective."),
        // navigation_and_routing — tabs, details, settings, sign-in.
        ("navigation_and_routing", "Kindred"),
        ("navigation_and_routing/new", "Too Like the Lightning"),
        ("navigation_and_routing/all", "The Lathe of Heaven"),
        ("navigation_and_routing/authors", "Octavia E. Butler"),
        ("navigation_and_routing/settings", "Show Dialog"),
        ("navigation_and_routing/signin", "no route table"),
        ("navigation_and_routing/book/0", "Left Hand of Darkness"),
        ("navigation_and_routing/book/1", "Ada Palmer"),
        ("navigation_and_routing/book/2", "Kindred"),
        ("navigation_and_routing/book/3", "The Lathe of Heaven"),
        ("navigation_and_routing/author/0", "Ursula K. Le Guin"),
        ("navigation_and_routing/author/1", "Too Like the Lightning"),
        ("navigation_and_routing/author/2", "Kindred"),
        // compass_app — the five screens.
        ("compass_app", "Let's explore"),
        ("compass_app/search", "Where to?"),
        ("compass_app/results", "Amazon Rainforest"),
        ("compass_app/activities", "Dog Sledding Experience"),
        ("compass_app/booking", "Alaska, United States"),
        // desktop_photo_search — both widget-set variants.
        ("desktop_photo_search", "Photo Search — Material"),
        ("desktop_photo_search/fluent", "Photo Search — fluent_ui"),
        // dynamic_theme.
        ("dynamic_theme", "change_text_scale_factor"),
        // testing_app.
        ("testing_app", "Added to favorites."),
        ("testing_app/favorites", "Favorites"),
    ]
    .into_iter()
    .map(|(r, m)| (r.to_string(), m))
    .collect();

    // Every animations demo, by the key its index row links to.
    for key in [
        "animated_container", "page_route", "controller", "tweens",
        "animated_builder", "custom_tween", "tween_sequence", "fade_transition",
        "expand_card", "carousel", "focus_image", "card_swipe", "flutter_animate",
        "repeating", "spring", "animated_list", "animated_positioned",
        "animated_switcher", "hero", "curved",
    ] {
        v.push((format!("animations/{key}"), "What it demonstrates"));
    }

    // The sixteen directories with no analogue. Each must reach its own note,
    // not the index — so assert on its verdict, not the shared banner.
    for (route, marker) in [
        ("add_to_app", "This screen is the embed"),
        ("analysis_defaults", "analysis_options.yaml"),
        ("android_splash_screen", "windowSplashScreenAnimatedIcon"),
        ("asset_transformation", "splash:// resolves a request to bytes"),
        ("background_isolate_channels", "without blocking"),
        ("docs", "compass_app"),
        ("google_maps", "OpenStreetMap vector tiles"),
        ("ios_app_clip", "entitlements"),
        ("pedometer", "Health Connect"),
        ("platform_channels", "MethodChannel"),
        ("platform_view_swift", "UIViewController"),
        ("simple_sdf", "does not render"),
        ("simple_shader", "does not render"),
        ("tool", "flutter analyze"),
        ("veggieseasons", "flutter/demos"),
        ("web_embedding", "hostElement"),
    ] {
        v.push((route.to_string(), marker));
    }
    v
}

#[test]
fn every_route_renders_its_own_screen() {
    let kit = assembled();
    let mut failures = Vec::new();
    for (route, marker) in cases() {
        let ui = render(&kit, &route);
        if !ui.contains(marker) {
            failures.push(format!(
                "route {route:?} rendered, but does not contain {marker:?} \
                 — it probably fell through to the index"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every directory listed on the index must have a route that resolves to
/// something other than the index. Catches an entry added to the list but never
/// wired into the router.
#[test]
fn every_indexed_sample_is_reachable() {
    let kit = assembled();
    let index_marker = "27 directories";
    let dirs = std::fs::read_dir(kit_dir())
        .expect("kit dir")
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".splash") && !n.starts_with('_'))
        .map(|n| n.trim_end_matches(".splash").to_string())
        .collect::<Vec<_>>();

    assert_eq!(dirs.len(), 27, "one .splash per flutter/samples directory");

    let mut unreachable = Vec::new();
    for dir in dirs {
        let ui = render(&kit, &dir);
        if ui.contains(index_marker) {
            unreachable.push(dir);
        }
    }
    assert!(
        unreachable.is_empty(),
        "these routes fell through to the index: {unreachable:?}"
    );
}

/// Every call to a shared helper must pass the arity that helper declares.
///
/// The VM does not check this: a call with too few arguments silently binds the
/// missing parameters to nil, and extra ones are dropped. `para(s, 14, 400,
/// colour, 40)` against a four-parameter `para` bound `colour` to `400` and the
/// height to a colour word — the screen still rendered, still contained its
/// marker string, and still passed every other test in this file, while looking
/// completely wrong. Three real bugs of this shape survived the render sweep.
#[test]
fn helper_calls_have_the_declared_arity() {
    // Comments are not code. A prose mention like "argb() then builds ..." was
    // being read as a zero-argument call to argb, which is a checker bug rather
    // than a kit one — strip line comments before scanning.
    // `//` only starts a comment outside a string literal. Cutting on the first
    // occurrence truncated `m_section("splash:// resolves ...")` mid-call and
    // reported it as taking fifteen arguments.
    fn strip_comment(line: &str) -> String {
        let (mut out, mut in_str, mut esc) = (String::new(), false, false);
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_str {
                out.push(c);
                match c {
                    _ if esc => esc = false,
                    '\\' => esc = true,
                    '"' => in_str = false,
                    _ => {}
                }
                continue;
            }
            if c == '"' {
                in_str = true;
                out.push(c);
                continue;
            }
            if c == '/' && chars.peek() == Some(&'/') {
                break;
            }
            out.push(c);
        }
        out.push('\n');
        out
    }
    let kit: String = assembled().lines().map(strip_comment).collect();

    // Number of top-level arguments between the parens that follow `name(`.
    // An empty list is 0, and a trailing comma (legal in this DSL) is not an
    // extra argument.
    fn arity_at(src: &str, open: usize) -> usize {
        let (mut depth, mut in_str, mut esc) = (1usize, false, false);
        let mut args: Vec<String> = vec![String::new()];
        for ch in src[open..].chars() {
            if in_str {
                args.last_mut().unwrap().push(ch);
                match ch {
                    _ if esc => esc = false,
                    '\\' => esc = true,
                    '"' => in_str = false,
                    _ => {}
                }
                continue;
            }
            match ch {
                '"' => in_str = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                ',' if depth == 1 => {
                    args.push(String::new());
                    continue;
                }
                _ => {}
            }
            args.last_mut().unwrap().push(ch);
        }
        if args.last().is_some_and(|a| a.trim().is_empty()) {
            args.pop();
        }
        args.len()
    }

    // Read each `fn name(a, b, c)` declaration out of the kit itself, so the
    // expected arity cannot drift from the definition.
    let mut declared: Vec<(String, usize)> = Vec::new();
    for line in kit.lines() {
        let Some(rest) = line.trim_start().strip_prefix("fn ") else {
            continue;
        };
        let Some(open) = rest.find('(') else { continue };
        let name = rest[..open].trim().to_string();
        let params = &rest[open + 1..];
        let Some(close) = params.find(')') else { continue };
        let inner = params[..close].trim();
        let n = if inner.is_empty() {
            0
        } else {
            inner.split(',').count()
        };
        declared.push((name, n));
    }
    assert!(
        declared.len() > 20,
        "expected to find the kit's helpers, found {}",
        declared.len()
    );

    let mut wrong = Vec::new();
    for (name, want) in &declared {
        let needle = format!("{name}(");
        let mut from = 0;
        while let Some(rel) = kit[from..].find(&needle) {
            let at = from + rel;
            from = at + needle.len();
            // Skip the declaration itself, and any longer identifier ending in
            // this name (`m_btn` inside `m_btn_text`).
            let before = kit[..at].trim_end();
            if before.ends_with("fn") {
                continue;
            }
            if kit[..at]
                .chars()
                .last()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
            {
                continue;
            }
            let got = arity_at(&kit, from);
            if got != *want {
                wrong.push(format!(
                    "{name}() called with {got} args, declared {want}: {:?}",
                    &kit[at..(at + 64).min(kit.len())]
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Every animation demo must actually animate, and no two may be the same.
///
/// The kit has no tween or controller; a screen moves only because the host
/// re-mounts it against a changing `st.t` and the tree is recomputed. That is
/// easy to get wrong in a way nothing else notices — a demo that ignores the
/// clock renders perfectly and simply sits there, which is what all twenty did
/// before. So: render each at several times and require the output to change,
/// and render them all at one time and require twenty different frames.
#[test]
fn every_animation_demo_moves_and_is_its_own() {
    const DEMOS: [&str; 20] = [
        "animated_container", "page_route", "controller", "tweens",
        "animated_builder", "custom_tween", "tween_sequence", "fade_transition",
        "expand_card", "carousel", "focus_image", "card_swipe",
        "flutter_animate", "repeating", "spring", "animated_list",
        "animated_positioned", "animated_switcher", "hero", "curved",
    ];
    let kit = assembled();
    let frame = |demo: &str, t: f64| {
        let src = kit::with_state_at(&format!("animations/{demo}"), false, t, &kit);
        let tree = splash_render::build(&src, kit::register_stub_capabilities)
            .unwrap_or_else(|| panic!("animations/{demo} at t={t} evaluated to nil"));
        to_makepad_ui(&tree)
    };

    // Several sample times, because any single pair can coincide: `pingpong`
    // over a 1.2s period is 0.5 at both t=0.3 and t=0.9.
    let mut still = Vec::new();
    for demo in DEMOS {
        let a = frame(demo, 0.15);
        let moved = [0.55, 1.05, 1.7, 2.3]
            .iter()
            .any(|t| frame(demo, *t) != a);
        if !moved {
            still.push(demo);
        }
    }
    assert!(still.is_empty(), "these demos do not animate: {still:?}");

    // And each must be its own effect, not a shared fallback.
    let mut frames: Vec<String> = DEMOS.iter().map(|d| frame(d, 0.7)).collect();
    frames.sort();
    let before = frames.len();
    frames.dedup();
    assert_eq!(
        frames.len(),
        before,
        "some demos render the identical frame — they are sharing an effect"
    );
}

/// The assembler's ordering contract, which the whole kit depends on.
#[test]
fn kit_assembles_head_first_and_index_last() {
    let kit = assembled();
    // The files' own bodies use `// ---- ` as a section rule, so only look at
    // markers that name a .splash file.
    let markers: Vec<&str> = kit
        .lines()
        .filter_map(|l| l.strip_prefix("// ---- "))
        .filter(|l| l.ends_with(".splash"))
        .collect();

    assert_eq!(markers.first(), Some(&"_kit.splash"), "_kit.splash is first");
    assert_eq!(markers.last(), Some(&"_index.splash"), "_index.splash is last");
    // 27 sample directories, plus the head and the tail.
    assert_eq!(markers.len(), 29, "one marker per .splash file");

    let mut middle = markers[1..markers.len() - 1].to_vec();
    let sorted = {
        let mut s = middle.clone();
        s.sort_unstable();
        s
    };
    middle.dedup();
    assert_eq!(middle.len(), 27, "no file appended twice");
    assert_eq!(
        markers[1..markers.len() - 1],
        sorted[..],
        "the middle must be sorted, so the assembled script is byte-stable"
    );
}
