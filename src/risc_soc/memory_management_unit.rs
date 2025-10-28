use ahash::AHashMap;
use crate::risc_soc::risc_soc::WordSize;
use std::{fmt::Debug};

pub type Address = u64;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemoryRequestType {
    READ,
    WRITE
}

#[derive(Debug, PartialEq)]
pub enum MemoryResponseType {
    CacheHit,
    CacheMiss,
    Valid,
    InvalidAddress,
    UnalignedAddress,
    NotWrittable,
    NotReadable,
    NotExecutable,
    WrongMemoryMap,
}

/// Some generic memory types, such as cache, DRAM, UART, and a generic IOMMU which can handle other IOs
/// Ths is used as unique identifier in the MMU for the MemMap 
/// Can be modified/extended to support other types of memories as needed
#[derive(Debug, Eq, Hash, PartialEq, PartialOrd, Clone, Copy)]
pub enum MemoryDeviceType {
    L1ICACHE,
    L1DCACHE,
    L2CACHE,
    LLCACHE,
    MROM, //for generic boot 
    DRAM,
    FLASH, 
    UART0,
    DEBUG,
    IOMMU //reference to other IO units
}

/// TODO: add methods for converting u8/u16/u32 etc to data vec for memory request
#[derive(Clone)]
pub struct MemoryRequest {
    pub request_type: MemoryRequestType,
    pub data_address: Address,
    pub data_size: WordSize,
    pub data: Option<Vec<u8>>,
}

/// TODO: add methods for converting byte array back to u8/u16/u32 etc for processor
#[derive(Debug)]
pub struct MemoryResponse {
    pub data: Vec<u8>,
    pub status: MemoryResponseType
}

pub trait MemoryDevice {

    /// minumum amount of info required for a new memory device
    /// should always check that end_address is higher then start_address
    fn new(memory_type: MemoryDeviceType, start_address: Address, end_address: Address) -> Self where Self:Sized;

    //general data request for both read and write
    fn send_data_request(&mut self, request: MemoryRequest) -> MemoryResponse;

    //read only request
    fn read_request(&self, request: MemoryRequest) -> MemoryResponse;

    /// get total size of memory in bytes
    fn size(&self) -> usize;

    /// each memory address should define a start and an end address
    fn start_end_addresses(&self) -> (Address, Address);

    /// we should know what kind of memory device we are dealing with
    fn get_memory_type(&self) -> MemoryDeviceType;

    /// init portion of memory with specific data such as DRAM with a binary
    /// address specified here is the physical address of this memory array
    fn init_mem(&mut self, address: Address, data: &[u8]);

    /// helper function to debug various aspects of the memory
    fn debug(&self, start_address: Address, end_address: Address) -> std::fmt::Result;

}


/// Memory Management Unit is usually used in the CPU to translate VAs to PAs, but in here we see it as a an actual manager of the memory
/// This means that besides virtual memory translation, it can be used to arbitrate the transaction of memory
/// For example it can be used to decide if memory requests should be forwarded to cache, RAM or an IO
/// A process function can be passed to it, where it processes a memory request, and it can return a memory response to the CPU
pub struct MemoryManagementUnit {
    memmap: AHashMap<MemoryDeviceType, Box<dyn MemoryDevice + Send + Sync>>,
    process_fn: fn(&mut Self, MemoryRequest) -> MemoryResponse,
    // TODO: add TLB
    // TODO: add another structure to cache/hold the mapping between an address and the memory device
}

impl MemoryManagementUnit {
    pub fn new(
        memmap: AHashMap<MemoryDeviceType, Box<dyn MemoryDevice + Send + Sync>>,
        process_fn: fn(&mut Self, MemoryRequest) -> MemoryResponse, 
    ) -> Self {
        Self { memmap, process_fn}
    }

    pub fn add_memory_device(&mut self, memory_device: Box<dyn MemoryDevice + Send + Sync>) {
        
        if self.memmap.contains_key(&memory_device.get_memory_type()) {
            panic!("There is already a device of this type defined in the MMU!");
        }

        if memory_device.get_memory_type() < MemoryDeviceType::L2CACHE {
            panic!("The Memory Management Unit is responsable for handling memories starting at the L2 cache in the memory hierarchy")
        }

        if memory_device.get_memory_type() > MemoryDeviceType::LLCACHE {
            //cache memories are not mapped to a specific memory range, they just cache a specific range
            for mem in &self.memmap {
                if *mem.0 > MemoryDeviceType::LLCACHE && 
                    memory_device.start_end_addresses().0 >= mem.1.start_end_addresses().0 &&
                    memory_device.start_end_addresses().0 <= mem.1.start_end_addresses().1 {
                        panic!("This memory device overlaps with other memory ranges already defined in the MMU!")
                } 
            }
        }
        
        self.memmap.insert(memory_device.get_memory_type(), memory_device);
    }

    pub fn init_section_into_memory(&mut self, address: Address, data: &[u8]) {
        for device in &mut self.memmap{
            let (start_address, end_address) = device.1.start_end_addresses();
            if address >= start_address && address < end_address {
                assert!(address + data.len() as Address <= end_address);   
                device.1.init_mem(address, data); 
            }
        }   
    }

    /// the main logic of the MemoryManagementUnit  whould be handled in its process_fn, includinf thigs such as address translation
    pub fn process_memory_request(&mut self, memory_request: MemoryRequest) -> MemoryResponse {
        (self.process_fn)(self, memory_request)
    }

}

impl Debug for MemoryManagementUnit {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for mem in &self.memmap {
            println!("{:?}: ({:X} -> {:X}) ", mem.0, mem.1.start_end_addresses().0, mem.1.start_end_addresses().1);
        }
        Ok(())
    }
}

/// If a MMU is not needed, it can be defaulted to this, which creates an empty structure
/// But it provides a basic process function which checks the data request address and forwards it to an available device in that memory range
impl Default for MemoryManagementUnit {
    fn default() -> Self {
        Self { 
            memmap: AHashMap::default(),
            process_fn: |_self, _request| {
                assert!(!_self.memmap.is_empty());
                for device in &mut _self.memmap {
                    let (start_address, end_address) = device.1.start_end_addresses();
                    if _request.data_address >= start_address && _request.data_address < end_address {
                        return device.1.send_data_request(_request);
                    }
                }
                MemoryResponse { data: vec![], status: MemoryResponseType::InvalidAddress }
            }
        }
    }
}