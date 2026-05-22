fn main() {
    let mut allocations: Vec<Vec<u8>> = Vec::new();
    loop {
        // Allocate 1MB at a time
        // The host engine will forcefully terminate the guest when it hits 64MB
        allocations.push(vec![0; 1024 * 1024]);
    }
}
