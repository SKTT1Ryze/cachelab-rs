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
use trace::Traces;
use cache::Cache;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let args_input: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let cli = Cli::parse(args_input);
    // println!("cli: {:?}", cli);
    
    let traces = Traces::from_path(cli.tracefile)?;
    // println!("traces: {:?}", trace);
    
    let mut cache: Cache<usize> = unsafe { Cache::init(cli.s as usize, cli.e as usize, cli.b as usize) };

    let (hits, misses, evicts) = cache.run_traces(traces);
    let mut f = File::create(".csim_results")?;
    f.write(format!("{} {} {}\n", hits, misses, evicts).as_bytes())?;
    Ok(())
}


