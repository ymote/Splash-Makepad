# flutter/samples, on Splash + makepad

> ## These are illustrations, not ports. Read this first.
>
> An independent review (OpenAI Codex, read-only) was asked whether this is a
> port of Flutter's widgets or static pictures that resemble them. Its verdict,
> which is correct:
>
> > *"mostly static pictures resembling Flutter widgets, plus limited route
> > navigation — not faithful Flutter ports."*
>
> Most of the kit's nodes are layout containers, and the DSL has no `onPressed`,
> no state model and no animator. The screens are faithful in content and
> geometry; they are not working widgets. Read every "ported" below as "drawn".
>
> The exceptions are worth naming, because they are real: the animation demos
> animate, the map is a real OpenStreetMap renderer, and the capability screens
> read live values off the device.

## What an outside review found

The index used to mark all 27 with a check. That was wrong, and this is the
correction. gpt-5.6-terra was given both repos read-only and asked to classify
every screen by what the code *does*, ignoring the comments — which in this
codebase are long and persuasive and were part of the problem. Its verdict:

| | count | what it means |
|---|---|---|
| works | 3 | live host data, or it actually moves |
| drawn only | 18 | geometry and tokens transcribed from the sample, nothing behind them |
| notes | 6 | prose. No port |

The index now says exactly that, in three sections, with a check only on the
three that work. The screens themselves are unchanged in that respect: what
changed is that the catalog stopped claiming otherwise.

Four things it found that were not just overclaiming:

- **`web_embedding` crashed the app.** SIGSEGV on the screen the index marked
  done. The `web` node's `src` was being applied as an image source on a Column
  and its slot width — a device pixel count — as a node width in vp, and ArkUI
  dereferenced a null frame node inside `SetWidth` rather than clamping.
- **The web overlay leaked.** Slots were never reset per build, so after
  visiting that screen the WebView floated over every screen after it.
- **`google_maps` drew nothing on ArkUI** while its own text said "rendered from
  OpenStreetMap vector tiles". The `map` tag hits the walker's unknown-tag arm.
- **`compass_app` never touched the location stack**, which was sitting in
  Splash-OH complete and unreachable — permissions declared, `location::get`
  written, and no way for a DSL screen to call it.

All four are fixed and verified on the device. The map now loads real OSM raster
tiles on ArkUI; the web surface is a real ArkWeb component and says what it can
and cannot do; the compass reads the platform's location switch and position.

## The "no analogue" screens are gone

Every one of the 27 directories now has a screen that says something true about
this stack. The old banner — *"No Splash+makepad analogue"* — was wrong six
times, and always for the same reason: it judged what the **makepad DSL** could
express rather than what the project has.

What it got wrong, and what was actually there:

| sample | the claim | what was true |
|---|---|---|
| `google_maps` | "a platform view; no widget tree describes a map" | makepad ships a 12k-line OpenStreetMap renderer with tilt |
| `platform_channels` | "there is no channel in the render pipeline" | `build` always took a `register` hook; the bridge has ~45 capabilities |
| `pedometer` | "needs a platform sensor API" | Splash-OH has `sensor::list`/`sample`/`stream` |
| `asset_transformation` | "Cargo has no asset pipeline" | `splash://` resolves a request to bytes, one generated |
| `add_to_app` | "no FlutterEngine equivalent" | this app *is* add-to-app, inverted |
| `web_embedding` | "no hostElement to embed into" | `webslot::declare` composites a WebView into the native tree |

The rest are configuration, tooling or prose in **both** repos — shared lints,
a launch window, an Xcode target, repo docs, CI tooling, a sample deleted
upstream. Those now name their counterpart (HarmonyOS atomic services are the
App Clip concept; `tests/flutter_samples.rs` is the CI walker) instead of being
dismissed.

The last two, `simple_shader` and `simple_sdf`, are done as well, and the way
they closed says the same thing as the six above. A fragment shader is a
function from a coordinate to a colour, and an SDF is arithmetic — so the DSL
evaluates them itself, once per cell instead of once per pixel, and emits a
grid of ordinary nodes. `sdHeart` needed `sqrt`, `min`, `sign` and
`smoothstep`, which the VM does not have; they are ~12 lines in `_kit.splash`
(`sqrt` by Newton's method). Same maths, same colours, same picture, and it
runs on ArkUI too, which has no fragment-shader path at all.

The compiled-MPSL variants (`FlutterShader`/`FlutterSdf` in `splash-widgets`)
are still the right answer on makepad and are still built. They compile and
the node is emitted, but nothing draws; the suspect is the Splash isolate not
resolving a widget this crate adds to the prelude. Unconfirmed, and no longer
blocking anything.

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
