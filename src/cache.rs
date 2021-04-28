//! Cache 实现

use std::{default::Default};
use super::address::Address;
use super::trace::*;

/// Cache 目录表项
#[derive(Clone)]
pub struct CacheEntry<Data: Default + Clone> {
    /// 有效位
    valid: bool,
    /// 标识位
    tag: usize,
    /// 数据
    inner: Data
}

pub struct Cache<Data: Default + Clone> {
    /// Cache 的组数, 每组包含的项数
    size: (usize, usize),
    /// 地址配置
    address: Address,
    /// (该组是否已经满了， 组内的 Cache 项)
    inner: Vec<Vec<CacheEntry<Data>>>
}

impl<Data: Default + Clone> Default for CacheEntry<Data> {
    fn default() -> Self {
        Self {
            valid: true,
            tag: 0,
            inner: Data::default()
        }
    }
}


impl<Data: Default + Clone> Cache<Data> {
    /// 初始化一个 Cache
    /// index_bits: 组索引位数
    /// entry_num: 关联度
    /// offset_bits: 内存块内地址位数
    pub unsafe fn init(index_bits: usize, entry_num: usize, offset_bits: usize) -> Self {
        let size = (2i32.pow(index_bits as u32) as usize, entry_num);
        let tag_bits = 64 - index_bits - offset_bits;
        let address = Address::new(tag_bits, index_bits);
        let inner: Vec<Vec<CacheEntry<Data>>> = vec![Vec::new(); size.0];
        Self {
            size,
            address,
            inner
        }
    }

    /// 访问 Cache
    pub fn access(&mut self, address: usize) -> CacheAccessResult {
        let tag = self.address.tag(address);
        let index = self.address.index(address);
        for (idx, entry) in self.inner[index].iter().enumerate() {
            if entry.tag == tag && entry.valid == true {
                return CacheAccessResult::Hit(index, idx);
            }
        }
        CacheAccessResult::Miss(index)
    }

    /// 替换 Cache 中的表项
    /// index: 组索引
    /// 返回是否驱逐
    pub fn insert_or_replace(&mut self, index: usize, entry: CacheEntry<Data>) -> bool {
        if self.inner[index].len() < self.size.1 {
            // 该组还没装满
            self.inner[index].push(entry);
            return false;
        } else {
            // 该组已经装满了
            // 从列表头部淘汰缓存项，并将新的缓存项压入列表尾部
            self.inner[index].remove(0);
            self.inner[index].push(entry);
            return true;
        }
    }

    /// 更新 Cache 的访问记录，将被访问的项放到列表尾部
    fn update(&mut self, index: usize, offset: usize) {
        let entry = self.inner[index].remove(offset);
        self.inner[index].push(entry);
    }

    /// 执行一条 trace 命令
    /// 返回 Cache 访问的情况和是否驱逐
    pub fn run_one_trace(&mut self, trace: TraceEntry) -> RunTraceResult {
        match trace.operation {
            Operation::InstructionLoad => { RunTraceResult::Skip }, // do nothing
            // 加载数据并写入，需要访问两次内存
            Operation::DataModify => {
                let address = trace.address;
                match self.access(address) {
                    CacheAccessResult::Miss(index) => {
                        // 不命中，加入 cache 项
                        let entry = CacheEntry {
                            valid: true,
                            tag: self.address.tag(address),
                            inner: Data::default()
                        };
                        if self.insert_or_replace(index, entry) {
                            RunTraceResult::MissReplaceHit
                        } else {
                            RunTraceResult::MissInsertHit
                        }
                    },
                    CacheAccessResult::Hit(index, offset) => {
                        // 命中，更新 cache 项
                        self.update(index, offset);
                        RunTraceResult::HitHit
                    }
                }
            },
            _ => {
                // L, S, M 三者在这里处理都是一样的
                let address = trace.address;
                match self.access(address) {
                    CacheAccessResult::Miss(index) => {
                        // 不命中，加入 cache 项
                        let entry = CacheEntry {
                            valid: true,
                            tag: self.address.tag(address),
                            inner: Data::default()
                        };
                        if self.insert_or_replace(index, entry) {
                            RunTraceResult::MissReplace
                        } else {
                            RunTraceResult::MissInsert
                        }
                    },
                    CacheAccessResult::Hit(index, offset) => {
                        // 命中，更新 cache 项
                        self.update(index, offset);
                        RunTraceResult::Hit
                    }
                }
            }
        }
    }
    
    /// 执行一个 traces 文件
    /// 返回 (命中， 缺失， 驱逐) 的次数
    pub fn run_traces(&mut self, traces: Traces) -> (usize, usize, usize) {
        let (mut hits, mut misses, mut evicts) = (0, 0, 0);
        for trace in traces.into_iter() {
            match self.run_one_trace(trace) {
                RunTraceResult::Skip => {},
                RunTraceResult::Hit => { hits += 1; },
                RunTraceResult::MissInsert => { misses += 1; },
                RunTraceResult::MissReplace => {
                    misses += 1;
                    evicts += 1;
                },
                RunTraceResult::MissInsertHit => {
                    hits += 1;
                    misses += 1;
                },
                RunTraceResult::MissReplaceHit => {
                    hits += 1;
                    misses += 1;
                    evicts += 1;
                },
                RunTraceResult::HitHit => {
                    hits += 2;
                }
            }
        }
        (hits, misses, evicts)
    }

}

pub enum CacheAccessResult {
    /// 缺失，返回需要替换的 Cache 项的组索引
    Miss(usize),
    /// 命中，返回命中 Cache 的组索引和组内索引
    Hit(usize, usize)
}

pub enum RunTraceResult {
    ///  跳过该 trace
    Skip,
    /// 命中
    Hit,
    /// 缺失并插入
    MissInsert,
    /// 缺失并驱逐
    MissReplace,
    /// 数据加载并修改，需要访问两次
    /// 第一次缺失并插入，第二次命中
    MissInsertHit,
    /// 第一次缺失并驱逐，第二次命中
    MissReplaceHit,
    /// 两次都命中
    HitHit
}

