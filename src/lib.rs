use std::fmt;
use wasmer_runtime::{Array, Memory, WasmPtr};

pub trait AsmScriptString {
    fn get_as_string(self, memory: &Memory) -> Result<String, Error>;

    fn size(offset: u32, memory: &Memory) -> Result<u32, Error> {
        if offset < 4 {
            return Err(Error::Mem("Wrong offset: les than 2"));
        }

        if let Some(cell) = memory.view::<u32>().get(offset as usize / 4 - 1) {
            Ok(cell.get())
        } else {
            Err(Error::Mem("Wrong offset: can't read size"))
        }
    }
}

impl AsmScriptString for WasmPtr<u16, Array> {
    fn get_as_string(self, memory: &Memory) -> Result<String, Error> {
        let offset = self.offset();
        let size = Self::size(offset, memory)?;
        if let Some(buf) = self.deref(memory, 0, size / 2) {
            let input: Vec<u16> = buf.iter().map(|b| b.get()).collect();
            let mut output: Vec<u8> = vec![0; size as usize * 3];
            let len = ucs2::decode(&input, &mut output)?;
            // should not loose because ucs2::decode should guarantee correct utf-8
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
