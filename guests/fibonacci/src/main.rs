use std::io::{self, Read, Write};

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let n: u32 = buffer.trim().parse().unwrap_or(0);
    let result = fibonacci(n);

    io::stdout().write_all(format!("{}", result).as_bytes())?;
    Ok(())
}
