use wasmer_runtime::Memory;

// TODO: impl Error
#[derive(Debug)]
pub enum Error {
    Ucs2(ucs2::Error),
    Mem(&'static str),
}

impl From<ucs2::Error> for Error {
    fn from(err: ucs2::Error) -> Self {
        Self::Ucs2(err)
    }
}

pub struct ASReader;

impl ASReader {
    pub fn size(offset: usize, memory: &Memory) -> Result<u32, Error> {
        if offset < 2 {
            return Err(Error::Mem("Offset is out of lower bound"));
        }

        unsafe {
            let ptr = memory.view::<u16>().as_ptr().add(offset - 2);
            let ptr = ptr as *const u32;
            Ok(*ptr)
        }
    }

    pub fn read_string(ptr: i32, memory: &Memory) -> Result<String, Error> {
        unsafe {
            let offset = (ptr >> 1) as usize;
            let size = Self::size(offset, memory)? as usize;
            if offset + size >= memory.size().bytes().0 {
                return Err(Error::Mem("Offset is out of upper bound"));
            }
            let ptr = memory.view::<u16>().as_ptr().add(offset as usize) as *const u16;
            let len = size / std::mem::size_of::<u16>();
            let input: &[u16] = std::slice::from_raw_parts(ptr, len);
            let output = std::alloc::alloc(
                std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap(),
            );
            let output: &mut [u8] = std::slice::from_raw_parts_mut(output, len * 3);
            let len = ucs2::decode(input, output)?;
            Ok(String::from_utf8_unchecked(output[..len].into()))
        }
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
