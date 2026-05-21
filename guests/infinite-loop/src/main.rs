fn main() {
    // Prevent the compiler from optimizing the loop away
    let mut count: u64 = 0;
    loop {
        count = count.wrapping_add(1);
        std::hint::black_box(count);
    }
}