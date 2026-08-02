//! Backend-agnostic UI node model.
//!
//! The Splash DSL evaluates (in the makepad-script VM) to a tree of plain data
//! objects `{t: "...", <attrs>, c: [...]}`. [`crate::build`] walks that into this
//! `UiNode` tree, which carries **no renderer dependency**. Each backend (ArkUI,
//! makepad, …) turns a `UiNode` tree into its own widgets — that is what makes
//! makepad just *one* render backend rather than *the* renderer.

/// Every node type the DSL can name. A backend maps each to one of its widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Column,
    Row,
    Stack,
    Scroll,
    List,
    Grid,
    Waterflow,
    Refresh,
    Swiper,
    Text,
    Image,
    Button,
    Toggle,
    Checkbox,
    Radio,
    Slider,
    Progress,
    Loading,
    Input,
    Textarea,
    DatePicker,
    TimePicker,
    TextPicker,
    /// An OpenStreetMap view — makepad ships a full vector-tile renderer
    /// (`widgets/src/map`, ~12k lines) with rotation and tilt, so a real map is
    /// not a platform view here, it is a widget.
    Map,
    /// The two fragment-shader samples, as compiled MPSL variants in
    /// `splash-widgets`. A DSL node cannot carry shader source — MPSL compiles
    /// at build time — but it can select a shader that was compiled.
    Shader,
    Sdf,
    // ---- Material components -------------------------------------------
    // The reference catalog (Splash-Android) states components *semantically*
    // and lets the renderer produce the real Material widget. These are those
    // nodes; a backend reads `variant` to pick which one of the family it is.
    /// Floating action button — `small` / `regular` / `large` / `extended`.
    Fab,
    /// Icon-only button — `standard` / `filled` / `tonal` / `outlined`.
    IconButton,
    /// A joined single-select strip, choices in `items`, chosen in `selected`.
    Segmented,
    /// `assist` / `filter` / `input` / `suggestion`.
    Chip,
    /// `elevated` / `filled` / `outlined`, optionally checkable.
    Card,
    /// A list row: `label` over `supporting`, one/two/three line by content.
    ListItem,
    /// A rule — `full` / `inset` / `vertical`.
    Divider,
    /// Fixed empty space.
    Spacer,
    /// A row that wraps its children onto further lines.
    Flow,
    /// A shape-scale swatch (`radius`), including the cut-corner family.
    ShapeBox,
    /// A colour-role swatch with its label.
    ColorSwatch,
    /// An icon carrying a count badge.
    BadgeIcon,
    /// The M3 search bar / search view.
    SearchBar,
    /// An exposed dropdown menu.
    Dropdown,
    /// Tab strip — `fixed` / `scrollable`, optionally with icons or badges.
    Tabs,
    /// Bottom navigation bar.
    NavBar,
    /// Navigation rail.
    NavRail,
    /// A carousel strip — `hero` / `multibrowse` / `uncontained` / `fullscreen`.
    Carousel,
    /// A group of radio buttons with one selection.
    RadioGroup,
    /// Two-thumb slider.
    RangeSlider,
    // ---- demo hosts -----------------------------------------------------
    // Composite screens the reference builds as one widget rather than out of
    // parts. They still have to draw something here, or the screens that use
    // them come out blank.
    /// A top app bar — `small` / `medium` / `large` / `center`.
    AppBarDemo,
    /// A bottom app bar with actions and a docked FAB.
    BottomBarDemo,
    /// A docked / floating / vertical toolbar.
    ToolbarDemo,
    /// An adaptive pane layout — `listdetail` / `supporting` / `feed`.
    AdaptiveDemo,
    /// The stage a motion demo animates.
    TransitionHost,
    /// octos-one's own widgets, ported to Android views in the reference.
    WeatherIcon,
    NavMap,
    GlassPanel,
    /// A web surface positioned into the native tree. The host reserves the
    /// space and puts a real WebView there — the hybrid Splash-OH's own cards
    /// use, reached from the DSL.
    Web,
}

impl NodeKind {
    /// Parse the DSL `t` tag. Unknown tags yield `None` (the node is dropped).
    pub fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            // The Android backend's catalog — the reference rendering, and the
            // only one authored against real Material widgets — spells these
            // three differently. They are the same nodes, so accept both rather
            // than fork the screens: `col` is by far the most used tag there (88
            // uses), and every one of them was dropped on the floor here.
            "col" => Self::Column,
            "switch" => Self::Toggle,
            "textfield" => Self::Input,
            "column" => Self::Column,
            "row" => Self::Row,
            "stack" => Self::Stack,
            "scroll" => Self::Scroll,
            "list" => Self::List,
            "grid" => Self::Grid,
            "waterflow" => Self::Waterflow,
            "refresh" => Self::Refresh,
            "swiper" => Self::Swiper,
            "text" => Self::Text,
            "image" => Self::Image,
            "button" => Self::Button,
            "toggle" => Self::Toggle,
            "checkbox" => Self::Checkbox,
            "radio" => Self::Radio,
            "slider" => Self::Slider,
            "progress" => Self::Progress,
            "loading" => Self::Loading,
            "input" => Self::Input,
            "textarea" => Self::Textarea,
            "datepicker" => Self::DatePicker,
            "timepicker" => Self::TimePicker,
            "textpicker" => Self::TextPicker,
            "map" => Self::Map,
            "shader" => Self::Shader,
            "sdf" => Self::Sdf,
            "fab" => Self::Fab,
            "iconbutton" => Self::IconButton,
            "segmented" => Self::Segmented,
            "chip" => Self::Chip,
            "card" => Self::Card,
            "listitem" => Self::ListItem,
            "divider" => Self::Divider,
            "spacer" => Self::Spacer,
            "flow" => Self::Flow,
            "shapebox" => Self::ShapeBox,
            "colorswatch" => Self::ColorSwatch,
            "badgeicon" => Self::BadgeIcon,
            "searchbar" => Self::SearchBar,
            "dropdown" => Self::Dropdown,
            "tabs" => Self::Tabs,
            "navbar" => Self::NavBar,
            "navrail" => Self::NavRail,
            "carousel" => Self::Carousel,
            "radiogroup" => Self::RadioGroup,
            "rangeslider" => Self::RangeSlider,
            "appbardemo" => Self::AppBarDemo,
            "bottombardemo" => Self::BottomBarDemo,
            "toolbardemo" => Self::ToolbarDemo,
            "adaptivedemo" => Self::AdaptiveDemo,
            "transitionhost" => Self::TransitionHost,
            "weathericon" => Self::WeatherIcon,
            "navmap" => Self::NavMap,
            "glasspanel" => Self::GlassPanel,
            "web" => Self::Web,
            _ => return None,
        })
    }

    /// Whether this kind lays its children out along the main axis vertically
    /// (column-like) — a convenience for simple backends.
    pub fn is_vertical_stack(self) -> bool {
        matches!(self, Self::Column | Self::Scroll | Self::List | Self::Card | Self::GlassPanel)
    }
}

/// All attributes a node can carry. Every field is optional; a backend applies
/// the ones it understands and ignores the rest. Colours are `0xAARRGGBB`.
/// `on` / `tap` are left for the backend to resolve against [`NodeKind`] (e.g.
/// `on` means checkbox-select vs toggle-value depending on the kind).
#[derive(Clone, Default, Debug)]
pub struct Attrs {
    pub text: Option<String>,
    pub label: Option<String>,
    pub placeholder: Option<String>,
    /// Makepad widget id (`name := Widget{…}`) so the widget is addressable
    /// (e.g. a signal Label the host reads, or a target of `ui.<id>.set_text`).
    pub id: Option<String>,
    /// Navigate on tap: emits `on_click` that writes the target route into the
    /// `nav_signal` widget, which the host app reads to switch screens.
    pub tapto: Option<String>,
    /// Image source: a resource ref or an `https://` URL.
    pub src: Option<String>,
    /// ObjectFit-style enum for images.
    pub fit: Option<i32>,
    pub w: Option<f32>,
    pub h: Option<f32>,
    /// Force Fit (hug-content) sizing on an axis, overriding the container
    /// default of Fill — for content-sized items like chips and buttons.
    pub fitw: Option<i32>,
    pub fith: Option<i32>,
    /// Force Fill sizing on width even for non-containers (e.g. a full-width
    /// Button used as a navigation list row).
    pub fillw: Option<i32>,
    /// Force Fill sizing on height (e.g. a themed page that must cover the
    /// viewport, not just hug its content).
    pub fillh: Option<i32>,
    pub size: Option<f32>,
    pub weight: Option<i32>,
    /// Render this text in the theme's icon font (Font Awesome) so a codepoint
    /// like `\u{f002}` paints a monochrome Material-style icon, not a colour emoji.
    pub icon: Option<i32>,
    /// The same `icon` key read as a *name* (`"add"`, `"favorite"`, …). The
    /// reference catalog names its icons rather than spelling codepoints, so the
    /// backend resolves the name to a glyph in whatever icon font it has.
    pub icon_name: Option<String>,
    pub color: Option<u32>,
    pub bg: Option<u32>,
    /// The far end of a two-stop gradient fill (makepad's `draw_bg.color_2`).
    /// The reference's shapeable images are named gradients, not bitmaps.
    pub bg2: Option<u32>,
    pub radius: Option<f32>,
    /// Material elevation (dp). Non-zero promotes a filled container to a
    /// shadow-casting view and scales its drop shadow.
    pub elevation: Option<f32>,
    pub pad: Option<f32>,
    /// Asymmetric padding: horizontal (`padx`) / vertical (`pady`), each
    /// overriding `pad` on its axis — for M3 insets like a button's 24dp
    /// horizontal / 6dp vertical padding that a uniform `pad` can't express.
    pub padx: Option<f32>,
    pub pady: Option<f32>,
    pub spacing: Option<f32>,
    pub margin: Option<f32>,
    /// Per-axis margin, overriding `margin` on its axis. Every section heading in
    /// the reference carries `marginy: 4`; dropping it made each screen sit a
    /// little tighter than the reference and drift further down the page.
    pub marginx: Option<f32>,
    pub marginy: Option<f32>,
    pub border: Option<f32>,
    pub bordercolor: Option<u32>,
    /// Which Material variant this node is — `filled`/`tonal`/`outlined`/`text`/
    /// `elevated` on a button, `small`/`large`/`extended` on a FAB, the type role
    /// on a text node. The reference catalog (Splash-Android, rendered with real
    /// `com.google.android.material.*` views) states components *semantically*
    /// this way rather than drawing look-alikes, and it is the single most used
    /// attribute there. Without it every variant collapses to one appearance.
    pub variant: Option<String>,
    /// Disabled state — the reference devotes a whole section per component to it.
    pub enabled: Option<i32>,
    /// The state slot this widget reads and writes. The reference screens are
    /// state-driven: a widget event writes `key`, the DSL is re-evaluated, and
    /// the new tree rebuilds the views — the DSL, not the host, decides what the
    /// screen says. Without it a catalog is a picture rather than a demo.
    pub key: Option<String>,
    /// What this widget asks the host to do when tapped — `"alert"`, `"modal"`,
    /// `"range"`. The reference writes it into `key`, and the host turns it into
    /// a real dialog / sheet / picker. Paired with [`Attrs::key`].
    pub action: Option<String>,
    /// `;`-separated choices, for segmented buttons and tab strips.
    pub items: Option<String>,
    /// Index of the chosen item in `items`.
    pub selected: Option<i32>,
    /// A field's floating label, and the supporting/error text beneath it.
    pub hint: Option<String>,
    /// A card's headline, and a list row's line count when content alone is
    /// ambiguous (M3: 56 / 72 / 88dp for one / two / three lines).
    pub title: Option<String>,
    pub lines: Option<i32>,
    /// A badge's contents; `count` is the numeric form.
    pub badge: Option<String>,
    pub count: Option<i32>,
    pub supporting: Option<String>,
    /// A field's supporting line (the reference spells it `helper`).
    pub helper: Option<String>,
    /// A checkbox in its third state — neither on nor off.
    pub indeterminate: Option<i32>,
    /// The Material role a swatch names (`"primary"`, `"onSurfaceVariant"`).
    pub group: Option<String>,
    /// A slider's range and step, and a range slider's second thumb.
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub value2: Option<f32>,
    pub error: Option<String>,
    /// A native control's selected/active role (M3 primary): the checked box, the
    /// radio dot, the switch's on-track, the slider's value track and handle, a
    /// focused field's outline. Distinct from `bg`, which stays the *container*
    /// (an unchecked track, a filled field) as it does on every other node.
    ///
    /// Controls need this because they are drawn by their own shaders, whose
    /// colours are not reachable from `bg`/`color` alone. The theming a widget
    /// kit registers on the app VM never reaches them either: `Splash` mounts its
    /// body on an isolate VM that only ever gets makepad's own `script_mod`
    /// (`widget_async.rs`), so the kit's variants are simply absent there. What
    /// does arrive is whatever the mounted dialect string carries — verified on
    /// device — so a control states its Material roles per instance.
    pub accent: Option<u32>,
    /// The ink drawn *on* `accent` (M3 on-primary): a checkbox's tick, a switch's
    /// thumb when on.
    pub markcolor: Option<u32>,
    pub value: Option<f32>,
    pub total: Option<f32>,
    pub align: Option<i32>,
    /// Child alignment within a container, 0.0..=1.0 on each axis.
    pub alignx: Option<f32>,
    pub aligny: Option<f32>,
    pub on: Option<i32>,
    pub tap: Option<i32>,
    /// Map camera. `tilt` is what makes the view 2.5D; `rotation` is the bearing.
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub zoom: Option<f64>,
    pub tilt: Option<f64>,
    pub rotation: Option<f64>,
    /// Absolute position for a surface the host composites (a web slot). The
    /// tree does not know where a node lands, so a screen that wants one says.
    pub x: Option<f64>,
    pub y: Option<f64>,
}

/// One node in the backend-agnostic tree.
#[derive(Clone, Debug)]
pub struct UiNode {
    pub kind: NodeKind,
    pub attrs: Attrs,
    pub children: Vec<UiNode>,
}

impl UiNode {
    /// Total node count including self — handy for tests and diagnostics.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(UiNode::count).sum::<usize>()
    }
}
