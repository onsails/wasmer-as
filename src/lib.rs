mod env;
mod tools;
mod string_ptr;

pub use string_ptr::StringPtr;
pub use env::Env;
pub use tools::abort;

use std::fmt;
use wasmer::{Memory};

pub trait Read<T> {
    fn read(self, memory: &Memory) -> Result<T, Error>;
    fn size(self, memory: &Memory) -> Result<u32, Error>;
    fn malloc(value: &str, memory: &Memory) -> Result<Box<Self>, Error>;
}

#[derive(Debug)]
pub enum Error {
    Mem(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Mem(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
