Helpers for dealing with assemblyscript memory inside wasmer-runtime
===

```rust
use std::error::Error;
use wasmer::*;
use wasmer_as::{AsmScriptRead, AsmScriptStringPtr};

#[derive(Clone)]
struct Env {
    memory: LazyInit<Memory>,
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.memory.initialize(
            instance
                .exports
                .get_memory("memory")
                .map_err(HostEnvInitError::from)?
                .clone(),
        );
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/get-string.wasm"));
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;

    let env = Env {
        memory: LazyInit::default(),
    };

    let import_object = imports! {
        "env" => {
            "abort" => Function::new_native_with_env(&store, env, abort),
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

    assert_eq!(string, "TheString");

    Ok(())
}

// if get_string throws an exception abort for some reason is being called
fn abort(
    env: &Env,
    message: AsmScriptStringPtr,
    filename: AsmScriptStringPtr,
    line: i32,
    col: i32
) {
    let memory = env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
```
