//! The whole L0 path, end to end, in the repository where its consumer lives.
//!
//! ```text
//!   L0 ledger  →  realize  →  kit::lower  →  _kit.splash  →  splash_render  →  UiNode
//! ```
//!
//! `splash-ui-l0` cannot check this itself: it has no dependency on
//! `splash-render` and must not acquire one, since that would pull a second
//! makepad lineage into its lockfile. So the profile's repository can only
//! assert the contract, and twice it asserted one that was wrong — five of
//! twenty-three roles mapped to tags that did not exist, and the attempt was
//! reverted.
//!
//! Here the contract is executed. `splash-ui-l0` is a dev-dependency, which is
//! possible only because it was extracted from `splash-core` and depends on
//! `serde_json` alone.

use splash_ui_l0::{kit, realize, RealizeLimits};

const KIT: &str = include_str!("../../../components/l0/_kit.splash");

const WEATHER: &str = include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/weather.card");
const NEWS: &str = include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/news.card");
const STOCK: &str = include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/stock.card");

fn build(card: &str, data: serde_json::Value) -> splash_render::UiNode {
    let report = realize(card, &data, RealizeLimits::default());
    assert!(
        report.diagnostics.is_empty(),
        "card did not realize cleanly: {:#?}",
        report.diagnostics
    );
    let root = report.root.expect("a realized tree");
    // The tail is a bare VARIABLE, not a call — `fn f() {…}` then `f()` is nil.
    let src = format!("{KIT}\n{}", kit::lower(&root));
    splash_render::build(&src, |_vm| {})
        .unwrap_or_else(|| panic!("the lowered card evaluated to nil:\n{}", kit::lower(&root)))
}

fn news_data() -> serde_json::Value {
    serde_json::json!({
        "lead": [{"id":"1","title":"Rust 1.95","author":"a","points":412.0,"comments":137.0,"url":"u"}],
        "feed": [{"id":"2","title":"Another","author":"b","points":90.0,"comments":12.0,"url":"u"}],
        "article": {}, "selected": "", "env": {"locale": {"lang":"en"}}
    })
}

fn stock_data() -> serde_json::Value {
    serde_json::json!({
        "movers": [{"ticker":"NVDA","name":"Nvidia","last":184.2,"change":3.1,"pct":1.7}],
        "quote": {"name":"Nvidia","last":184.2,"change":3.1,"pct":1.7,"open":181.0,
                  "high":185.6,"low":180.2,"volume":41200000.0,"mktcap":4.52e12,"pe":58.3},
        "selected": "", "range": "m1", "env": {"locale":{}},
        "copy": {"movers":"Top Movers","open":"Open","high":"High","low":"Low",
                 "volume":"Volume","mktcap":"Mkt Cap","pe":"P/E"}
    })
}

fn weather_data() -> serde_json::Value {
    serde_json::json!({
        "place": {"name":"Kyoto","lat":35.0,"lon":135.8},
        "now": {"temp":21.0,"cond":"clear","feels":20.0,"humidity":54.0,"wind":3.2,
                "pressure":1013.0,"uv":4.0,"visibility":10.0},
        "week": {"days":[{"dayname":"Mon","hi":24.0,"lo":15.0,"cond":"clear"}],
                 "min_lo":15.0,"max_hi":24.0},
        "sun": {"rise":5.1,"set":18.9}, "moon": {"phase":0.5,"illum":50.0},
        "scene": "https://x/y.jpg", "city":"", "units":"c", "days":7.0,
        "env": {"locale":{"lang":"en","temp_unit":"c"}}
    })
}

/// Every reference card builds through the kit into a real tree.
///
/// The node count is the point: a card that evaluated to a single empty node
/// would still be `Some` and would still render as a blank screen.
#[test]
fn every_reference_card_builds_through_the_kit() {
    for (name, tree) in [
        ("news", build(NEWS, news_data())),
        ("stock", build(STOCK, stock_data())),
        ("weather", build(WEATHER, weather_data())),
    ] {
        assert!(
            tree.count() > 10,
            "{name} produced a {}-node tree — too small to be a card",
            tree.count()
        );
    }
}

/// The card's TEXT survives the trip.
///
/// A tree of the right shape carrying no words is the failure this catches: the
/// structure is what a node count checks, and the content is what a reader sees.
#[test]
fn a_card_carries_its_text_through_the_kit() {
    fn words(n: &splash_render::UiNode, out: &mut Vec<String>) {
        if let Some(t) = n.attrs.text.as_deref() {
            out.push(t.to_owned());
        }
        for c in &n.children {
            words(c, out);
        }
    }
    let mut out = Vec::new();
    words(&build(NEWS, news_data()), &mut out);
    let text = out.join(" | ");
    for expected in ["HACKER NEWS", "Top Stories", "Rust 1.95"] {
        assert!(text.contains(expected), "{expected:?} missing from: {text}");
    }
}

/// **The reason this lowering exists.** A card must carry no presentation.
///
/// `makepad::lower` emits ten hardcoded colours and a font-size ramp, so it
/// decides what a theme decides and reaches one backend of three. The kit
/// lowering must name roles only — and this asserts it of the emitted SOURCE,
/// because the built tree is full of colours by design: the kit puts them there.
#[test]
fn the_lowered_card_names_roles_and_no_presentation() {
    let report = realize(STOCK, &stock_data(), RealizeLimits::default());
    let src = kit::lower(&report.root.expect("root"));

    for forbidden in ["#", "draw_bg", "draw_text", "SolidView", "RoundedView", "Inset{"] {
        assert!(
            !src.contains(forbidden),
            "{forbidden:?} is presentation and belongs to the theme:\n{src}"
        );
    }
    assert!(src.contains("l0_panel("), "roles must be named:\n{src}");
}

/// A role the kit cannot draw becomes a VISIBLE marker, never nothing.
///
/// Five of the six data visualisations have no kind in this renderer. A card
/// that silently loses its temperature bars still looks complete, which is the
/// failure `ui-profile-l0.md` §1.1 exists to prevent.
#[test]
fn an_unanswerable_role_survives_as_a_marker() {
    let report = realize(WEATHER, &weather_data(), RealizeLimits::default());
    let src = kit::lower(&report.root.expect("root"));
    assert!(
        src.contains("l0_unsupported(\"TempBar\")"),
        "TempBar has no kit answer and must lower to a named marker:\n{src}"
    );

    fn words(n: &splash_render::UiNode, out: &mut Vec<String>) {
        if let Some(t) = n.attrs.text.as_deref() {
            out.push(t.to_owned());
        }
        for c in &n.children {
            words(c, out);
        }
    }
    let mut out = Vec::new();
    words(&build(WEATHER, weather_data()), &mut out);
    assert!(
        out.iter().any(|w| w.contains("TempBar")),
        "and the marker must reach the tree, naming the role: {out:?}"
    );
}
