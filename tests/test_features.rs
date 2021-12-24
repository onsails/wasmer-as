use std::error::Error;
use wasmer::{imports, Function, Instance, Module, Store};
use wasmer_as::{abort, Env, Read, StringPtr, BufferPtr, Write};

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
        match instance.exports.get_function("__pin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__unpin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__collect") {
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

    let str_ptr_2 = StringPtr::alloc(&"hello return".to_string(), &env)?;
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
        match instance.exports.get_function("__pin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__unpin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__collect") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
    );

    let get_string = instance
        .exports
        .get_native_function::<(), StringPtr>("getString")?;

    let mut str_ptr = get_string.call()?;
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "hello test");

    str_ptr.write(&"hallo tast".to_string(), &env)?;

    let str_ptr_2 = get_string.call()?;
    let string = str_ptr_2.read(memory)?;

    assert_eq!(string, "hallo tast");
    Ok(())
}

#[test]
fn read_buffers() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/buffer.wasm"));
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
        .get_native_function::<(), BufferPtr>("get_buffer")?;

    let str_ptr = get_string.call()?;
    let vec = str_ptr.read(memory)?;
    let expected: Vec<u8> = vec![0x01, 0x03, 0x03, 0xFF];
    assert_eq!(vec, expected);
    Ok(())
}

#[test]
fn alloc_buffer() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/sort_buffer.wasm"
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
        match instance.exports.get_function("__pin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__unpin") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
        match instance.exports.get_function("__collect") {
            Ok(func) => Some(func.clone()),
            _ => None,
        },
    );

    let sort_buffer = instance
        .exports
        .get_native_function::<i32, ()>("sortBuffer")?;

    let input: Vec<u8> = vec![0x03, 0x02, 0x00, 0x01];
    let buffer_ptr = BufferPtr::alloc(&input, &env)?;
    sort_buffer.call(buffer_ptr.offset() as i32)?;
    let sorted = buffer_ptr.read(memory)?;

    let expected: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03];
    assert_eq!(sorted, expected);
    Ok(())
}
