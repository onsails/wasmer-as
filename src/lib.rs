use wasmer_runtime::Memory;

pub struct ASReader;

impl ASReader {
    pub fn size(offset: usize, memory: &Memory) -> u32 {
        unsafe {
            let ptr = memory.view::<u16>().as_ptr().add(offset);
            let ptr = ptr as *const u32;
            let ptr = ptr.offset(-1);
            *ptr
        }
    }

    pub fn read_string(ptr: i32, memory: &Memory) -> Result<String, ucs2::Error> {
        unsafe {
            let offset = (ptr >> 1) as usize;
            let size = Self::size(offset, memory) as usize;
            let ptr = memory.view::<u16>().as_ptr().add(offset as usize) as *const u16;
            let len = size / std::mem::size_of::<u16>();
            let input: &[u16] = std::slice::from_raw_parts(ptr, len);
            let output = std::alloc::alloc(
                std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>()).unwrap(),
            );
            let output: &mut [u8] = std::slice::from_raw_parts_mut(output, size);
            let len = ucs2::decode(input, output)?;
            Ok(String::from_utf8_unchecked(output[..len].into()))
        }
    }
}
