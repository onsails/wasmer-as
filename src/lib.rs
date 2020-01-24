use std::fmt;
use wasmer_runtime::{Array, Memory, WasmPtr};

pub struct ASReader;

impl ASReader {
    pub fn size(offset: usize, memory: &Memory) -> Result<u32, Error> {
        if let Some(cell) = memory.view::<u32>().get(offset / 2 - 1) {
            Ok(cell.get())
        } else {
            Err(Error::Mem("Wrong offset: can't read size"))
        }
    }

    pub fn read_string(ptr: i32, memory: &Memory) -> Result<String, Error> {
        let ptr = (ptr >> 1) as u32;
        let size = Self::size(ptr as usize, memory)?;
        let ptr: WasmPtr<u16, Array> = WasmPtr::new(ptr * 2);
        if let Some(buf) = ptr.deref(memory, 0, size / 2) {
            let input: Vec<u16> = buf.iter().map(|b| b.get()).collect();
            let mut output: Vec<u8> = vec![0; size as usize * 3];
            let len = ucs2::decode(&input, &mut output)?;
            // should not loss because ucs2::decode should guarantee correct utf-8
            Ok(String::from_utf8_lossy(output[..len].into()).into_owned())
        } else {
            Err(Error::Mem("Wrong offset: can't read buf"))
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Ucs2(ucs2::Error),
    Mem(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Ucs2(err) => write!(f, "{:?}", err),
            Error::Mem(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<ucs2::Error> for Error {
    fn from(err: ucs2::Error) -> Self {
        Self::Ucs2(err)
    }
}

#[cfg(test)]
#[macro_use]
extern crate wasmer_runtime;

#[cfg(test)]
mod tests {
    use super::ASReader;
    use std::fs::File;
    use std::io::prelude::*;
    use wasmer_runtime::{imports, instantiate, Ctx, Func};

    #[test]
    fn read_strings() {
        let mut file = File::open("get-string.wasm").expect("Failed to open wasm file");
        let mut wasm: Vec<u8> = vec![];
        file.read_to_end(&mut wasm).expect("Unnable to read wasm");

        let import_object = imports! {
            "env" => {
                "abort" => func!(abort),
            },
        };
        let instance = instantiate(&wasm[..], &import_object).expect("Unable to instantiate");
        let get_string: Func<(), i32> = instance.func("getString").expect("Unable to export func");
        let str_ptr = get_string.call().expect("Call failed");
        let string = ASReader::read_string(str_ptr, instance.context().memory(0))
            .expect("Unable to read string");
        assert_eq!(string, "TheString");
    }

    #[allow(dead_code)]
    fn abort(_ctx: &mut Ctx, _message: i32, _filename: i32, _line: i32, _col: i32) {
        eprintln!("abort called");
    }
}
