use super::{Env, Memory, Read, Write};
use std::convert::{TryFrom, TryInto};
use wasmer::{Array, Value, WasmPtr, FromToNativeWasmType};

use crate::tools::export_asr;

#[derive(Clone, Copy)]
pub struct BufferPtr(WasmPtr<u8, Array>);

impl BufferPtr {
    fn new(offset: u32) -> Self {
        Self(WasmPtr::new(offset))
    }
    pub fn offset(&self) -> u32 {
        self.0.offset()
    }
}

unsafe impl FromToNativeWasmType for BufferPtr {
    type Native = i32;
    fn to_native(self) -> Self::Native {
        self.offset() as i32
    }
    fn from_native(n: Self::Native) -> Self {
        Self::new(n as u32)
    }
}

impl Read<Vec<u8>> for BufferPtr {
    fn read(&self, memory: &Memory) -> anyhow::Result<Vec<u8>> {
        let size = self.size(memory)?;
        if let Some(buf) = self.0.deref(memory, 0, size * 2) {
            Ok(buf.iter().map(|b| b.get()).collect())
        } else {
            anyhow::bail!("Wrong offset: can't read buf")
        }
    }

    fn size(&self, memory: &Memory) -> anyhow::Result<u32> {
        size(self.0.offset(), memory)
    }
}

impl Write<Vec<u8>> for BufferPtr {
    fn alloc(value: &Vec<u8>, env: &Env) -> anyhow::Result<Box<BufferPtr>> {
        let new = export_asr!(fn_new, env);
        let size = i32::try_from(value.len())?;

        let offset = u32::try_from(
            new.call(&[Value::I32(size), Value::I32(0)])
                .expect("Failed to call __new")
                .get(0)
                .unwrap()
                .i32()
                .unwrap(),
        )?;
        write_buffer(offset, value, env)?;
        Ok(Box::new(BufferPtr::new(offset)))
    }

    fn write(&mut self, value: &Vec<u8>, env: &Env) -> anyhow::Result<Box<Self>> {
        let prev_size = size(
            self.offset(),
            env.memory.get_ref().expect("Failed to load memory"),
        )?;
        let new_size = u32::try_from(value.len())?;
        if prev_size == new_size {
            write_buffer(self.offset(), value, env)?;
            Ok(Box::new(*self))
        } else {
            // unpin old ptr
            let unpin = export_asr!(fn_pin, env);
            unpin
                .call(&[Value::I32(self.offset().try_into().unwrap())])
                .expect("Failed to call __unpin");

            // collect
            let collect = export_asr!(fn_collect, env);
            collect.call(&[]).expect("failed to call __collect");

            // alloc with new size
            BufferPtr::alloc(value, env)
        }
    }

    fn free(self, _env: &Env) -> anyhow::Result<()> {
        todo!("Release the memory from this string")
    }
}

fn write_buffer(offset: u32, value: &[u8], env: &Env) -> anyhow::Result<()> {
    let view = env
        .memory
        .get_ref()
        .expect("Failed to load memory")
        .view::<u8>();
    // We count in 32 so we have to devide by 2
    let from = usize::try_from(offset)? / 2;
    for (bytes, cell) in value.iter().zip(view[from..from + value.len()].iter()) {
        cell.set(*bytes);
    }
    Ok(())
}

fn size(offset: u32, memory: &Memory) -> anyhow::Result<u32> {
    if offset < 4 {
        anyhow::bail!("Wrong offset: less than 2")
    }
    // read -4 offset
    // https://www.assemblyscript.org/memory.html#internals
    if let Some(cell) = memory.view::<u32>().get(offset as usize / (32 / 8) - 1) {
        Ok(cell.get() / 2)
    } else {
        anyhow::bail!("Wrong offset: can't read size")
    }
}
