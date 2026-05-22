use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    match TcpStream::connect("8.8.8.8:80") {
        Ok(_) => {
            io::stdout().write_all(b"DANGER: Network sandbox breached!\n")?;
        }
        Err(e) => {
            // WASI blocks socket creation
            io::stdout().write_all(format!("{}", e).as_bytes())?;
        }
    }
    Ok(())
}
