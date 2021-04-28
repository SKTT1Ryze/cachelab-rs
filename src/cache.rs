//! Cache 实现

use std::{default::Default};
use super::address::Address;

/// Cache 目录表项
#[derive(Default, Clone)]
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
    /// 数据
    inner: Vec<Vec<CacheEntry<Data>>>
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
        let inner: Vec<Vec<CacheEntry<Data>>> = vec![vec![CacheEntry::default(); size.1]; size.0];
        Self {
            size,
            address,
            inner
        }
    }

    pub fn access(&mut self, address: usize) -> CacheAccessResult {
        todo!()
    }
}

pub enum CacheAccessResult {
    /// 缺失，返回需要替换的 Cache 项的组索引和组内索引
    Miss(usize, usize),
    /// 命中
    /// todo: 应该返回命中的 Cache 项的数据
    Hit
}