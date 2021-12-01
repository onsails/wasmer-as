use std::convert::TryFrom;
use wasmer::{WasmPtr, Array, Value};
use super::{Read, Memory, Env, Write};

pub type StringPtr = WasmPtr<u16, Array>;

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
        let new = env.new.as_ref().expect("Assembly Script Runtime ot exported");
        let size = i32::try_from(value.len()).expect("Cannot convert value size t i32");

        let ptr = new.call(&[Value::I32(size << 1), Value::I32(1)]).expect("Failed to call __new").get(0).unwrap().i32().unwrap();
        let utf16 = value.encode_utf16();
        let view = env.memory.get_ref().expect("Failed to load memory").view::<u16>();

        let from = usize::try_from(ptr)? / 2;
        for (bytes, cell) in utf16.into_iter().zip(view[from..from + (size as usize)].iter()) {
            cell.set(bytes);
        }
        Ok(Box::new(StringPtr::new(ptr as u32)))
    }

    fn write(value: &str, memory: &Env) -> anyhow::Result<Box<Self>> {
        todo!()
    }

    fn free() -> anyhow::Result<()> {
        todo!()
    }
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