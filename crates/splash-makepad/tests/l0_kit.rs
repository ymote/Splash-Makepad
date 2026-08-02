//! The L0 theme kit is a contract with `splash-render`, and this is where that
//! consumer lives — so the contract is CHECKED here rather than asserted
//! elsewhere.
//!
//! Two earlier attempts to lower L0 at this renderer were reverted for exactly
//! that: they mapped roles to tags nobody had run, and five of twenty-three did
//! not exist. The repository that owns the profile has no dependency on this
//! one, so nothing there can catch a drift. These tests can.

const KIT: &str = include_str!("../../../components/l0/_kit.splash");

/// Every role, and the call that exercises it.
///
/// Kept as data rather than as one big card so a role that evaluates to nil is
/// NAMED. Inside a larger tree an empty node is just an absence, which is the
/// failure mode `ui-profile-l0.md` §1.1 exists to prevent.
const ROLES: &[(&str, &str)] = &[
    ("l0_hero", r#"l0_hero("$184.20", 40)"#),
    ("l0_title", r#"l0_title("Top Stories")"#),
    ("l0_body", r#"l0_body("Rust 1.95 lands")"#),
    ("l0_row_text", r#"l0_row_text("NVDA")"#),
    ("l0_caption", r#"l0_caption("HACKER NEWS")"#),
    ("l0_value", r#"l0_value("41.2M")"#),
    ("l0_stat", r#"l0_stat("+1.7%", 1)"#),
    ("l0_surface", r#"l0_surface([l0_title("t")])"#),
    ("l0_surface_photo", r#"l0_surface_photo("https://x/y.jpg", [l0_title("t")])"#),
    ("l0_col", r#"l0_col([l0_title("t")])"#),
    ("l0_col_gap", r#"l0_col_gap(8, [l0_title("t")])"#),
    ("l0_row", r#"l0_row([l0_title("t")])"#),
    ("l0_row_gap", r#"l0_row_gap(6, [l0_title("t")])"#),
    ("l0_grid", r#"l0_grid([l0_title("t")])"#),
    ("l0_panel", r#"l0_panel([l0_title("t")])"#),
    ("l0_rule", r#"l0_rule()"#),
    ("l0_tile", r#"l0_tile("Open", "$181.00")"#),
    ("l0_chip", r#"l0_chip("1M", 1)"#),
    ("l0_photo", r#"l0_photo("https://x/y.jpg")"#),
    ("l0_weathericon", r#"l0_weathericon("clear")"#),
    ("l0_unsupported", r#"l0_unsupported("TempBar")"#),
];

/// A script's value is a bare VARIABLE, never a call.
///
/// `fn f() { … }` followed by `f()` evaluates to nil — which looks exactly like
/// a broken kit and is not. Every kit in this repository ends `let node = …`
/// then `node`, and getting this wrong made all 21 roles report as dead on the
/// first run.
fn eval(call: &str) -> Option<splash_render::UiNode> {
    let src = format!("{KIT}\nlet node = {call}\nnode\n");
    splash_render::build(&src, |_vm| {})
}

#[test]
fn every_role_produces_a_node() {
    let dead: Vec<&str> = ROLES
        .iter()
        .filter(|(_, call)| eval(call).is_none())
        .map(|(name, _)| *name)
        .collect();
    assert!(
        dead.is_empty(),
        "these roles evaluated to nothing: {dead:?}\n\
         A role that produces nil renders as an absence — the card looks complete \
         and is not (profile §1.1)."
    );
}

/// A role must land on the kind its meaning implies, not merely on *a* kind.
///
/// `Panel` becoming a `Text` would still produce a tree and still pass the test
/// above, and the card would be unreadable.
#[test]
fn each_role_lands_on_the_kind_its_meaning_implies() {
    for (call, expected) in [
        (r#"l0_title("t")"#, "Text"),
        (r#"l0_panel([l0_title("t")])"#, "Card"),
        (r#"l0_row([l0_title("t")])"#, "Row"),
        (r#"l0_col([l0_title("t")])"#, "Column"),
        (r#"l0_grid([l0_title("t")])"#, "Grid"),
        (r#"l0_rule()"#, "Divider"),
        (r#"l0_chip("1M", 1)"#, "Chip"),
        (r#"l0_photo("https://x/y.jpg")"#, "Image"),
        (r#"l0_weathericon("clear")"#, "WeatherIcon"),
        (r#"l0_tile("Open", "$1")"#, "Card"),
        (r#"l0_surface([l0_title("t")])"#, "Column"),
        (r#"l0_surface_photo("u", [l0_title("t")])"#, "Stack"),
    ] {
        let tree = eval(call).unwrap_or_else(|| panic!("{call} evaluated to nil"));
        assert_eq!(
            format!("{:?}", tree.kind),
            expected,
            "{call} landed on the wrong kind"
        );
    }
}

/// A chip must LOOK different when selected.
///
/// Every chip in the stock card's range picker rendered identically once, so the
/// card could not show which range was chosen — and the golden recorded that as
/// correct. Presentation is the theme's business, but "this one is selected" is
/// meaning, and it has to survive.
#[test]
fn a_selected_chip_differs_from_an_unselected_one() {
    let on = eval(r#"l0_chip("1M", 1)"#).expect("on");
    let off = eval(r#"l0_chip("1M", 0)"#).expect("off");

    // `bg`, specifically — NOT `selected`.
    //
    // The first version of this test compared `(bg, selected)`, and a mutation
    // that made the fill constant still passed: `selected` alone differed. But
    // an attribute existing in `Attrs` is not evidence that a backend draws it,
    // and this one could not be shown to reach the chip's rendering. Asserting
    // it would have repeated the original defect exactly — every chip drawn
    // identically, with a green test.
    assert_ne!(
        on.attrs.bg, off.attrs.bg,
        "a selected chip must differ in its FILL, which is drawn"
    );
}

/// A statistic's tint carries direction, not decoration.
///
/// Red-versus-green is presentation; "this value fell" is meaning. A lowering
/// that dropped the attribute lost both, which is why the profile calls `tint`
/// the instructive case.
#[test]
fn a_stat_is_tinted_by_direction() {
    let up = eval(r#"l0_stat("+1.7%", 1)"#).expect("up");
    let flat = eval(r#"l0_stat("0.0%", 0)"#).expect("flat");
    let down = eval(r#"l0_stat("-1.7%", -1)"#).expect("down");
    assert_ne!(up.attrs.color, down.attrs.color, "a rise and a fall must differ");
    assert_ne!(up.attrs.color, flat.attrs.color, "a rise and no change must differ");
    assert_ne!(down.attrs.color, flat.attrs.color, "a fall and no change must differ");
}

/// A role this renderer cannot draw renders a VISIBLE marker.
///
/// Five of the six data visualisations have no kind here. Returning nothing
/// would let a card lose its temperature bars and still look complete.
#[test]
fn an_unsupported_role_is_visible_rather_than_absent() {
    let tree = eval(r#"l0_unsupported("TempBar")"#).expect("nil");
    let mut text = String::new();
    fn walk(n: &splash_render::UiNode, out: &mut String) {
        if let Some(t) = n.attrs.text.as_deref() {
            out.push_str(t);
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(&tree, &mut text);
    assert!(
        text.contains("TempBar"),
        "the marker must NAME the role it stands in for, got {text:?}"
    );
}
