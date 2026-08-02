//! L0 cards, as routes this host can render.
//!
//! This is what "the kit is wired on this backend" means, and it was the missing
//! piece: `components/l0/_kit.splash` existed, `kit::lower` emitted calls to it,
//! `splash_render` evaluated them and `to_makepad_ui` mapped them — and nothing
//! anywhere mounted the result. The whole path was tested and unrenderable.
//!
//! Every reference card becomes a route:
//!
//! ```text
//!   l0/news  ->  realize  ->  kit::lower  ->  _kit.splash  ->  splash_render
//! ```
//!
//! **The data is baked and static.** These are the same blobs the profile's own
//! tests use, so what appears here is what those tests assert — and where a
//! source would be answered live on a backend that can fetch, this one shows the
//! seeded value. That is a real difference from octos-one and it is why the two
//! are not expected to look identical.

use splash_ui_l0::{kit, realize, RealizeLimits};

/// The theme. Every L0 route is this plus one lowered card.
const KIT: &str = include_str!("../../../components/l0/_kit.splash");

const NEWS: &str = include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/news.card");
const STOCK: &str = include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/stock.card");
const WEATHER: &str =
    include_str!("../../../../Splash/crates/splash-ui-l0/tests/fixtures/weather.card");

const NEWS_DATA: &str = include_str!("data/news.json");
const STOCK_DATA: &str = include_str!("data/stock.json");
const WEATHER_DATA: &str = include_str!("data/weather.json");

/// The routes this module answers, and what each renders.
pub const ROUTES: &[(&str, &str)] = &[
    ("l0/news", "L0 — news"),
    ("l0/stock", "L0 — stock"),
    ("l0/weather", "L0 — weather"),
];

pub fn has(route: &str) -> bool {
    ROUTES.iter().any(|(r, _)| *r == route)
}

pub fn title_of(route: &str) -> Option<&'static str> {
    ROUTES.iter().find(|(r, _)| *r == route).map(|(_, t)| *t)
}

/// A card's source, realized and lowered against the theme.
///
/// A card that fails to realize returns its diagnostics AS A CARD rather than an
/// empty string: an unrenderable route is otherwise a blank screen, which reads
/// as a layout bug rather than as a rejected card.
pub fn source_for(route: &str) -> String {
    let (card, data) = match route {
        "l0/news" => (NEWS, NEWS_DATA),
        "l0/stock" => (STOCK, STOCK_DATA),
        _ => (WEATHER, WEATHER_DATA),
    };
    let data: serde_json::Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(e) => return failed(&format!("data did not parse: {e}")),
    };
    let report = realize(card, &data, RealizeLimits::default());
    let Some(root) = report.root else {
        let why: Vec<String> = report.diagnostics.iter().map(|d| d.message.clone()).collect();
        return failed(&why.join("; "));
    };
    format!("{KIT}\n{}", kit::lower(&root))
}

/// A visible failure, in the kit's own vocabulary.
fn failed(why: &str) -> String {
    let escaped = why.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{KIT}\nlet node = l0_surface([l0_title(\"card did not realize\"), l0_body(\"{escaped}\")])\nnode\n"
    )
}

#[cfg(test)]
mod tests {
    /// Every L0 route evaluates to a real tree through the whole path.
    ///
    /// This host is the only place in this repository that MOUNTS an L0 card.
    /// Before it, the kit, the lowering, the evaluator and the widget mapping
    /// were each tested and the composition of them was unrenderable — there
    /// was nowhere to put one on screen.
    #[test]
    fn every_l0_route_builds_a_tree() {
        for (route, title) in super::ROUTES {
            let src = super::source_for(route);
            let tree = splash_render::build(&src, |_vm| {})
                .unwrap_or_else(|| panic!("{route} ({title}) evaluated to nil"));
            assert!(
                tree.count() > 10,
                "{route} produced a {}-node tree — too small to be a card",
                tree.count()
            );
        }
    }

    /// A route that cannot realize renders its reason, not a blank screen.
    #[test]
    fn a_card_that_does_not_realize_says_why() {
        let src = super::failed("the reason");
        let tree = splash_render::build(&src, |_vm| {}).expect("the failure card evaluates");
        let mut text = String::new();
        fn words(n: &splash_render::UiNode, out: &mut String) {
            if let Some(t) = n.attrs.text.as_deref() {
                out.push_str(t);
            }
            for c in &n.children {
                words(c, out);
            }
        }
        words(&tree, &mut text);
        assert!(text.contains("the reason"), "got {text:?}");
    }

    /// The five data visualisations reach the tree on THIS backend too.
    ///
    /// octos-one proves them against its own native widgets; this proves the
    /// same card reaches the same kinds here, which is the claim §1.1 makes
    /// about `UiNode` being the branch point.
    #[test]
    fn the_visualisations_reach_this_backends_tree() {
        let tree = splash_render::build(&super::source_for("l0/weather"), |_vm| {})
            .expect("weather evaluates");
        let mut kinds = Vec::new();
        fn walk(n: &splash_render::UiNode, out: &mut Vec<String>) {
            out.push(format!("{:?}", n.kind));
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&tree, &mut kinds);
        for expected in ["TempBar", "SunArc", "MoonPhase", "AqiContour"] {
            assert!(kinds.iter().any(|k| k == expected), "{expected} missing");
        }
    }
}
