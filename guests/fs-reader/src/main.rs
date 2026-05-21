use std::fs;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    match fs::read_to_string("/etc/passwd") {
        Ok(content) => {
            io::stdout().write_all(b"DANGER: Sandbox breached!\n")?;
            io::stdout().write_all(content.as_bytes())?;
        }
        Err(e) => {
            // WASI will block this and return a Permission Denied or No Such File error
            io::stdout().write_all(format!("{}", e).as_bytes())?;
        }
    }
    Ok(())
}