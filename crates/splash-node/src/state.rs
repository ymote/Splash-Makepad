//! The thing that makes a screen more than a picture.
//!
//! A `.splash` screen is evaluated to a tree once per mount. Nothing survives
//! that, so until now the only state a screen could have was the four fields
//! the host injects as `st` — route, palette, clock, backend — and the only
//! thing a tap could do was change the route. That is why the catalog's
//! checkboxes never checked, its steppers never stepped and its filter chips
//! never filtered: not eighteen separate omissions, one missing store.
//!
//! This is that store. Keys are strings the kit picks, values are numbers,
//! because a number covers everything a control here needs to remember —
//! checked, selected index, a count, a slider position, which of N tabs.
//!
//! Reached from the DSL as `sget(key, default)`, written by a tap that names an
//! action instead of a route:
//!
//! ```text
//! {t: "row", tapto: "set:signin_remember=!", c: [...]}   // toggle
//! {t: "row", tapto: "set:guests=+1", c: [...]}           // step
//! {t: "row", tapto: "set:planner_tab=~3", c: [...]}      // cycle 0,1,2
//! ```
//!
//! It lives here rather than in either backend because both evaluate the same
//! kit and both should behave the same way: a checkbox that checks on the phone
//! is a checkbox that checks on the desktop.

use std::collections::BTreeMap;
use std::sync::Mutex;

static STATE: Mutex<BTreeMap<String, f64>> = Mutex::new(BTreeMap::new());

/// The value at `key`, or `dflt` when nothing has written one.
///
/// Defaulting here rather than in the DSL is deliberate. The alternative is
/// injecting the whole map as an object and letting a screen read a missing
/// property, which yields nil — and nil then flows into an argb() or a width
/// and the failure surfaces somewhere else entirely as a blank box.
pub fn get(key: &str, dflt: f64) -> f64 {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.get(key).copied())
        .unwrap_or(dflt)
}

/// The value at `key`, seeding it with `dflt` if nothing has written one.
///
/// Seeding on read, not just defaulting, and that distinction is the whole
/// reason this is a separate function. `apply` has to toggle against the
/// *current* value, and it has no idea what a screen considers its default —
/// so a control declared `sget("m3_switch", 1)` toggled from an assumed 0 to 1
/// and rendered exactly the same as before. The tap did something; the screen
/// could not tell. The first render is what teaches the store what the
/// defaults are.
pub fn get_or_seed(key: &str, dflt: f64) -> f64 {
    if let Ok(mut s) = STATE.lock() {
        return *s.entry(key.to_string()).or_insert(dflt);
    }
    dflt
}

pub fn set(key: &str, v: f64) {
    if let Ok(mut s) = STATE.lock() {
        s.insert(key.to_string(), v);
    }
}

/// Apply an action from a `tapto: "set:…"` target. Returns false if it is not
/// one, so the caller can fall through to routing.
///
/// The form is `key=rhs`, where rhs is one of:
///
/// | rhs    | meaning                                    |
/// |--------|--------------------------------------------|
/// | `!`    | toggle between 0 and 1                     |
/// | `+n`   | add n                                      |
/// | `-n`   | subtract n                                 |
/// | `~n`   | increment, wrapping at n                   |
/// | `^n`   | increment, clamped at n                    |
/// | `v n`  | anything else parses as the literal value   |
///
/// A relative form matters more than it looks: `+1` needs the old value, and a
/// tap target is a *string baked into the tree at build time*. If a stepper had
/// to encode its next value it would encode the value that was current when the
/// tree was built, so the second tap on a stale tree would undo the first.
pub fn apply(action: &str) -> bool {
    let Some(body) = action.strip_prefix("set:") else {
        return false;
    };
    let Some((key, rhs)) = body.split_once('=') else {
        return false;
    };
    let key = key.trim();
    let rhs = rhs.trim();
    let cur = get(key, 0.0);
    let next = match rhs.chars().next() {
        Some('!') => {
            if cur == 0.0 {
                1.0
            } else {
                0.0
            }
        }
        Some('+') => cur + rhs[1..].parse::<f64>().unwrap_or(1.0),
        Some('-') => cur - rhs[1..].parse::<f64>().unwrap_or(1.0),
        Some('~') => {
            let n = rhs[1..].parse::<f64>().unwrap_or(1.0);
            if n <= 0.0 {
                0.0
            } else {
                (cur + 1.0) % n
            }
        }
        Some('^') => {
            let n = rhs[1..].parse::<f64>().unwrap_or(0.0);
            (cur + 1.0).min(n)
        }
        _ => match rhs.parse::<f64>() {
            Ok(v) => v,
            Err(_) => return false,
        },
    };
    set(key, next);
    true
}

/// Forget everything. For tests, which would otherwise leak state between them.
pub fn reset() {
    if let Ok(mut s) = STATE.lock() {
        s.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The store is global, and `cargo test` runs these on parallel threads —
    /// so without this one test's `reset()` wipes another's keys mid-assert.
    /// Found the honest way: an intermittent failure that only appeared once
    /// there were two tests calling reset.
    static SERIAL: Mutex<()> = Mutex::new(());

    #[test]
    fn actions_do_what_they_say() {
        let _s = SERIAL.lock();
        reset();
        assert!(apply("set:a=!"));
        assert_eq!(get("a", 0.0), 1.0);
        assert!(apply("set:a=!"));
        assert_eq!(get("a", 0.0), 0.0);

        assert!(apply("set:n=+2"));
        assert!(apply("set:n=+2"));
        assert_eq!(get("n", 0.0), 4.0);
        assert!(apply("set:n=-1"));
        assert_eq!(get("n", 0.0), 3.0);

        // ~3 wraps: 0 1 2 0
        reset();
        for want in [1.0, 2.0, 0.0] {
            apply("set:t=~3");
            assert_eq!(get("t", 0.0), want);
        }

        // ^2 clamps rather than wrapping.
        reset();
        for want in [1.0, 2.0, 2.0] {
            apply("set:c=^2");
            assert_eq!(get("c", 0.0), want);
        }

        assert!(apply("set:lit=7"));
        assert_eq!(get("lit", 0.0), 7.0);

        // A route is not an action.
        assert!(!apply("date_planner/maya"));
        assert!(!apply("index"));
        // Malformed rhs is not an action either, so it cannot silently zero a
        // key that a screen is reading.
        assert!(!apply("set:x=banana"));
    }

    #[test]
    fn a_toggle_flips_a_control_whose_default_is_on() {
        let _s = SERIAL.lock();
        reset();
        // What a screen declaring `sget("k", 1)` does on first render.
        assert_eq!(get_or_seed("k", 1.0), 1.0);
        apply("set:k=!");
        assert_eq!(get_or_seed("k", 1.0), 0.0, "toggling an on-by-default control must turn it off");
        apply("set:k=!");
        assert_eq!(get_or_seed("k", 1.0), 1.0);
    }

    #[test]
    fn a_default_is_returned_only_when_nothing_was_written() {
        let _s = SERIAL.lock();
        reset();
        assert_eq!(get("missing", 3.0), 3.0);
        set("missing", 0.0);
        assert_eq!(get("missing", 3.0), 0.0);
    }
}
