Helpers for dealing with assemblyscript memory inside wasmer-runtime
===

```rust
#[macro_use]
extern crate wasmer_runtime;

use std::error::Error;
use wasmer_runtime::{imports, instantiate, Array, Ctx, Func, WasmPtr};
use wasmer_as::AsmScriptString;

fn main() -> Result<(), Box<dyn Error>> {
    let mut wasm: Vec<u8> = include_bytes!("get-string.wasm");

    let import_object = imports! {
        "env" => {
            "abort" => func!(abort),
        },
    };

    let instance = instantiate(&wasm[..], &import_object)?;

    // for the test we use simple function returning constant string:
    //
    // export function getString(): string {
    //   return "TheString";
    // }
    let get_string: Func<(), WasmPtr<u16, Array>> = instance.func("getString")?;
    
    let str_ptr = get_string.call()?;
    
    let string = str_ptr.get_as_string(instance.context().memory(0))?;

    assert_eq!(string, "TheString");

    Ok(())
}

// if get_string throws an exception abort for some reason is being called
fn abort(ctx: &mut Ctx, message: WasmPtr<u16, Array>, filename: WasmPtr<u16, Array>, line: i32, col: i32) {
    let memory = ctx.memory(0);
    let message = message.get_as_string(memory).unwrap();
    let filename = filename.get_as_string(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
```
