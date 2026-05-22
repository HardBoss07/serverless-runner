use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Write exactly 3000 bytes. The DB snippet should strictly be <= 2048 characters.
    let payload = vec![b'B'; 3000];
    io::stdout().write_all(&payload)?;
    Ok(())
}
