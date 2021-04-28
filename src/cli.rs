//! 命令行参数解析部分

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

impl<'trace> std::default::Default for Cli<'trace> {
    fn default() -> Self {
        Self {
            help: false,
            track: false,
            s: 0,
            e: 0,
            b: 0,
            tracefile: ""
        }
    }
}

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