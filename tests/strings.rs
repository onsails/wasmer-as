#[macro_use]
extern crate wasmer_runtime;
extern crate wasmer_as;

use std::error::Error;
use wasmer_as::AsmScriptString;
use wasmer_runtime::{imports, instantiate, Array, Ctx, Func, WasmPtr};

#[test]
fn read_strings() -> Result<(), Box<dyn Error>> {
    let wasm = include_bytes!("../get-string.wasm");

    let import_object = imports! {
        "env" => {
            "abort" => func!(abort),
        },
    };
    let instance = instantiate(&wasm[..], &import_object).expect("Unable to instantiate");
    let get_string: Func<(), WasmPtr<u16, Array>> = instance.func("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.get_as_string(instance.context().memory(0))?;

    assert_eq!(string, "TheString");

    Ok(())
}

#[allow(dead_code)]
fn abort(_ctx: &mut Ctx, _message: i32, _filename: i32, _line: i32, _col: i32) {
    eprintln!("abort called");
}
