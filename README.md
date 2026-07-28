# Splash-Makepad

Themed, cross-platform **component kits** for apps built on the **Splash DSL → makepad native-widget** renderer.

Author a UI once as plain-data Splash DSL; it is evaluated in the makepad-script VM, translated to makepad's own widget dialect, and mounted as **real native makepad widgets** at runtime (with on-device hot reload). This repo is the home for the render pipeline **and** the themed component sets that ride on it — Material 3 today; **iOS** and **liquid-glass** planned.

## The pipeline

```
Splash DSL  ──►  splash-render  ──►  UiNode tree  ──►  splash-makepad  ──►  makepad dialect string
{t:"column",       (makepad-script VM,   (backend-       (pure translation)     View{…}/Label{…}/…
 c:[ … ]}           renderer-free)        agnostic)                              │
                                                                                 ▼
                                                          makepad `Splash` widget .set_text() → live native widgets
```

- **`crates/splash-render`** — backend-agnostic core: evaluates the Splash DSL in the makepad-script VM and walks it into a `UiNode` tree. Depends only on `makepad-script`. Unit-tested.
- **`crates/splash-makepad`** — the makepad backend: `to_makepad_ui(&UiNode) -> String` turns the tree into makepad's `View{}/Label{}/…` dialect. Pure, unit-tested — no makepad-platform/draw needed to build or test.
- **`crates/splash-widgets`** — the **themed native-widget kits** (Material 3 now; iOS / liquid-glass later), as **external `script_mod!` variants of makepad's widgets** (see *Fork-free theming* below).
- **`components/<theme>/`** — each theme's **component library**, authored as `.splash` (e.g. `components/material/catalog.splash`, ~35 Material components + demo screens). Pure data — hot-reloadable, no rebuild.

## Fork-free theming (the key design point)

makepad's native controls — checkbox, switch, radio, slider, text field — are drawn by their own MPSL shaders; their look is **not** reachable from the Splash DSL. It **is** reachable from an external crate, but only one way works:

| Mechanism | Result |
|---|---|
| Runtime `script_eval!` override of `mod.prelude.widgets.*` | shader **dropped** → widget renders blank ❌ |
| **Compiled `script_mod!`** — extend the base (`mod.widgets.CheckBox = mod.widgets.CheckBoxFlat{ draw_bg +: {…} }`), then reference into the prelude | shader **kept** ✅ |

The `script_mod!` macro compiles the MPSL at build time; a runtime string never gets compiled. So `splash-widgets` restyles makepad's widgets against **upstream makepad** with **no fork** — theming itself needs no upstream change. (Verified on device: a compiled variant renders; the runtime override renders blank.)

Each new theme is just more variants in `splash-widgets` + a `.splash` component library — no makepad fork per theme.

### The one upstream PR

Building + running the `kit-host` against upstream surfaced exactly **one** thing upstream doesn't have: a **`Splash` main-VM-mount option**. Upstream's `Splash` always allocates an *isolate* VM (`alloc_splash_vm_with_network(allow_net)`), but the light theme and a shared heap live on the app's **main** VM. The fix is the small `isolate: false` field this project's fork added to `widgets/src/splash.rs` — upstreaming it lets a trusted, app-generated kit mount on the main VM (correct theme, no isolate-heap animator panics). Until then the kit mounts on an isolate (dark-default theme). That is the *only* upstream change needed; everything else runs against upstream `dev` as-is.

## Relationship to makepad

Upstream `makepad/makepad` (branch `dev`) already ships everything this needs — the `makepad-script` VM (`platform/script`) and the `Splash` runtime-mount widget. Nothing here requires a makepad fork:

- The **core crates** depend on `makepad-script` by git.
- **`splash-widgets`** and the kit apps depend on upstream **`makepad-widgets`** by git. (Keep the transitive `makepad-script` rev aligned with `splash-render`'s.)

## Build

```sh
cargo test            # builds + tests the portable core (splash-render, splash-makepad)
```

`splash-widgets` is excluded from the default workspace build (it needs the full upstream-makepad build); wire it into an app's makepad workspace to use it — call `splash_widgets::widgets_mod(vm)` in place of `makepad_widgets::widgets_mod(vm)`.

## Status

- ✅ `splash-render` + `splash-makepad` — portable render pipeline; **compile + test against upstream `makepad-script`** (rev `e1c2164b`), no fork
- ✅ **Material 3 kit** — `components/material/catalog.splash`: ~35 components (buttons, FABs, cards, chips, nav bar/rail/drawer, app bars, dialog/menu/sheets as **real interactive overlays**, pickers, tabs, badges, toolbars), M3 tokens (colour, type scale + Medium weight, shape, elevation, surface tones), Font-Awesome monochrome icons, and real animation (circular spinner + shape-morph loading indicator)
- ✅ `splash-widgets` — Material 3 native-control variants (checkbox/switch/radio/slider/text field) + `LoadingMorph`, fork-free; **compiles against upstream `makepad-widgets`**
- ✅ **`apps/kit-host`** — generic app shell that **builds + runs against upstream makepad** (desktop, ~37 MB binary), fork-free, mounting the Material kit via `splash_widgets::widgets_mod`
- ⏳ **The one upstream PR:** the `Splash` main-VM-mount option (see above) — the single change needed for correct light-theme rendering
- ⏳ **Next:** that PR (or an isolate-VM theme/heap fix so the mount works isolated); Android build via `cargo-makepad`; Button **touch-ripple** as a `RippleButton` variant; **iOS** + **liquid-glass** kits

## License

MIT OR Apache-2.0 (matching makepad).
