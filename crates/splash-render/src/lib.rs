//! Backend-agnostic Splash renderer core.
//!
//! [`build`] evaluates the Splash DSL in the **renderer-free** makepad-script VM
//! and returns a [`UiNode`] tree. Backends (ArkUI, makepad, …) turn that tree
//! into their own widgets, which is what makes makepad *one* render backend
//! rather than *the* renderer. Nothing here depends on makepad-platform,
//! makepad-draw, or any widget crate — only on the VM.
//!
//! ```no_run
//! let src = r#"{t:"column", bg: 4278190080, c:[ {t:"text", text:"hi", h: 20} ]}"#;
//! let tree = splash_render::build(src, |_vm| {}).unwrap();
//! assert_eq!(tree.kind, splash_render::NodeKind::Column);
//! ```

mod eval;

pub use eval::{add_global_fn, build, num_prop, prop, string_prop};

/// The node model, re-exported.
///
/// It lives in `splash-node` now, which depends on nothing at all: a branch
/// point that drags a VM lineage with it cannot be adopted by a host that
/// already has one, and octos-one could not take this crate for exactly that
/// reason. Re-exported so every existing consumer of `splash_render::UiNode`
/// keeps working — the split is a packaging change, not an API one.
pub use splash_node::{state, Attrs, NodeKind, UiNode};

/// Re-exported so backends and hosts can name VM types (for capability
/// registration) without taking their own makepad-script dependency/version.
pub use makepad_script;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_a_simple_tree() {
        // A column with two text children — the DSL is real script, evaluated
        // by the VM (note the arithmetic colour, since hex literals are 0 here).
        let src = r#"
            fn argb(a,r,g,b){ return ((a*256+r)*256+g)*256+b }
            {t:"column", bg: argb(255,20,20,20), pad: 12, c: [
                {t:"text", text:"Splash", size: 20, weight: 7, color: argb(255,255,255,255), w: 120, h: 28},
                {t:"text", text:"on the shared VM", size: 14, color: argb(255,200,200,200), w: 200, h: 20},
            ]}
        "#;
        let tree = build(src, |_vm| {}).expect("evaluates");
        assert_eq!(tree.kind, NodeKind::Column);
        assert_eq!(tree.attrs.pad, Some(12.0));
        assert_eq!(tree.attrs.bg, Some(0xFF141414));
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].kind, NodeKind::Text);
        assert_eq!(tree.children[0].attrs.text.as_deref(), Some("Splash"));
        assert_eq!(tree.children[0].attrs.weight, Some(7));
        assert_eq!(tree.count(), 3);
    }

    #[test]
    fn loops_and_helpers_run_in_the_vm() {
        // Proves the tree is *computed*, not literal: a while-loop builds rows.
        let src = r#"
            let kids = []
            let i = 0
            while i < 3 { kids.push({t:"row", h: 40, c: [ {t:"text", text:"row " + i, w: 80, h: 20} ]}); i = i + 1 }
            {t:"column", c: kids}
        "#;
        let tree = build(src, |_vm| {}).expect("evaluates");
        assert_eq!(tree.children.len(), 3);
        assert_eq!(
            tree.children[2].children[0].attrs.text.as_deref(),
            Some("row 2")
        );
    }

    #[test]
    fn unknown_root_tag_is_none() {
        assert!(build(r#"{t:"nope"}"#, |_vm| {}).is_none());
    }
}
