//! traces 文件解析

use std::{io::Read, path::Path};
use std::io;
use std::fs::File;
use std::default::Default;

#[derive(Debug)]
pub enum Operation {
    InstructionLoad,
    DataLoad,
    DataStore,
    DataModify
}

#[derive(Debug)]
pub struct TraceEntry {
    pub operation: Operation,
    pub address: usize,
    pub size: usize
}

impl Default for TraceEntry {
    fn default() -> Self {
        Self {
            operation: Operation::InstructionLoad,
            address: 0,
            size: 0
        }
    }
}

#[derive(Debug)]
pub struct Traces {
    inner: Vec<TraceEntry>
}

impl Traces {
    pub fn empty() -> Self {
        Self {
            inner: Vec::new()
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref();
        let f = File::open(p)?;
        let mut reader = io::BufReader::new(f);
        let mut source = String::new();
        reader.read_to_string(&mut source)?;
        let lines: Vec<&str> = source.split('\n').filter(|x| x.len() > 0).collect();
        let mut trace = Traces::empty();
        for line in lines {
            let mut trace_entry = TraceEntry::default();
            let s = String::from(line);
            let s_v: Vec<&str> = s.split(' ').filter(|x| x.len() > 0).collect();
            assert!(s_v.len() == 2);
            trace_entry.operation = match s_v[0] {
                "I" => Operation::InstructionLoad,
                "L" => Operation::DataLoad,
                "S" => Operation::DataStore,
                "M" => Operation::DataModify,
                _ => panic!("Invalid operaion in trace file!")
            };
            let t: Vec<&str> = s_v[1].split(',').collect();
            assert!(t.len() == 2);
            let t: Vec<usize> = t.iter().map(|x| {
                usize::from_str_radix(x, 16).unwrap()
            }).collect();
            trace_entry.address = t[0];
            trace_entry.size  = t[1];
            trace.inner.push(trace_entry);
        }
        Ok(trace)
    }
}

impl std::iter::IntoIterator for Traces {
    type Item = TraceEntry;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}