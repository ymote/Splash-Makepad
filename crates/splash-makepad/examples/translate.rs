//! Evaluate a Splash card and print the makepad UI dialect this backend emits.
//!
//! Same shared VM + `UiNode` that Splash-OH renders to ArkUI — here rendered to
//! makepad's `View{…}/Label{…}` script instead. Run with:
//!   cargo run -p splash-makepad --example translate

const SAMPLE: &str = r#"
fn argb(a, r, g, b) { return ((a * 256 + r) * 256 + g) * 256 + b }
let bg    = argb(255, 18, 18, 18)
let white = argb(255, 255, 255, 255)
let red   = argb(255, 255, 40, 40)

{t: "column", w: 402, h: 220, bg: bg, pad: 16, c: [
    {t: "row", w: 370, h: 32, c: [
        {t: "column", w: 30, h: 22, bg: red, radius: 6},
        {t: "text", text: "Splash → makepad", size: 20, weight: 8, color: white, w: 300, h: 28},
    ]},
    {t: "text", text: "one VM, one node model, two backends", size: 14, color: white, w: 360, h: 22},
]}
"#;

fn main() {
    // Optional .splash file arg; otherwise the embedded sample.
    let src = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(&path).expect("read .splash file"),
        None => SAMPLE.to_string(),
    };
    let tree = splash_render::build(&src, |_vm| {}).expect("evaluates");
    eprintln!("// UiNode tree: {} nodes\n", tree.count());
    print!("{}", splash_makepad::to_makepad_ui(&tree));
}
