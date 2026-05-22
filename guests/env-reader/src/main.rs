use std::env;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "Env var not found".to_string());
    io::stdout().write_all(db_url.as_bytes())?;
    Ok(())
}
