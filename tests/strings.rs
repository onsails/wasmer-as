#[macro_use]
extern crate wasmer_runtime;
extern crate wasmer_as;

use std::error::Error;
use wasmer_as::{AsmScriptRead, AsmScriptStringPtr};
use wasmer_runtime::{imports, instantiate, Ctx, Func};

#[test]
fn read_strings() -> Result<(), Box<dyn Error>> {
    let wasm = include_bytes!("../get-string.wasm");

    let import_object = imports! {
        "env" => {
            "abort" => func!(abort),
        },
    };
    let instance = instantiate(&wasm[..], &import_object).expect("Unable to instantiate");
    let get_string: Func<(), AsmScriptStringPtr> = instance.func("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.read(instance.context().memory(0))?;

    assert_eq!(string, "TheString");

    Ok(())
}

#[allow(dead_code)]
fn abort(
    _ctx: &mut Ctx,
    _message: AsmScriptStringPtr,
    _filename: AsmScriptStringPtr,
    _line: i32,
    _col: i32,
) {
    eprintln!("abort called");
}
