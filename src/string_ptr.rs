use super::{Env, Memory, Read, Write};
use std::convert::{TryFrom, TryInto};
use wasmer::{Array, Value, WasmPtr};

pub type StringPtr = WasmPtr<u16, Array>;

macro_rules! export_asr {
    ($func_name:ident, $env:expr) => {
        $env.$func_name
            .as_ref()
            .expect("Assembly Script Runtime not exported")
    };
}

impl Read<String> for StringPtr {
    fn read(self, memory: &Memory) -> anyhow::Result<String> {
        let size = self.size(memory)?;
        // we need size / 2 because assemblyscript counts bytes
        // while deref considers u16 elements
        if let Some(buf) = self.deref(memory, 0, size / 2) {
            let input: Vec<u16> = buf.iter().map(|b| b.get()).collect();
            Ok(String::from_utf16_lossy(&input))
        } else {
            anyhow::bail!("Wrong offset: can't read buf")
        }
    }

    fn size(self, memory: &Memory) -> anyhow::Result<u32> {
        size(self.offset(), memory)
    }
}

impl Write<String> for StringPtr {
    fn alloc(value: &str, env: &Env) -> anyhow::Result<Box<StringPtr>> {
        let new = export_asr!(fn_new, env);
        let size = i32::try_from(value.len())?;

        let offset = u32::try_from(
            new.call(&[Value::I32(size << 1), Value::I32(1)])
                .expect("Failed to call __new")
                .get(0)
                .unwrap()
                .i32()
                .unwrap(),
        )?;
        write_str(offset, value, env)?;

        // pin
        let pin = export_asr!(fn_pin, env);
        pin.call(&[Value::I32(offset.try_into().unwrap())])
            .expect("Failed to call __pin");

        Ok(Box::new(StringPtr::new(offset)))
    }

    fn write(&self, value: &str, env: &Env) -> anyhow::Result<()> {
        let prev_size = size(
            self.offset(),
            env.memory.get_ref().expect("Failed to load memory"),
        )?;
        let new_size = u32::try_from(value.len())? << 1;
        if prev_size == new_size {
            write_str(self.offset(), value, env)?
        } else {
            todo!("Remove this and reallocate of bigger or smaller space")
        }
        Ok(())
    }

    fn free(_env: &Env) -> anyhow::Result<()> {
        todo!("Release the memory from this string")
    }
}

fn write_str(offset: u32, value: &str, env: &Env) -> anyhow::Result<()> {
    let utf16 = value.encode_utf16();
    let view = env
        .memory
        .get_ref()
        .expect("Failed to load memory")
        .view::<u16>();
    // We count in 32 so we have to devide by 2
    let from = usize::try_from(offset)? / 2;
    for (bytes, cell) in utf16.into_iter().zip(view[from..from + value.len()].iter()) {
        cell.set(bytes);
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
        Ok(cell.get())
    } else {
        anyhow::bail!("Wrong offset: can't read size")
    }
}
