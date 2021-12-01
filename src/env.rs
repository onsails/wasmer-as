use wasmer::{Function, HostEnvInitError, Instance, LazyInit, Memory, WasmerEnv};

#[derive(Clone, Default)]
pub struct Env {
    pub memory: LazyInit<Memory>,
    pub new: Option<Function>,
}

impl Env {
    pub fn new(memory: Memory, new: Option<Function>) -> Env {
        let mut env = Env::default();
        env.memory.initialize(memory);
        env.new = new;
        env
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
            self.new = Some(func.clone())
        }
        self.memory.initialize(mem);
        Ok(())
    }
}
