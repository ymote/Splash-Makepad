//! Assemble a directory of `.splash` files into one script.
//!
//! The Splash DSL has no `import`. A kit that spans more than one file is
//! therefore assembled by **concatenation**, and the order is load-bearing: the
//! shared helpers must be defined before the files that call them, and the file
//! that picks a screen must come last so its value is the script's value.
//!
//! The convention this crate fixes:
//!
//! | file | position | holds |
//! |---|---|---|
//! | `_kit.splash` | first | tokens and helpers every screen uses |
//! | everything else | sorted, between | one file per screen group, defining `fn screen_*` |
//! | `_index.splash` | last | the index screen and the route dispatch |
//!
//! Sorting the middle keeps the output byte-identical across platforms, so a
//! host baking the result and a test sweeping it see the same script.
//!
//! Note that within the assembled script every file shares one scope: a helper
//! defined in one file is callable from any other regardless of order, because
//! screens are only *called* after the whole script has evaluated. Order matters
//! for top-level `let` bindings, not for `fn` definitions.

use std::io;
use std::path::Path;

/// The file that must come first, and the one that must come last.
const HEAD: &str = "_kit.splash";
const TAIL: &str = "_index.splash";

/// Concatenate every `.splash` file in `dir` into one script, in the order
/// described above. Each file is preceded by a `// ---- <name>` marker so a
/// parse error in the assembled script can be traced back to its source file.
///
/// Fails if `dir` cannot be read or if either `_kit.splash` or `_index.splash`
/// is missing — without them the script has no helpers or no value.
pub fn concat_kit(dir: &Path) -> io::Result<String> {
    let mut middle = Vec::new();
    let mut has_head = false;
    let mut has_tail = false;

    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("splash") {
            continue;
        }
        match path.file_name().and_then(|n| n.to_str()) {
            Some(HEAD) => has_head = true,
            Some(TAIL) => has_tail = true,
            Some(name) => middle.push(name.to_string()),
            None => {}
        }
    }

    let missing = |what: &str| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} has no {what}", dir.display()),
        )
    };
    if !has_head {
        return Err(missing(HEAD));
    }
    if !has_tail {
        return Err(missing(TAIL));
    }
    // Deterministic across filesystems, which read_dir order is not.
    middle.sort();

    let mut out = String::new();
    let push = |name: &str, out: &mut String| -> io::Result<()> {
        out.push_str("// ---- ");
        out.push_str(name);
        out.push('\n');
        out.push_str(&std::fs::read_to_string(dir.join(name))?);
        out.push('\n');
        Ok(())
    };
    push(HEAD, &mut out)?;
    for name in &middle {
        push(name, &mut out)?;
    }
    push(TAIL, &mut out)?;
    Ok(out)
}

/// Prepend the host state object the kit reads as `st`.
///
/// Every screen reads `st.route` to know what to draw and `st.dark` to pick a
/// palette. A host injects them by prefixing one line, which is also how
/// `kit-host` feeds the Material catalog its state.
pub fn with_state(route: &str, dark: bool, kit: &str) -> String {
    with_state_at(route, dark, 0.0, kit)
}

/// Same, plus a clock.
///
/// The pipeline evaluates the DSL to a tree once per mount, so nothing in a
/// screen can move on its own — there is no tween, no controller, no animator
/// in the vocabulary. What there *is* is the host's frame loop: re-mount with a
/// changing `st.t` and the tree is recomputed against it, which is animation by
/// the only route this design leaves open. `t` is seconds since the app
/// started, as an f64.
///
/// Hosts that never animate can keep calling [`with_state`], which pins t to 0.
pub fn with_state_at(route: &str, dark: bool, t: f64, kit: &str) -> String {
    format!(
        "let st = {{ route: {route:?}, dark: {}, t: {t} }}\n{kit}",
        u8::from(dark)
    )
}
