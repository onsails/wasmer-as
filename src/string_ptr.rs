use wasmer::{WasmPtr, Array};
use super::{Read, Memory, Error};

pub type StringPtr = WasmPtr<u16, Array>;

impl Read<String> for StringPtr {
    fn read(self, memory: &Memory) -> Result<String, Error> {
        let size = self.size(memory)?;
        // we need size / 2 because assemblyscript counts bytes
        // while deref considers u16 elements
        if let Some(buf) = self.deref(memory, 0, size / 2) {
            let input: Vec<u16> = buf.iter().map(|b| b.get()).collect();
            Ok(String::from_utf16_lossy(&input))
        } else {
            Err(Error::Mem("Wrong offset: can't read buf"))
        }
    }

    fn size(self, memory: &Memory) -> Result<u32, Error> {
        if self.offset() < 4 {
            return Err(Error::Mem("Wrong offset: less than 2"));
        }
        // read -4 offset
        // https://www.assemblyscript.org/memory.html#internals
        if let Some(cell) = memory.view::<u32>().get(self.offset() as usize / (32 / 8) - 1) {
            Ok(cell.get())
        } else {
            Err(Error::Mem("Wrong offset: can't read size"))
        }
    }

    fn malloc(_value: &str, _memory: &Memory) -> Result<Box<StringPtr>, Error> {
        todo!();
    }
}