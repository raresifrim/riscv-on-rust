
use crate::risc_soc::{memory_management_unit::{Address, MemoryDevice, MemoryDeviceType, MemoryRequest, MemoryRequestType, MemoryResponse}, risc_soc::WordSize};
use crate::risc_soc::memory_management_unit::MemoryResponseType;

/// Curently acts as direct momery, and not as a real cache, basically as in an Embedded Microprocessor
/// Can be used to represent Instruction or Data Memory for a RV processor, or both
/// Also there is no memory Virtualization at the moment, so addresses must be bounded by the defined sizes
#[derive(Debug)]
pub struct Cache {
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
    memory_type: MemoryDeviceType
}

impl MemoryDevice for Cache {
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
            line_size: 64, //some default cache line
            num_lines: 1024 * 1024, //some default ideal size (64MB), could be used for embedded MCUs
            start_address,
            end_address
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
        if request.request_type == MemoryRequestType::READ {
            self.load_data(request.data_address, request.data_size)
        } else {
            let data = match request.data {
                Some(d) => { 
                    if d.len() == 0 || d.len() < request.data_size as usize {
                        panic!("Trying to store less data then requested in cache memory!");
                    }
                    d
                },
                None => {
                    panic!("Made a request to store no data in cache memory!");
                }
            }; 
            self.store_data(request.data_address, request.data_size, data)
        }
    }

    //read only request available to not lock core for write
    #[inline]
    fn read_request(&self, request: MemoryRequest) -> MemoryResponse {
        assert!(request.request_type == MemoryRequestType::READ);
        self.load_data(request.data_address, request.data_size)
    }
}

impl Cache {
    /// start and end address ranges that should be cacheble (ex. a large region from the RAM memory)
    pub fn new_with_lines(cache_type: MemoryDeviceType, line_size: usize, num_lines: usize, start_address: Address) -> Self {
        
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
            end_address: start_address + size
        }
    }

    /// we return an array of bytes equal to te requested size(byte, half, word)
    /// its' the job of the processor to further extend it into a 32-bit register
    fn load_data(&self, address: Address, word_size:WordSize) -> MemoryResponse {
        let row_index  = address - (address % (WordSize::WORD as Address));
        let byte_index = address % (WordSize::WORD as Address); 
        if address > self.end_address || address < self.start_address {
            return MemoryResponse { data: vec![], valid: MemoryResponseType::CacheMiss };
        }
        if (address + (word_size as Address) -1) >= (row_index + self.line_size as Address) {
           panic!("Address provided to Cache memory is not properly aligned."); 
        }
        
        // as in a real processor, data is copied from memory to a register
        // so we should not return a reference, but actually copy the data and pass it to the processor
        let mut data = vec![];
        for i in 0..(word_size as usize) {
            //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
            data.push(self.data[row_index as usize][byte_index as usize + i]);
        }
        
        MemoryResponse { data, valid: MemoryResponseType::CacheHit }

    }

    /// for data store, is the other way: we receive a byte array and its size and we store it in the memory
    /// its the job of the processor to give as an exact array, but if it passes a larger array, we use the provided size to store the needed amount
    fn store_data(&mut self, address: Address, word_size:WordSize, data: Vec<u8>) -> MemoryResponse {
        let row_index  = address - (address % (WordSize::WORD as Address));
        let byte_index = address % (WordSize::WORD as Address); 
        if address > self.end_address || address < self.start_address {
            return MemoryResponse { data: vec![], valid: MemoryResponseType::CacheMiss };
        }
        if (address + (word_size as Address) -1) >= row_index + self.line_size as Address {
           panic!("Address provided to Cache memory is not properly aligned."); 
        }
        
        for i in 0..(word_size as usize) {
            //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
            self.data[row_index as usize][byte_index as usize + i] = data[i];
        }

        MemoryResponse { data: vec![], valid: MemoryResponseType::CacheHit }

    }
}
