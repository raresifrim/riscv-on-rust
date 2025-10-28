use std::mem;

use crate::risc_soc::memory_management_unit::{MemoryResponseType};
use crate::risc_soc::{
    memory_management_unit::{
        Address, MemoryDevice, MemoryDeviceType, MemoryRequest,
        MemoryRequestType, MemoryResponse,
    },
    risc_soc::WordSize,
};
use crate::risc_soc::cache::CacheResponse;
use crate::risc_soc::cache::Cache;

/// Acts as direct momery, and not as a real cache, basically as in an Embedded/Baremetal Microprocessor
/// Can be used to represent Instruction or Data Memory for a RV processor, or both
/// Also there is no memory Virtualization for this kind of memory, so addresses must be bounded by the defined sizes
#[derive(Debug)]
pub struct MCUCache {
    data: Box<[Box<[u8]>]>,
    /// line size as number of bytes
    line_size: usize,
    /// number of lines per cache
    num_lines: usize,
    /// as we can load an arbitrary elf binary with a defined start address for code and data
    /// we save it here as base for actual address calculation inside the data array
    /// otherwise if we use a memory hierarchy, we can define the star and end memory region that should be cacheble
    start_address: Address,
    end_address: Address,
    /// the memory type of the device
    memory_type: MemoryDeviceType,
}

impl MemoryDevice for MCUCache {
    fn new(cache_type: MemoryDeviceType, start_address: Address, end_address: Address) -> Self {
        assert!(end_address > start_address);
        assert!(cache_type <= MemoryDeviceType::LLCACHE);

        let mut data = vec![];
        for _ in 0..1024 * 1024 {
            let row = vec![0u8; 64]; //default kind of cache line
            data.push(row.into_boxed_slice());
        }

        Self {
            memory_type: cache_type,
            data: data.into_boxed_slice(),
            line_size: 64,          //some default cache line
            num_lines: 1024 * 1024, //some default ideal size (64MB), could be used for embedded MCUs
            start_address,
            end_address,
        }
    }

    /// get total size of memory in bytes
    #[inline]
    fn size(&self) -> usize {
        self.num_lines * self.line_size
    }

    #[inline]
    fn start_end_addresses(&self) -> (Address, Address) {
        (self.start_address, self.end_address)
    }

    #[inline]
    fn get_memory_type(&self) -> MemoryDeviceType {
        self.memory_type
    }

    #[inline]
    fn send_data_request(&mut self, request: MemoryRequest) -> MemoryResponse {

        let response;
        if request.request_type == MemoryRequestType::READ {
            response = self.read_request(request);
        } else {
            let data = match request.data {
                Some(mut d) => {
                    if d.len() == 0 || d.len() < request.data_size as usize {
                        panic!("Trying to store less data then requested in cache memory!");
                    }
                    if (request.data_size as usize) < d.len() {
                        d.truncate(request.data_size as usize);
                    }
                    d
                }
                None => {
                    panic!("Made a request to store no data in cache memory!");
                }
            };
            let cache_response = self.store_data(request.data_address, data);
            response = MemoryResponse{
                data: vec![],
                status: cache_response.status
            }
        }
        return response;
    }

    //read only request available to not lock core for write
    #[inline]
    fn read_request(&self, request: MemoryRequest) -> MemoryResponse {
        assert!(request.request_type == MemoryRequestType::READ);
        let cache_response = self.load_data(request.data_address);
        let byte_index = (request.data_address - self.start_address) % self.line_size as u64; 
        let mut data = vec![0u8; request.data_size as usize];
        if cache_response.status == MemoryResponseType::CacheHit { 
            for i in 0..request.data_size as usize{
                data[i] = cache_response.cache_line[byte_index as usize + i];
            }
        }
        MemoryResponse { data, status: cache_response.status }
    }

    fn init_mem(&mut self, address: Address, data: &[u8]) {
        for byte in 0..data.len() {
            let current_address = address as usize + byte;
            let byte_index = current_address % self.line_size;
            let row_index = current_address / self.line_size;
            self.data[row_index][byte_index] = data[byte];
        }
    }

    fn debug(&self, start_address: Address, end_address: Address) -> std::fmt::Result {
        assert!(start_address >= self.start_address && end_address <= self.end_address);
        println!("\nMemory {:?}: {{", self.memory_type);
        let num_words = end_address - start_address;
        let num_lines = num_words / self.line_size as u64;
        for i in 0..=num_lines {
            let current_line = start_address + i * self.line_size as u64; 
            print!("{:X}: ", current_line);
            for w in 0..self.line_size {
                if w % 4 == 0 {
                    print!(" ");
                }
                let current_line = (current_line - self.start_address) as usize;
                print!("{:X}", self.data[current_line][w]);
            }
            print!("\n")
        }
        println!("}}");
        Ok(())
    }
}

impl Cache for MCUCache {
    fn new_with_lines(
        cache_type: MemoryDeviceType,
        line_size: usize,
        num_lines: usize,
        start_address: Address,
    ) -> Self {
        //we should at least provide a line size equal to the word size of the CPU
        assert!(num_lines > 0 && line_size >= WordSize::WORD as usize);
        assert!(cache_type <= MemoryDeviceType::LLCACHE);

        let mut data = vec![];
        for _ in 0..num_lines {
            //let mut row: Vec<AtomicU8> = Vec::new();
            //row.resize_with(line_size, || AtomicU8::new(0));
            let row = vec![0u8; line_size];
            data.push(row.into_boxed_slice());
        }

        let size = (num_lines * line_size) as Address;

        Self {
            memory_type: cache_type,
            data: data.into_boxed_slice(),
            line_size,
            num_lines,
            start_address,
            end_address: start_address + size,
        }
    }

    fn load_data(&self, address: Address) -> CacheResponse {
        let mut response = self.translate_address(address);
        if response.status == MemoryResponseType::CacheHit {
            // as in a real processor, data is copied from memory to a register
            // so we should not return a reference, but actually copy the data and pass it to the processor
            for i in 0..self.line_size {
                //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
                response
                    .cache_line
                    .push(self.data[response.index as usize][i]);
            }
        }

        response
    }

    /// for data store, is the other way: we receive a byte array and its size and we store it in the memory
    /// its the job of the processor to give as an exact array, but if it passes a larger array, we use the provided size to store the needed amount
    fn store_data(&mut self, address: Address, data: Vec<u8>) -> CacheResponse {
        let mut response = self.translate_address(address);
        if response.status == MemoryResponseType::CacheHit {
            let byte_index = address % self.line_size as u64;
            if (address - self.start_address + (data.len() as Address) - 1)
                >= (response.index + self.line_size as Address)
            {
                response.index = 0;
                response.status = MemoryResponseType::UnalignedAddress;
                return response;
            }

            for i in 0..data.len() {
                //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
                self.data[response.index as usize][byte_index as usize + i] = data[i];
            }
        }
        response
    }

    /// We are using this cache memory as direct ram/rom memory for our baremetal CPU
    /// So we are using the start and end address to define the memory regions for .text and .data sections
    /// And whatever Virtual Address we are receiving, we are subtractng the defined start address from it
    fn translate_address(&self, address: Address) -> CacheResponse {
        if address > self.end_address || address < self.start_address {
            return CacheResponse {
                cache_line: vec![],
                index: 0,
                tag: 0,
                status: MemoryResponseType::WrongMemoryMap,
            };
        }
        let address = address - self.start_address;
        let row_index = address / self.line_size as u64;
        CacheResponse {
            cache_line: vec![],
            index: row_index,
            tag: 0,
            status: MemoryResponseType::CacheHit,
        }
    }
}
