//! Assemble a directory of `.splash` files into one script and print it.
//!
//! The DSL has no `import`, so a multi-file kit is concatenated in a fixed
//! order (see [`splash_makepad::kit`]). This is the same call `build.rs` makes,
//! exposed so a kit can be assembled by hand — to push over for hot reload, or
//! to feed the `translate` example:
//!
//! ```text
//! cargo run -p splash-makepad --example assemble -- components/flutter \
//!     > /tmp/flutter_kit.splash
//! adb push /tmp/flutter_kit.splash /data/local/tmp/flutter_samples.splash
//! ```
//!
//! With `--route <name>` the host's `st` line is prepended too, so the output
//! is directly runnable through `translate`:
//!
//! ```text
//! cargo run -p splash-makepad --example assemble -- components/flutter \
//!     --route date_planner/maya > /tmp/one.splash
//! cargo run -p splash-makepad --example translate -- /tmp/one.splash
//! ```

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap_or_else(|| {
        eprintln!("usage: assemble <kit-dir> [--route <route>] [--dark]");
        std::process::exit(2);
    });

    let mut route: Option<String> = None;
    let mut dark = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--route" => route = args.next(),
            "--dark" => dark = true,
            other => {
                eprintln!("unexpected argument {other:?}");
                std::process::exit(2);
            }
        }
    }

    let kit = match splash_makepad::kit::concat_kit(&PathBuf::from(&dir)) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("cannot assemble {dir}: {e}");
            std::process::exit(1);
        }
    };

    match route {
        Some(r) => print!("{}", splash_makepad::kit::with_state(&r, dark, &kit)),
        None => print!("{kit}"),
    }
}
