# 华中科技大学计算机系统结构实验报告
计科校交 1801 车春池  

## Cache 模拟器实验
### 实验目的
+ 理解 Cache 工作原理
+ 如何实现一个高效的模拟器
### 实验环境
+ Linux 64-bit, Rust 语言
### 实验思路
#### 实验测试
实验是通过给出一系列的 trace 文件，作为程序的输入参数，然后利用脚本程序比对程序输出的结果和参考实现输出的结果进行测试。  
脚本程序会将比对结果和分数打印出来。  
在这个测试的原理基础上，程序的实现就与语言无关了。因此我选择使用 Rust 语言来完成这个实验。  
#### 命令行参数解析
实验的第一个要求是需要我们正确处理命令行参数。首先我定义了一个结构体来存放命令行参数的信息：  
```Rust
/// 解析命令行参数
#[derive(Debug)]
pub struct Cli<'trace> {
    /// 是否显示帮助信息
    pub help: bool,
    /// 是否显示轨迹信息
    pub track: bool,
    /// 组索引位数
    pub s: u32,
    /// 关联度（每组包含的缓存行数）
    pub e: u32,
    /// 内存块内地址位数
    pub b: u32,
    /// 内存访问轨迹文件名
    pub tracefile: &'trace str
}
```
然后我为这个结构体实现一个方法用于从字符串数组转换到具体的结构体实例：  
```Rust
impl<'trace> Cli<'trace> {
    pub fn parse(args: Vec<&str>) -> Cli {
        let mut cli = Cli::default();
        let mut check = [0, 0, 0, 0];
        let mut pos = 0;
        while pos < args.len() {
            match args[pos] {
                "-h" => cli.help = true,
                "-v" => cli.track = true,
                "-s" => {
                    // 组索引位数
                    cli.s = args[pos + 1].parse::<u32>().unwrap();
                    check[0] = 1;
                    pos += 1;
                },
                "-E" => {
                    // 关联度
                    cli.e = args[pos + 1].parse::<u32>().unwrap();
                    check[1] = 1;
                    pos += 1;
                },
                "-b" => {
                    // 内存块内地址位数
                    cli.b = args[pos + 1].parse::<u32>().unwrap();
                    check[2] = 1;
                    pos += 1;
                },
                "-t" => {
                    // 内存访问轨迹文件名
                    cli.tracefile = args[pos + 1];
                    check[3] = 1;
                    pos += 1;
                }
                _ => panic!("Invalid arguments!")
            }
            pos += 1;
        }
        check.iter().for_each(|x| {
            if *x == 0 {
                panic!("Lack arguments!")
            }
        });
        cli
    }
}
```

这样子我们就可以很方便地解析命令行参数了：  
```Rust
fn main() {
    let args: Vec<String> = env::args().collect();
    let args_input: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let cli = Cli::parse(args_input);
    println!("{:?}", cli);
}
```

#### 虚拟地址解析
访问 Cache 的时候一般都是通过虚拟地址访问，这时候虚拟地址的解析就会和 Cache 的参数（比如容量，组相联数目等）相关。  
因此我们需要根据输入参数的不同来调整虚拟地址的解析方式。  
我采用的办法是创建一个抽象虚拟地址的结构体和为这个结构体实现一些方法：  
```Rust
//! 64 位 16 进制地址

use bit_field::BitField;

pub struct Address {
    tag_bits: usize,
    index_bits: usize,
}

impl Address {
    pub fn new(tag_bits: usize, index_bits: usize) -> Self {
        Self {
            tag_bits,
            index_bits
        }
    }
    
    pub fn tag(&self, address: usize) -> usize {
        let start = 64 - self.tag_bits;
        address.get_bits(start..64)
    }

    pub fn index(&self, address: usize) -> usize {
        let start = 64 - self.tag_bits - self.index_bits;
        let end = 64 - self.tag_bits;
        address.get_bits(start..end)
    }

    pub fn _offset(&self, address: usize) -> usize {
        let end = 64 - self.tag_bits - self.index_bits;
        address.get_bits(0..end)
    }
}
```
这样就能在代码中很方便地解析虚拟地址了。  
#### 轨迹文件解析
实验包根据一些列轨迹文件用于测试，它记录了某一程序在运行过程中访问内存的序列及其参数（地址，大小等）。  
每行记录一次或两次的内存访问的信息，格式为：operation address, size  
operation 表示内存访问的类型：指令装载，数据装载，数据存储，数据修改。  
address 表示 64 位 16 进制虚拟地址。  
size 表示访问的内存字节数量。  
首先定义一个结构体保存每行 trace 文件的数据：  
```Rust
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
```

然后定义一个结构体保存一个 trace 文件的信息：  
```Rust
#[derive(Debug)]
pub struct Traces {
    inner: Vec<TraceEntry>
}
```

为 Traces 结构体实现一个从 trace 文件到具体结构体实例的方法和打印内存访问轨迹的方法：  
```Rust
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

    /// 打印轨迹
    pub fn print_trace(&self) {
        self.inner.iter().for_each(|e| {
            println!("{} {}, {}", e.operation.as_str(), e.address, e.size);
        })
    }
}
```
实现了这些方法后，我们可以很方便地解析一个轨迹文件：  
```Rust
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let args_input: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    let cli = Cli::parse(args_input);
    let traces = Traces::from_path(cli.tracefile)?;
}
```

#### Cache 实现
首先通过一个结构体来抽象 Cache 的表项：  
```Rust
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
```

可以看到一个 Cache 的表项包括有效位，标识位和数据位。  
然后定义一个叫做 Cache 的结构体：  
```Rust
pub struct Cache<Data: Default + Clone> {
    /// Cache 的组数, 每组包含的项数
    size: (usize, usize),
    /// 地址配置
    address: Address,
    /// 组内的 Cache 项
    inner: Vec<Vec<CacheEntry<Data>>>
}
```
可以看到 Cache 的 inner 成员是 CacheEntry 的二维数组，保存着多个 Cache 表项。  
然后通过两个枚举抽象访问 Cache 的结果和运行一行 trace 记录的结果：  
```Rust
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
```

最后在这些抽象基础之上，通过 LRU 算法来实现 Cache 访问的模拟过程：  
```Rust
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
```
### 实验结果和分析
`main.rs` 代码：  
```Rust
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

    if cli.help {
        help();
    }

    let traces = Traces::from_path(cli.tracefile)?;
    
    if cli.track {
        traces.print_trace();
    }
    
    let mut cache: Cache<usize> = unsafe { Cache::init(cli.s as usize, cli.e as usize, cli.b as usize) };
    let (hits, misses, evicts) = cache.run_traces(traces);

    let mut f = File::create(".csim_results")?;
    f.write(format!("{} {} {}\n", hits, misses, evicts).as_bytes())?;
    Ok(())
}


fn help() {
    println!("Usage:");
    println!("  csim [-hv] -s <s> -E <E> -b <b> -t <tracefile>")
}
```

用 `test-csim` 脚本文件进行测试：  
```bash
make test
```

结果：  
![test_res](./test_res.png)  

可以看到测试用例全部通过。  

## 总结和体会
通过这次实验，我通过写代码模拟了一遍 Cache 的工作过程，对 Cache 的工作原理更加熟悉了。  
本次实验我通过 Rust 语言实现，这使得我对 Rust 语言的写法更加熟悉，同时也更加深刻认识到了 Rust 语言在系统编程中的优越性。  

## 对实验课程的建议
多点实验，少些理论。  
