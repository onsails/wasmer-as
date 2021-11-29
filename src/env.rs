use wasmer::{HostEnvInitError, Instance, LazyInit, Memory, WasmerEnv};

#[derive(Clone)]
pub struct Env {
    pub memory: LazyInit<Memory>,
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
