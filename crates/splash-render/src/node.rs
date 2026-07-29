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
}

impl NodeKind {
    /// Parse the DSL `t` tag. Unknown tags yield `None` (the node is dropped).
    pub fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
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
            _ => return None,
        })
    }

    /// Whether this kind lays its children out along the main axis vertically
    /// (column-like) — a convenience for simple backends.
    pub fn is_vertical_stack(self) -> bool {
        matches!(self, Self::Column | Self::Scroll | Self::List)
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
    pub color: Option<u32>,
    pub bg: Option<u32>,
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
    pub border: Option<f32>,
    pub bordercolor: Option<u32>,
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
