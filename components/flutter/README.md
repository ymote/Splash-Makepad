# flutter/samples, on Splash + makepad

> ## These are illustrations, not ports. Read this first.
>
> An independent review (OpenAI Codex, `gpt-5.6-terra`, read-only) was asked
> whether this is a port of Flutter's widgets or a set of static pictures that
> resemble them. Its verdict, which is correct:
>
> > *"mostly static pictures resembling Flutter widgets, plus limited route
> > navigation — not faithful Flutter ports."*
>
> The numbers back it up. Of 285 explicit nodes here, **244 (86%) are
> `column`/`row`/`scroll`**, and there is **not one `{t:"button"}` node**. Every
> button, FAB, chip, tab, list row and app-bar action is a styled container.
>
> That is a property of the stack, not of effort. `NodeKind` has 23 tags and
> `Attrs` has 34 scalar fields, and between them **zero** concepts for
> `onPressed`, state (`pressed`/`hovered`/`focused`/`disabled`/`selected`),
> animation, or semantics. A Flutter widget essentially *is* that state machine
> — `WidgetStateProperty` resolved against a state set and driven by an
> `AnimationController`. Matching `Size(64,40)` and `StadiumBorder` reproduces
> one row of a table whose other dimension this DSL cannot express.
>
> Two concrete defects the review surfaced, both verified:
>
> - **The makepad emitter never forwards `on`, `value` or `total`.** So
>   `{t:"checkbox", on: 1}` renders an *unchecked* box and `{t:"slider",
>   value: 0.35}` sits at its default — while the caption beside it says 0.35.
>   The controls are real native widgets; their declared state never reaches
>   them. Not yet fixed.
> - **One "exact" metric was wrong.** The notched CupertinoListTile subtitle is
>   `_kNotchedSubtitleFontSize` = **14**, not the base tile's `_kSubtitleFontSize`
>   = 12 (`cupertino/list_tile.dart:44-46,349`). Fixed.
>
> What this directory honestly is: **a static frame of each screen**, with real
> layout, real colour tokens, real content from the samples, and route
> navigation between screens. What it is not: working widgets. Read every
> "ported" below as "drawn".


Every directory of [flutter/samples](https://github.com/flutter/samples) has a
`.splash` file here — 27 of them, authored in the Splash DSL and rendered as
**native makepad widgets**. 108 routes, each verified by
`crates/splash-makepad/tests/flutter_samples.rs`.

```sh
cargo test -p splash-makepad          # sweep all 108 routes, no device needed
cargo run  -p flutter-samples         # run the catalog
```

## What ported, and what did not

Eleven of the 27 directories are apps with a UI to draw. They are ported: 92
screens, with the samples' real content — the M3 type scale at its actual sp
values, the six elevation levels with their dp and surface-tint percentages, all
nine `date_planner` events with their task lists, the four `libraryInstance`
books, the real `destinations.json` entries, the seven `platform_design`
settings rows.

The other sixteen exist to demonstrate Flutter's **platform integration** —
embedding into a host app, plugins, FFI, GLSL shaders, build tooling. There is
nothing to draw, so each gets a screen stating what the sample teaches and
exactly why it does not port, rather than an invented UI.

| directory | screens | notes |
|---|---|---|
| `material_3_demo` | 4 | Components, Color, Typography, Elevation |
| `cupertino_gallery` | 23 | all 21 widget pages, plus Widgets/Settings tabs |
| `animations` | 21 | index plus all 20 demos, as still frames — see below |
| `navigation_and_routing` | 13 | Popular/New/All, authors, details, settings, sign-in |
| `date_planner` | 10 | the four Period buckets, all nine events |
| `platform_design` | 5 | four Material tabs plus the iOS chrome |
| `compass_app` | 5 | Home, Search, Results, Activities, Booking |
| `form_app` | 5 | index plus the four form demos |
| `desktop_photo_search` | 2 | both the Material and fluent_ui variants |
| `testing_app` | 2 | home and favourites |
| `dynamic_theme` | 1 | chat surface; the theme toggle is real |
| 16 others | 16 | `add_to_app`, `platform_channels`, `simple_sdf`, … |

## The honest limits

These are **visual ports**. The pipeline evaluates the DSL to a tree once per
mount; there is no per-component state, no async, no HTTP, no navigation stack
and no animation. So:

- **`animations`** ports its index and all 20 titles, but not the animations.
  The DSL has no tween, curve or controller. Two demos *are* live, because
  makepad's own shaders drive them off draw time: the circular spinner and the
  M3 shape-morph indicator.
- **`compass_app`** ports its five screens but not its architecture, which is
  most of the sample — MVVM, repositories, use-cases, DI, offline-first store.
- **`desktop_photo_search`** ports the split-pane shape; the Unsplash search
  needs an HTTP client the pipeline does not have. The tiles are placeholders.
- **`form_app`** shows the real validation messages in place rather than
  triggering them — there is no `FormState` to validate.
- **`navigation_and_routing`** makes every screen reachable, but by re-mount,
  not by routing. `tapto` is a one-string signal, not a route table.
- **`simple_sdf` / `simple_shader`** are the interesting near-misses: makepad
  draws every widget with MPSL and has a first-class `Sdf2d` API, but no DSL
  node carries shader source, and MPSL compiles at build time — a runtime string
  is never compiled. Reaching them means a compiled variant in `splash-widgets`
  that a DSL node selects by name.

Anything a screen cannot honestly render says so on the screen.

## How the kit is assembled

The DSL has no `import`, so the kit is **concatenated**, in an order
`splash_makepad::kit` fixes:

| file | position | holds |
|---|---|---|
| `_kit.splash` | first | M3 and iOS tokens, chrome helpers, the "no analogue" screen |
| one per sample | sorted, between | `fn screen_*` for that sample |
| `_index.splash` | last | the index and the route dispatch |

`splash_makepad::kit::concat_kit` is what the route-sweep test and the
`assemble` example both call. The **app** bakes the same files with
`include_str!` instead, because `cargo-makepad` compiles the Android build
inside a generated wrapper crate that never runs the app's build script —
`OUT_DIR` is undefined there and the build fails to compile. A relative
`include_str!` resolves against the source file and works on both targets. The
cost is a file list spelled out in `main.rs`, which
`baked_kit_matches_the_directory` pins to the directory so it cannot drift.

Assemble a kit by hand with:

```sh
cargo run -p splash-makepad --example assemble -- components/flutter \
    --route date_planner/maya | cargo run -p splash-makepad --example translate -- /dev/stdin
```

## Two things the makepad-script VM does that shape these files

Both cost real debugging time and are easy to hit again:

1. **A bare function call as the final top-level expression evaluates to nil.**
   `screen_index()` on the last line yields nothing and `build` returns `None`.
   It must be bound first — `let node = screen_index()`, then `node`. Wrapping
   in `if` or an object literal also works.
2. **There is no substring or `startsWith`.** Parameterised routes
   (`date_planner/maya`) are matched by rebuilding the full route string from
   the same data the screen renders and comparing for equality — see the `for`
   loops at the bottom of `_index.splash`.

## Three that were wrongly written off

`google_maps`, `platform_channels` and `pedometer` each carried a "no analogue"
screen. All three were wrong, and wrong the same way: they judged what the
**makepad DSL** could express in isolation, rather than what this project has.

- **google_maps** — makepad ships a ~12k-line OpenStreetMap vector-tile
  renderer with rotation and tilt (`widgets/src/map`). A map is a widget here,
  not a platform view. Now a real map at the sample's own camera, plus a 2.5D
  view.
- **platform_channels** — `splash_render::build` has always taken a `register`
  hook for injecting host functions, and Splash-OH's weather card already used
  it. The bridge carries ~45 capabilities. `invoke(tool)` now reaches that
  registry, installed by the bridge at mount so the renderer still does not
  depend on it. On device the screen shows the real answers.
- **pedometer** — the FFIgen/JNIgen half has no counterpart, but the app is a
  step counter over a platform sensor, and Splash-OH has `sensor::list` /
  `sample` / `stream`.

The remaining thirteen look genuinely inert to me — lint config, an Android
launch screen, an Xcode target, a UIKit technique, repo docs, CI tooling, a
sample deleted upstream. Two are near-misses I have not done: `simple_sdf` and
`simple_shader` need a compiled MPSL variant in `splash-widgets` that a DSL node
selects by name, and `web_embedding` could use Splash-OH's web slots
(`webslot::declare`). Treat the count as "not yet", not "impossible".

## Visual QA

Every screen was driven onto a real device (OnePlus 6T, Android) and looked at.
`tools/visual-qa.sh` writes each route to a file the app polls, screenshots the
result over adb, and builds labelled contact sheets:

```sh
cargo makepad android run -p flutter-samples --release   # once
tools/visual-qa.sh                                       # 108 screens
tools/visual-qa.sh cupertino                             # or a subset
DARK=1 tools/visual-qa.sh                                # dark palette
```

This matters because the route sweep cannot see any of it. A screen whose
control never binds its shader, whose container collapses to zero height, or
whose Label clips its own descenders translates perfectly and passes every
assertion. **Nine defects were found by looking that no test caught:**

| what | why it happened |
|---|---|
| every screen blank | `page()` asked for `height: Fill` inside the host's `Splash{height: Fit}` |
| descenders sheared off every label | hand-picked text heights sat just under the font's line box |
| paragraphs clipped mid-sentence | a Label with no width does not wrap; one with a guessed height clips the wrapped lines |
| all three pickers empty white boxes | makepad has no picker widget, so `datepicker`/`timepicker`/`textpicker` fall through the translator to a bare `View` |
| chat bubbles one pixel wide | a `fillw` paragraph inside a `fitw` column — Fill resolving against Fit |
| list rows overlapping | fixed row heights, once the text inside started measuring itself |
| long titles truncated | app-bar and nav-bar titles were single-line |
| bullet dashes floating mid-sentence | rows centre their children by default |
| `icon()` calls silently losing their colour | a bad mechanical edit of mine — see the arity test |

The fixes are all one rule: **let content measure itself**. `txt`, `para` and
`icon` carry no height, rows that hold them are `fith`, and vertical space comes
from the parent's `spacing`/`pad`. Fixed heights are now only used where the box
really is fixed — a 30px colour tile, a 96px thumbnail.

### What still deviates from Flutter, and why

These are structural, not fixable by editing `.splash`:

- **Controls are makepad's, not Material's.** `CheckBox`, `RadioButton`,
  `Toggle` and `Slider` are drawn by makepad's own MPSL shaders. They render and
  they work; they do not look like Material or Cupertino. Restyling them is what
  `splash-widgets` is for — that is the fork-free theming this repo already
  demonstrates, just not yet applied to these kits.
- **No ripples, no elevation tint overlays, no state layers.** `elevation` maps
  to a drop shadow only.
- **Icons are Font Awesome**, not Material Symbols or SF Symbols, because that
  is the face the theme ships.
- **The pickers are drawn, not native** — see `c_wheel` in `_kit.splash`. They
  do not spin.
- **Nothing animates** beyond the two makepad widgets whose shaders run off draw
  time.

## Making taps work at all

Nothing in the kit was clickable, and the reason was two separate defects that
each silently swallowed the tap.

**1. A `View` ignores `on_click`.** `on_click` is a `ScriptFnRef` field on
`Button`, `CheckBox` and `GlassPanel` and nowhere else. Every `tapto` on a row,
card or list item emitted a property that the resulting `View` parsed and threw
away. `Button` in turn takes no children, so the tappable region cannot simply
*be* a Button without losing the icon, the two text lines and the chevron.

The translator now wraps any tappable container in an `Overlay` view holding the
original content plus a transparent, content-sized `Button` on top — the Button
owns the hit area and the callback, the content underneath is untouched. Pinned
by `a_tappable_container_gets_a_button_over_it`.

Two details of that emitted Button are load-bearing, both found the hard way:
one property per line (the comma-joined form does not parse), and **no `draw_bg`
override** — merging `border_size` into the themed button shader, which has no
such instance, kills the whole widget silently.

**2. `nav_signal` has to live inside the mounted tree.** Upstream's `Splash`
mounts on an isolate VM, and makepad injects `ui` into that VM *resolved against
the splash's own view root* (`inject_splash_ui_handle` in
`widgets/src/widget_async.rs`, which returns early for the main VM). So
`ui.nav_signal` inside a handler could never see the host's `nav_signal` Label,
which is a sibling of the `Splash` widget rather than inside it. `page()` now
emits its own hidden `nav_signal`; the host's widget search does descend into
the mounted subtree, so it still reads it.

This is the same isolate-VM constraint as [the one upstream
PR](../../README.md#the-one-upstream-pr). With `isolate: false` the workaround
stops being load-bearing, but it stays correct either way.

> Note for anyone debugging this: the hot-reload path replaces the `.splash`
> **data** only. The translator is Rust compiled into the binary, so a change to
> the emitted dialect needs a rebuild and reinstall — pushing a new kit will not
> show it. That cost an hour of chasing a fix that was already correct.

## The bug this port found

Every route is asserted on a string only that screen emits, because the router
falls through to the index for anything it does not recognise — so "it rendered"
proves nothing. That check caught a real defect in the translator, not in these
files:

> A node with `elevation` set was promoted to `RoundedShadowView`, but `emit`
> decided whether to recurse into children by matching the widget name against
> `"View"` or `"RoundedView"`. `RoundedShadowView` matched neither, so **every
> raised container silently dropped its children** — a Material elevated card
> with content rendered as an empty shadow box.

The Material catalog never hit it because it only ever puts `elevation` on an
empty tonal swatch. Child emission is now decided by the node's *kind* rather
than by the concrete widget; `a_raised_container_keeps_its_children` in
`crates/splash-makepad/src/lib.rs` pins it.
