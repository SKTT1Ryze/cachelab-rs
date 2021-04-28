//! 华中科技大学计算机学院体系结构 Cache 实验 Rust 语言实现
//! 

mod cli;
mod trace;
mod address;
mod cache;

use std::io::prelude::*;
use std::fs::File;
use std::env;
use cli::Cli;
use trace::Trace;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let args_input: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let cli = Cli::parse(args_input);
    println!("cli: {:?}", cli);
    let trace = Trace::from_path(cli.tracefile)?;
    println!("trace: {:?}", trace);
    let mut f = File::create(".csim_results")?;
    f.write(b"0 0 0\n")?;
    println!("hits:{} misses:{} evictions:{}", 0, 0, 0);
    Ok(())
}


