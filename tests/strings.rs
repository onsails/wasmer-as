use std::error::Error;
use wasmer::{Store, Module, Instance, imports, Function};
use wasmer_as::{Read, StringPtr, Env, abort};

#[test]
fn read_strings() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/test_wat.wat"
    ));
    let store = Store::default();
    let module = Module::new(&store, wasm_bytes)?;
    let import_object = imports! {
        "env" => {
            "abort" => Function::new_native_with_env(&store, Env::default(), abort),
        },
    };

    let instance = Instance::new(&module, &import_object)?;
    let memory = instance.exports.get_memory("memory").expect("get memory");

    let get_string = instance
        .exports
        .get_native_function::<(), StringPtr>("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "$¢ह한𝌆");

    Ok(())
}
