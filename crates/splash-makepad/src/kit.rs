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
    with_state_sized(route, dark, t, 0.0, 0.0, kit)
}

/// As [`with_state_at`], plus the viewport size in vp as `st.vw`/`st.vh`.
///
/// A page cannot ask to fill on this backend: `Splash` wraps whatever it mounts
/// in `View{height:Fit, …}` (SPLASH_PREFIX, its own Rust source), so a root
/// asking for `height: Fill` resolves against a Fit parent and collapses — the
/// screen renders blank, measured on device. A Fit wrapper does size to its
/// child, though, so a root with an explicit height fills the window and needs
/// nothing changed upstream.
///
/// Zero means "not known yet" and the kit falls back to Fit, which is what the
/// first mount gets before any window geometry has arrived.
pub fn with_state_sized(
    route: &str,
    dark: bool,
    t: f64,
    vw: f64,
    vh: f64,
    kit: &str,
) -> String {
    format!(
        "let st = {{ route: {route:?}, dark: {}, t: {t}, vw: {vw}, vh: {vh}, \
         backend: \"makepad\" }}\n{kit}",
        u8::from(dark)
    )
}

/// Register the capability surface the kit expects, as a stub.
///
/// `platform_channels` and `pedometer` call `invoke(tool)`. On Splash-OH the
/// bridge installs the real registry; this backend has no capabilities at all,
/// and an unregistered global evaluates to nil — which rendered those screens
/// with their answers silently blank. Answering in words is the honest form.
pub fn register_stub_capabilities(vm: &mut splash_render::makepad_script::ScriptVm) {
    use splash_render::makepad_script::makepad_live_id::*;
    let f = splash_render::add_global_fn(
        vm,
        &[(id!(tool), splash_render::makepad_script::ScriptValue::NIL)],
        |vm, a| {
            let tool = splash_render::string_prop(vm, a, id!(tool)).unwrap_or_default();
            let msg = format!("unavailable on the makepad backend ({tool})");
            vm.bx.heap.new_string_from_str(&msg)
        },
    );
    vm.set_injected_global(id!(invoke), f);

    // State reads the same way it does on ArkUI, from the same store shape.
    // The kit is one file; a control that checks on the phone checks here too.
    let g = splash_render::add_global_fn(
        vm,
        &[
            (id!(key), splash_render::makepad_script::ScriptValue::NIL),
            (id!(dflt), splash_render::makepad_script::ScriptValue::NIL),
        ],
        |vm, a| {
            let key = splash_render::string_prop(vm, a, id!(key)).unwrap_or_default();
            let dflt = splash_render::num_prop(vm, a, id!(dflt)).unwrap_or(0.0);
            splash_render::makepad_script::ScriptValue::from_f64(
                splash_render::state::get_or_seed(&key, dflt),
            )
        },
    );
    vm.set_injected_global(id!(sget), g);
}
