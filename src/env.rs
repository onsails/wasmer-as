use wasmer::{Function, HostEnvInitError, Instance, LazyInit, Memory, WasmerEnv};

#[derive(Clone, Default)]
pub struct Env {
    pub memory: LazyInit<Memory>,
    pub fn_new: Option<Function>,
    pub fn_pin: Option<Function>,
    pub fn_unpin: Option<Function>,
    pub fn_collect: Option<Function>,
}

impl Env {
    pub fn new(
        arg_memory: Memory,
        fn_new: Option<Function>,
        fn_pin: Option<Function>,
        fn_unpin: Option<Function>,
        fn_collect: Option<Function>,
    ) -> Env {
        let mut memory = LazyInit::<Memory>::default();
        memory.initialize(arg_memory);
        Env {
            memory,
            fn_new,
            fn_pin,
            fn_unpin,
            fn_collect,
        }
    }
}

impl WasmerEnv for Env {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        let mem = instance
            .exports
            .get_memory("memory")
            .map_err(HostEnvInitError::from)?
            .clone();
        if let Ok(func) = instance.exports.get_function("__new") {
            self.fn_new = Some(func.clone())
        }
        if let Ok(func) = instance.exports.get_function("__pin") {
            self.fn_pin = Some(func.clone())
        }
        if let Ok(func) = instance.exports.get_function("__unpin") {
            self.fn_unpin = Some(func.clone())
        }
        if let Ok(func) = instance.exports.get_function("__collect") {
            self.fn_collect = Some(func.clone())
        }
        self.memory.initialize(mem);
        Ok(())
    }
}
