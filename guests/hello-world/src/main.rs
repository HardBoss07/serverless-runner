use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let name = if buffer.trim().is_empty() {
        "Guest"
    } else {
        buffer.trim()
    };
    let response = format!("Hello, {}! (Rendered by Wasmtime)\n", name);

    io::stdout().write_all(response.as_bytes())?;
    Ok(())
}
