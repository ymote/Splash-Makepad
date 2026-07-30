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

## Controls that are controls

The review's middle column — eighteen screens "drawn only" — was not eighteen
problems. It was one. A `.splash` screen is evaluated to a tree once per mount,
nothing survives that, and the only thing a tap could do was change the route.
So no checkbox could stay checked: there was nowhere to put "checked".

There is now. `state.rs` in both backends holds a key→value store, the DSL reads
it with `sget(key, default)`, and a tap names an action instead of a route:

```text
{t: "row", tapto: "set:m3_c1=!",        c: [...]}   // toggle
{t: "row", tapto: "set:ca_guests=+1",   c: [...]}   // step
{t: "row", tapto: "set:dps_query=~4",   c: [...]}   // cycle
{t: "row", tapto: "set:m3_radio=2",     c: [...]}   // pick, for radio groups
```

Actions ride the same interning as routes, so a control needs no new node
attribute and both backends get it from the one place a tap already lands.

All eighteen are wired against it, and
`a_control_changes_what_the_screen_renders` holds twenty-eight cases to it:
render, apply the action the control names, render again, and the two must
differ. That is the test the catalog never had, and it is why eighteen screens
could sit in the index marked as ports while none of their controls worked.

Eight had controls already drawn and inert — Material 3, Cupertino Gallery, Form
App, Date Planner, Compass, Platform Design, Photo Search, Testing. The other
ten had none at all, so each got the interaction its sample actually has:

| screen | what it does now |
|---|---|
| `dynamic_theme` | the transcript arrives a turn at a time, and `change_text_scale_factor` scales this screen's own type — two of the sample's three declared functions are real |
| `google_maps` | the camera steps through four zooms, each a different request to OSM |
| `simple_sdf` / `simple_shader` | resample the field at 4 resolutions — the point of both screens is that it is arithmetic, not a picture |
| `web_embedding` | the slot navigates between three pages (ArkUI only; there is no slot on makepad) |
| `platform_channels` | call again, and narrow to one channel. `device.notifications` was registered and reached by nothing — it is on the list now |
| `pedometer` | re-read the sensor, with the read count so a fresh answer is distinguishable |
| `background_isolate_channels` | re-run the off-main call |
| `asset_transformation` | step through the three requests one at a time |
| `platform_view_swift` | the sample's own interaction: switch halves and pass a counter across |

Three defects fell out of writing it, each invisible before:

- `sget` had to **seed** the store on first read, not just default. `apply`
  toggles against the current value and cannot know a screen considers a
  control on by default, so `sget("m3_switch", 1)` toggled from an assumed 0 to
  1 and rendered identically. The tap worked; the screen could not show it.
- `a or b` is not an operator in this VM. It parsed, and Testing App counted one
  favourite instead of three.
- `txt()` sizes its box from `s.len()`, so a number has no length and the node
  vanished entirely — the Compass stepper drew its two buttons with nothing
  between them.

Scroll position survives a tap. A rebuild replaces the Scroll node, so the view
used to snap to the top — checking a checkbox halfway down a screen left you at
the top of it. The offset is read off the old node before it is dropped and
written back onto the new one, but only when the route is unchanged: a tap that
ticks a checkbox should leave you looking at the checkbox, and a tap that
navigates should start the new screen at the top. That needed the shim's only
getter, `splash_get_f32`.

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

## Running the kit on HarmonyOS (attempted, not working)

`cargo makepad ohos` exists and the kit can be got as far as a signed HAP that
installs and runs on a HarmonyOS phone (verified on a Mate 70 Air). It renders
nothing but its background. This records how far it gets and what is in the way,
so the next attempt does not rediscover it.

**Working, verified from the device log:** XComponent callbacks registered, EGL
context and window surface created, vsync registered, main loop entered, surface
1320x2523 at density 3.25. `Event::Startup` fires, the kit evaluates
(`built=true nodes=236 ui_len=71753`), and the `Splash` widget accepts the
dialect and produces a view without error.

**Not working:** no glyphs. `Cx::get_dependency` looks in a dependency map and,
on a miss, falls back to a platform asset read. That map is never populated on
any platform — Android satisfies every lookup through its
`to_java_load_asset` fallback — and OpenHarmony has no such fallback, so every
font request fails silently. With no font bytes, text measures zero, every `Fit`
container collapses, and the window shows only whatever `Fill` background sits
behind it. That single cause explains the blank screen, and it is why a plain
`Label` placed in the host chrome is invisible too.

**Nine build breaks had to be fixed before it would compile or package at all.**
Four are the same mistake: OpenHarmony reports `target_os = "linux"`, so
desktop-Linux paths get selected for it.

| break | cause |
|---|---|
| `linux_video_playback` unresolved | call site gated `linux, not(android)`; module gated `not(ohos)` |
| `no field opengl_cx` (x2) | `create_gl_render_bridge` compiled for OHOS, whose `CxOs` has no EGL context of its own |
| `-lxkbcommon` not found | `build.rs` links it for OS `linux`; not in the OHOS sysroot |
| `-lssl` / `-lcrypto` not found | the Linux network backend links OpenSSL by name. OHOS ships `libnet_ssl.so` / `libohcrypto.so`, which export none of the OpenSSL symbols — not a rename |
| no hilog writer | `log_with_level` dispatches through a function pointer; nothing installed one for OHOS |
| `Cx::init_log()` never called | desktop, Android and wasm entry points all call it; the OHOS napi entry did not |
| `signingConfigs: []` | the generated DevEco project is unsigned |
| bundle name mismatch | a provisioning profile is bound to one bundle name |
| `PackageHap` needs a JRE | DevEco ships one at `Contents/jbr` |

Until logging worked the platform was completely silent, which made every
failure above indistinguishable from the next.

**Also missing:** `WindowGeomChange` never fires on OHOS, so `st.vw`/`st.vh` stay
0 and `page()` falls back to `Fit`; and the app sandbox blocks
`/data/local/tmp`, so the host's hot-reload and route-override files are
unreachable there.

None of these fixes are in this repo — they were made in a local makepad fork and
are not committed. Treat OpenHarmony support as its own piece of work rather than
a step in porting a sample.
