Helpers for dealing with assemblyscript memory inside wasmer-runtime
===

```rust
use std::error::Error;
use wasmer::{Instance, Memory, MemoryType, Module, Store};
use wasmer_as::{AsmScriptRead, AsmScriptStringPtr, Env, abort};

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-wasm/build/optimized.wasm"
    ));
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let memory = Memory::new(&store, MemoryType::new(1, None, false)).unwrap();
    let import_object = imports! {
        "env" => {
            "abort" => Function::new_native_with_env(&store, memory, abort),
        },
    };
    let instance = Instance::new(&module, &import_object)?;

    // for the test we use simple function returning constant string:
    //
    // export function getString(): string {
    //   return "TheString";
    // }
    let get_string = instance.exports.get_function("getString")?;

    let results = get_string.call(&[])?;

    let str_ptr = results.first().expect("get pointer");
    let str_ptr = AsmScriptStringPtr::new(str_ptr.unwrap_i32() as u32);

    let memory = instance.exports.get_memory("memory").expect("get memory");
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "$¢ह한𝌆");

    Ok(())
}
```
