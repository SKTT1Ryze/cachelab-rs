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