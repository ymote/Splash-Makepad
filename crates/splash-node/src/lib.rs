//! The Splash UI node model — the branch point.
//!
//! `ui-profile-l0.md` §1.1: a card lowers to a theme's component kit, the kit
//! evaluates to a [`UiNode`] tree, and each backend turns that tree into its own
//! widgets. **`UiNode` is where the paths diverge**, and anything that lowers
//! past it reaches one platform and silently abandons the others.
//!
//! A branch point has to be adoptable, which is why this crate depends on
//! nothing at all. `splash-render` evaluates the DSL and needs a VM; a host that
//! already has its own VM needs the *model*, not the evaluator. Before this
//! split, taking the model meant taking `makepad-script` too — and octos-one
//! could not, because `makepad-error-log v1.0.0` then existed at two paths and
//! Cargo refuses to write that lockfile.
//!
//! So the division is: **this crate is the contract, `splash-render` is one way
//! of producing it.** A second host writes its own evaluator against its own VM
//! and produces the same tree.

mod node;
pub mod state;

pub use node::{Attrs, NodeKind, UiNode};
