Helpers for dealing with assemblyscript memory inside wasmer-runtime
===

```rust
fn main() -> Result<(), Error> {
    let import_object = imports! {
        "env" => {
            "abort" => func!(abort),
        },
    };

    let mut instance = instantiate(&wasm[..], &import_object)?;

    let add_one: Func<(i32, i32), i32> = instance.func("add")?;

    let value = add_one.call(42, 2)?;

    assert_eq!(value, 44);

    Ok(())
}

fn abort(ctx: &mut Ctx, message: i32, filename: i32, line: i32, col: i32) {
    let memory = ctx.memory(0);
    let message = ASReader::read_string(message, memory).unwrap();
    let filename = ASReader::read_string(filename, memory).unwrap();
    eprintln!("Error: {} at {}:{} col: {}", message, filename, line, col);
}
```