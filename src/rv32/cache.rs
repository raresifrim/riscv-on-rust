
use std::sync::atomic::AtomicU8;

use crossbeam_channel::{Receiver, Sender};
use crate::rv32::rv32::{WordSize};

#[derive(PartialEq)]
pub enum MemoryRequestType {
    READ,
    WRITE
}

/// TODO: add methods for converting u8/u16/u32 etc to data vec for memory request
pub struct CacheDataRequest {
    pub request_type: MemoryRequestType,
    pub data_address: usize,
    pub data_size: WordSize,
    pub data: Option<Vec<u8>>,
}

/// TODO: add methods for converting byte array back to u8/u16/u32 etc for processor
pub struct CacheDataResponse {
    pub data: Vec<u8>,
}


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
    start_address: usize,
}

impl Cache {
    pub fn new(line_size: usize, num_lines: usize, start_address: usize) -> Self {
        let mut data = vec![];
        for _ in 0..num_lines {
            //let mut row: Vec<AtomicU8> = Vec::new();
            //row.resize_with(line_size, || AtomicU8::new(0));
            let row = vec![0u8; line_size];
            data.push(row.into_boxed_slice());
        }

        Self {
            data: data.into_boxed_slice(),
            line_size,
            num_lines,
            start_address
        }
    }

    pub fn modify_start_adress(&mut self, start_address: usize) {
        self.start_address = start_address;
    }

    /// get total size of memry in bytes
    pub fn size(&self) -> usize {
        self.num_lines * self.line_size
    }

    /// we return an array of bytes equal to te requested size(byte, half, word)
    /// its' the job of the processor to further extend it into a 32-bit register
    fn load_data(&self, address: usize, word_size:WordSize) -> Vec<u8> {
        let physical_address = address - self.start_address;
        let row_index: usize  = physical_address - (physical_address % (WordSize::WORD as usize));
        let byte_index: usize = physical_address % (WordSize::WORD as usize); 
        if physical_address >= self.num_lines + self.line_size {
            panic!("Address provided to Cache memory is out of bounds");
        }
        if (physical_address + (word_size as usize) -1) >= row_index + self.line_size {
           panic!("Address provided to Cache memory is not properly aligned."); 
        }
        
        // as in a real processor, data is copied from memory to a register
        // so we should not return a reference, but actually copy the data and pass it to the processor
        let mut data = vec![];
        for i in 0..(word_size as usize) {
            //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
            data.push(self.data[row_index][byte_index + i]);
        }
        
        data

    }

    /// for data store, is the other way: we receive a byte array and its size and we store it in the memory
    /// its the job of the processor to give as an exact array, but if it passes a larger array, we use the provided size to store the needed amount
    fn store_data(&mut self, address: usize, word_size:WordSize, data: Vec<u8>) {
        let physical_address = address - self.start_address;
        let row_index: usize  = physical_address - (physical_address % (WordSize::WORD as usize));
        let byte_index: usize = physical_address % (WordSize::WORD as usize); 
        if physical_address >= self.num_lines + self.line_size {
            panic!("Address provided to Cache memory is out of bounds");
        }
        if (physical_address + (word_size as usize) -1) >= row_index + self.line_size {
           panic!("Address provided to Cache memory is not properly aligned."); 
        }
        
        for i in 0..(word_size as usize) {
            //we respect the LE here: MSB on higher addresses in both cache memory and returned vector of bytes
            self.data[row_index][byte_index + i] = data[i];
        }
    
    }

    pub fn send_data_request(&mut self, request: CacheDataRequest) -> Option<CacheDataResponse> {
        if request.request_type == MemoryRequestType::READ {
            Some(CacheDataResponse { data: self.load_data(request.data_address, request.data_size) })
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
            self.store_data(request.data_address, request.data_size, data);
            None
        }
    }

    //read only request available to not lock core for write
    pub fn read_request(&self, request: CacheDataRequest) -> CacheDataResponse {
        assert!(request.request_type == MemoryRequestType::READ);
        CacheDataResponse { data: self.load_data(request.data_address, request.data_size)}
    }
}
