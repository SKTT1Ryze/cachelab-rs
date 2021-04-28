use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut f = File::create(".csim_results").unwrap();
    f.write(b"0 0 0\n").unwrap();
    println!("hits:{} misses:{} evictions:{}", 0, 0, 0);
}
