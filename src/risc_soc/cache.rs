use crate::risc_soc::memory_management_unit::MemoryResponseType;
use crate::risc_soc::{
    memory_management_unit::{
        Address, MemoryDevice, MemoryDeviceType
    },
};

pub struct CacheResponse {
    pub cache_line: Vec<u8>,
    pub index: Address,
    pub tag: Address,
    pub status: MemoryResponseType,
}

pub trait Cache: MemoryDevice {
    /// start and end address ranges that should be cacheble (ex. a large region from the RAM memory)
    /// the start and end addresses here depende on the underlying cache implementation: ex. VIPT, PIPT, etc.
    fn new_with_lines(
        cache_type: MemoryDeviceType,
        line_size: usize,
        num_lines: usize,
        start_address: Address,
    ) -> Self where Self: Sized;

    fn load_data(&self, address: Address) -> CacheResponse;

    fn store_data(&mut self, address: Address, data: Vec<u8>) -> CacheResponse;

    /// function to validate address (ex. tag) and report a cache hit or miss
    fn is_address_hit(&self, address: Address) -> CacheResponse;
}
