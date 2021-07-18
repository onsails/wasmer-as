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

#[test]
fn read_strings() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-wasm/build/optimized.wasm"
    ));
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
    let memory = instance.exports.get_memory("memory").expect("get memory");

    let get_string = instance
        .exports
        .get_native_function::<(), AsmScriptStringPtr>("getString")?;

    let str_ptr = get_string.call()?;
    let string = str_ptr.read(memory)?;

    assert_eq!(string, "TheString ©");

    Ok(())
}

#[allow(dead_code)]
fn abort(
    env: &Env,
    message: AsmScriptStringPtr,
    filename: AsmScriptStringPtr,
    line: i32,
    col: i32,
) {
    let memory = env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
