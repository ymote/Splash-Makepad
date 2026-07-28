//! Evaluate Splash source in the makepad-script VM and walk the result into a
//! [`UiNode`] tree. This module owns the **only** makepad-script dependency in
//! the render path — backends never touch the VM, they consume `UiNode`.

use crate::node::{Attrs, NodeKind, UiNode};
use makepad_script::apply::*;
use makepad_script::array::ScriptArrayStorage;
use makepad_script::makepad_live_id::*;
use makepad_script::traits::*;
use makepad_script::*;

/// Evaluate Splash `src` and walk it into a `UiNode` tree.
///
/// `register` runs against the fresh VM *before* evaluation so the host can
/// inject its capabilities (e.g. a `fetch` function) as globals. Pass a no-op
/// (`|_| {}`) if the script needs none. Returns `None` if the script evaluates
/// to nil (a parse or runtime error) or the root tag is unknown.
pub fn build(src: &str, register: impl FnOnce(&mut ScriptVm)) -> Option<UiNode> {
    let mut std_slot = 0;
    let mut host = 0;
    let vm = &mut ScriptVm {
        host: &mut host,
        std: &mut std_slot,
        bx: Box::new(ScriptVmBase::new()),
    };

    register(vm);

    let value = vm.eval(ScriptMod {
        cargo_manifest_path: String::new(),
        module_path: String::from("splash"),
        file: String::from("splash.splash"),
        line: 0,
        column: 0,
        code: src.to_string(),
        values: Vec::new(),
    });

    if value.is_nil() {
        return None;
    }
    walk(vm, value, 0)
}

/// One DSL object → one `UiNode`, recursing into `c`.
fn walk(vm: &mut ScriptVm, value: ScriptValue, depth: usize) -> Option<UiNode> {
    // A malformed script must not be able to blow the native stack.
    if depth > 32 {
        return None;
    }
    let tag = string_prop(vm, value, id!(t)).unwrap_or_default();
    let kind = NodeKind::from_tag(&tag)?;

    let attrs = Attrs {
        text: string_prop(vm, value, id!(text)),
        label: string_prop(vm, value, id!(label)),
        placeholder: string_prop(vm, value, id!(placeholder)),
        id: string_prop(vm, value, id!(id)),
        tapto: string_prop(vm, value, id!(tapto)),
        src: string_prop(vm, value, id!(src)),
        fit: int_prop(vm, value, id!(fit)),
        w: f32_prop(vm, value, id!(w)),
        h: f32_prop(vm, value, id!(h)),
        fitw: int_prop(vm, value, id!(fitw)),
        fith: int_prop(vm, value, id!(fith)),
        fillw: int_prop(vm, value, id!(fillw)),
        fillh: int_prop(vm, value, id!(fillh)),
        size: f32_prop(vm, value, id!(size)),
        weight: int_prop(vm, value, id!(weight)),
        icon: int_prop(vm, value, id!(icon)),
        color: u32_prop(vm, value, id!(color)),
        bg: u32_prop(vm, value, id!(bg)),
        radius: f32_prop(vm, value, id!(radius)),
        elevation: f32_prop(vm, value, id!(elevation)),
        pad: f32_prop(vm, value, id!(pad)),
        padx: f32_prop(vm, value, id!(padx)),
        pady: f32_prop(vm, value, id!(pady)),
        spacing: f32_prop(vm, value, id!(spacing)),
        margin: f32_prop(vm, value, id!(margin)),
        border: f32_prop(vm, value, id!(border)),
        bordercolor: u32_prop(vm, value, id!(bordercolor)),
        value: f32_prop(vm, value, id!(value)),
        total: f32_prop(vm, value, id!(total)),
        align: int_prop(vm, value, id!(align)),
        alignx: f32_prop(vm, value, id!(alignx)),
        aligny: f32_prop(vm, value, id!(aligny)),
        on: int_prop(vm, value, id!(on)),
        tap: int_prop(vm, value, id!(tap)),
    };

    // `c` is a ScriptArray, NOT an object with a vec — arrays are their own heap
    // type in this VM, so treating one as an object drops the whole subtree.
    let mut children = Vec::new();
    for kid in children_of(vm, value) {
        if let Some(child) = walk(vm, kid, depth + 1) {
            children.push(child);
        }
    }

    Some(UiNode {
        kind,
        attrs,
        children,
    })
}

// ---- VM value helpers ------------------------------------------------------
// Exposed `pub` so a host can read capability-call arguments with the same
// coercion the walk uses (see `add_global_fn`).

/// The value at `key` in `obj`, or `None`.
pub fn prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<ScriptValue> {
    vm.bx.heap.value_for_apply(obj, key.into(), &Apply::Eval)
}

/// A string property (owned copy), or `None`.
pub fn string_prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<String> {
    let v = prop(vm, obj, key)?;
    vm.string_with(v, |_vm, s| s.to_string())
}

/// A numeric property, via the VM's own coercion rather than bit-poking the
/// NaN-boxed representation, so ints, floats and colour literals all work.
pub fn num_prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<f64> {
    let v = prop(vm, obj, key)?;
    if v.is_nil() {
        return None;
    }
    let mut out: f64 = 0.0;
    <f64 as ScriptApply>::script_apply(&mut out, vm, &Apply::Eval, &mut Scope::default(), v);
    Some(out)
}

fn f32_prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<f32> {
    num_prop(vm, obj, key).map(|v| v as f32)
}
fn int_prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<i32> {
    num_prop(vm, obj, key).map(|v| v as i32)
}
fn u32_prop(vm: &mut ScriptVm, obj: ScriptValue, key: LiveId) -> Option<u32> {
    num_prop(vm, obj, key).map(|v| v as u32)
}

/// The `c` array's members, copied out so the walk can re-borrow the vm.
fn children_of(vm: &mut ScriptVm, value: ScriptValue) -> Vec<ScriptValue> {
    let Some(c) = prop(vm, value, id!(c)) else {
        return Vec::new();
    };
    let Some(arr) = c.as_array() else {
        return Vec::new();
    };
    match vm.bx.heap.array_storage(arr) {
        ScriptArrayStorage::ScriptValue(v) => v.iter().copied().collect(),
        _ => Vec::new(),
    }
}

/// Register a native function on the VM and return it as a value ready to inject
/// as a global (e.g. `vm.set_injected_global(id!(fetch), add_global_fn(vm, …))`).
/// The closure receives the call's argument object as a value; read it with
/// [`string_prop`] / [`num_prop`].
pub fn add_global_fn<F>(vm: &mut ScriptVm, args: &[(LiveId, ScriptValue)], f: F) -> ScriptValue
where
    F: Fn(&mut ScriptVm, ScriptValue) -> ScriptValue + 'static,
{
    let base = &mut *vm.bx;
    let mut native = base.code.native.borrow_mut();
    let obj = native.add_fn(&mut base.heap, args, move |vm, args| f(vm, args.into()));
    obj.into()
}
