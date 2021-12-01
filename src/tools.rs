use super::{Env, Read, StringPtr};

// if get_string throws an exception abort for some reason is being called
pub fn abort(env: &Env, message: StringPtr, filename: StringPtr, line: i32, col: i32) {
    let memory = env.memory.get_ref().expect("initialized memory");
    let message = message.read(memory).unwrap();
    let filename = filename.read(memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
