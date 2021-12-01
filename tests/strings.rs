use std::error::Error;
use wasmer::{imports, Function, Instance, Module, Store};
use wasmer_as::{abort, Env, Read, StringPtr, Write};

#[test]
fn read_strings() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_wat.wat"));
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

#[test]
fn read_alloc_strings() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/runtime_exported.wat"
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

    let env = Env::new(
        memory.clone(),
        match instance.exports.get_function("__new") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
    );

    let get_string = instance
        .exports
        .get_native_function::<(), StringPtr>("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "hello test");

    let str_ptr_2 = StringPtr::alloc("hello return", &env)?;
    let string = str_ptr_2.read(memory)?;
    assert_eq!(string, "hello return");

    Ok(())
}

#[test]
fn read_write_strings() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/runtime_exported.wat"
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

    let env = Env::new(
        memory.clone(),
        match instance.exports.get_function("__new") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
    );

    let get_string = instance
        .exports
        .get_native_function::<(), StringPtr>("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "hello test");

    str_ptr.write("hallo tast", &env)?;

    let str_ptr_2 = get_string.call()?;
    let string = str_ptr_2.read(memory)?;

    assert_eq!(string, "hallo tast");
    Ok(())
}
