use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Create a 2MB payload of the letter 'A'
    let payload = vec![b'A'; 2 * 1024 * 1024];
    
    // Attempting to write this will either truncate at 1MB or return an IO error 
    // depending on how wasmtime-wasi pipe handles the overflow.
    let _ = io::stdout().write_all(&payload);
    
    Ok(())
}