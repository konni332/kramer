mod bishop_mask;
mod rook_mask;

fn main() {
    println!("Hello, world!");
}

struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    table: Vec<u64>,
}
