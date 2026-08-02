//! The reference catalog's screens, baked in.
//!
//! These are the **same `.splash` files** the Splash-Android catalog renders
//! with real `com.google.android.material.*` views. Baking the identical source
//! here is what makes the two backends comparable: same input, so any difference
//! on screen is this renderer's.
//!
//! `include_str!` rather than reading the directory, because cargo-makepad
//! builds Android inside a generated wrapper crate that never runs a build
//! script (the same reason the flutter kit is baked).

/// Shared helpers every screen composes from (`section`, `caption`, `group`, …).
pub const KIT: &str = include_str!("../../../components/material/screens/kit.splash");

/// Every route, in the order the reference lists them.
pub const SCREENS: &[(&str, &str)] = &[
    ("adaptive", include_str!("../../../components/material/screens/adaptive.splash")),
    ("allcomponents", include_str!("../../../components/material/screens/allcomponents.splash")),
    ("badge", include_str!("../../../components/material/screens/badge.splash")),
    ("bottomappbar", include_str!("../../../components/material/screens/bottomappbar.splash")),
    ("bottomnav", include_str!("../../../components/material/screens/bottomnav.splash")),
    ("bottomsheet", include_str!("../../../components/material/screens/bottomsheet.splash")),
    ("button", include_str!("../../../components/material/screens/button.splash")),
    ("card", include_str!("../../../components/material/screens/card.splash")),
    ("carousel", include_str!("../../../components/material/screens/carousel.splash")),
    ("checkbox", include_str!("../../../components/material/screens/checkbox.splash")),
    ("chip", include_str!("../../../components/material/screens/chip.splash")),
    ("color", include_str!("../../../components/material/screens/color.splash")),
    ("datepicker", include_str!("../../../components/material/screens/datepicker.splash")),
    ("dialog", include_str!("../../../components/material/screens/dialog.splash")),
    ("divider", include_str!("../../../components/material/screens/divider.splash")),
    ("dockedtoolbar", include_str!("../../../components/material/screens/dockedtoolbar.splash")),
    ("elevation", include_str!("../../../components/material/screens/elevation.splash")),
    ("fab", include_str!("../../../components/material/screens/fab.splash")),
    ("floatingtoolbar", include_str!("../../../components/material/screens/floatingtoolbar.splash")),
    ("font", include_str!("../../../components/material/screens/font.splash")),
    ("imageview", include_str!("../../../components/material/screens/imageview.splash")),
    ("listitem", include_str!("../../../components/material/screens/listitem.splash")),
    ("loadingindicator", include_str!("../../../components/material/screens/loadingindicator.splash")),
    ("materialswitch", include_str!("../../../components/material/screens/materialswitch.splash")),
    ("menu", include_str!("../../../components/material/screens/menu.splash")),
    ("musicplayer", include_str!("../../../components/material/screens/musicplayer.splash")),
    ("navigationdrawer", include_str!("../../../components/material/screens/navigationdrawer.splash")),
    ("navigationrail", include_str!("../../../components/material/screens/navigationrail.splash")),
    ("octoswidgets", include_str!("../../../components/material/screens/octoswidgets.splash")),
    ("preferences", include_str!("../../../components/material/screens/preferences.splash")),
    ("progressindicator", include_str!("../../../components/material/screens/progressindicator.splash")),
    ("radiobutton", include_str!("../../../components/material/screens/radiobutton.splash")),
    ("search", include_str!("../../../components/material/screens/search.splash")),
    ("shapetheming", include_str!("../../../components/material/screens/shapetheming.splash")),
    ("sidesheet", include_str!("../../../components/material/screens/sidesheet.splash")),
    ("slider", include_str!("../../../components/material/screens/slider.splash")),
    ("snackbar", include_str!("../../../components/material/screens/snackbar.splash")),
    ("tabs", include_str!("../../../components/material/screens/tabs.splash")),
    ("textfield", include_str!("../../../components/material/screens/textfield.splash")),
    ("timepicker", include_str!("../../../components/material/screens/timepicker.splash")),
    ("topappbar", include_str!("../../../components/material/screens/topappbar.splash")),
    ("transition", include_str!("../../../components/material/screens/transition.splash")),
];

/// The reference's own name for a route -- shared so the toolbar and the index
/// cannot disagree about what a screen is called.
pub fn title_of(route: &str) -> &str {
    match route {
        INDEX => "Catalog",
        "allcomponents" => "All components",
            "adaptive" => "Adaptive layouts",
            "badge" => "Badge",
            "bottomappbar" => "Bottom app bar",
            "bottomnav" => "Bottom navigation",
            "bottomsheet" => "Bottom sheet",
            "button" => "Button",
            "card" => "Card",
            "carousel" => "Carousel",
            "checkbox" => "Checkbox",
            "chip" => "Chip",
            "color" => "Color palette",
            "datepicker" => "Date picker",
            "dialog" => "Dialog",
            "divider" => "Divider",
            "dockedtoolbar" => "Docked toolbar",
            "elevation" => "Elevation",
            "fab" => "Floating action button",
            "floatingtoolbar" => "Floating toolbar",
            "font" => "Typography",
            "imageview" => "Image view",
            "listitem" => "List item",
            "loadingindicator" => "Loading indicator",
            "materialswitch" => "Switch",
            "menu" => "Menu",
            "musicplayer" => "Music player",
            "navigationdrawer" => "Navigation drawer",
            "navigationrail" => "Navigation rail",
            "preferences" => "Preferences",
            "progressindicator" => "Progress indicator",
            "radiobutton" => "Radio button",
            "search" => "Search",
            "shapetheming" => "Shape theming",
            "sidesheet" => "Side sheet",
            "slider" => "Slider",
            "snackbar" => "Snackbar",
            "tabs" => "Tabs",
            "textfield" => "Text field",
            "timepicker" => "Time picker",
            "topappbar" => "Top app bar",
            "transition" => "Transition",
        other => other,
    }
}

/// Is this a route we can actually draw?
///
/// `source_for` falls back to the first screen for anything unknown, which is
/// silent: pushing a typo, or the string `home` that was never a route, drew
/// `adaptive` under the wrong title rather than reporting anything.
pub fn has(route: &str) -> bool {
    route == INDEX || SCREENS.iter().any(|(n, _)| *n == route)
}

/// The route name of the catalog index.
pub const INDEX: &str = "index";

/// The index: one tappable row per screen, in the order the reference lists
/// them. Generated rather than written as a `.splash` file so it cannot drift
/// out of step with `SCREENS`.
fn index_source() -> String {
    let mut rows = String::new();
    for (name, _) in SCREENS {
        // The chevron is now affordance only -- the whole row is tappable and
        // still scrolls, because the target does not capture the finger. See
        // `emit_click_overlay`.
        rows.push_str(&format!(
            "  {{t:\"row\", tapto:\"{name}\", h: 56, aligny: 0.5, fillw: 1, c:[ \
             {{t:\"text\", size: 16, text:\"{}\", fillw: 1}}, \
             {{t:\"text\", size: 16, icon: 1, text:\"\\u{{f054}}\"}} ]}},\n",
            title_of(name)
        ));
    }
    format!("{{t:\"scroll\", c:[ {{t:\"col\", pad: 16, spacing: 0, c: [\n{rows}]}} ]}}")
}

/// The full source for a route: shared kit first, then the screen.
pub fn source_for(route: &str) -> String {
    if route == INDEX {
        return format!("{KIT}\n{}", index_source());
    }
    let body = SCREENS
        .iter()
        .find(|(n, _)| *n == route)
        .or_else(|| SCREENS.first())
        .map(|(_, s)| *s)
        .unwrap_or("");
    format!("{KIT}\n{body}")
}
