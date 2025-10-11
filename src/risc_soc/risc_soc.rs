use super::pipeline_stage::*;
use crate::risc_soc::cache::{Cache};
use crate::risc_soc::memory_management_unit::{Address, MemoryManagementUnit, MemoryRequest, MemoryResponse};
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::fs;
use object::{elf, Endianness};
use object::read::elf::{FileHeader, SectionHeader};
use crate::risc_soc::memory_management_unit::MemoryDevice;


/// type used to represent data inside the RV32 which is always a 4 byte word
pub type RV32Word = u32;

/// sizes of the supported words in bytes
#[derive(Debug, Clone, Copy)]
pub enum WordSize {
    BYTE = 1,
    HALF = 2,
    WORD = 4,
    DOUBLE = 8,
}

#[derive(Debug)]
pub struct RiscCore {
    pub stages: Vec<PipelineStage>,
    icache: Option<Cache>,
    dcache: Option<Cache>,
    pub registers: Registers,
    pub program_counter: AtomicU32,
    mmu: MemoryManagementUnit
}

impl RiscCore {
    pub fn new(
        num_stages: usize
    ) -> Self {
        // create an empty array of stages
        let stages = Vec::with_capacity(num_stages);
        Self {
            stages,
            icache: None,
            dcache: None,
            registers: Registers::default(),
            program_counter: AtomicU32::new(0x8000_0000),
            mmu: MemoryManagementUnit::default()
        }
    }

    pub fn add_l1_icache(
        &mut self,
        icache: Cache,
        dcache: Cache
    ) -> &mut Self {
        self.icache = Some(icache);
        self.dcache = Some(dcache);
        self
    }

    pub fn add_mmu(&mut self, mmu: MemoryManagementUnit){
        self.mmu = mmu;
    }

    pub fn icache_read(&self, request: MemoryRequest) -> MemoryResponse {
        if self.icache.is_some() {
            self.icache.as_ref().unwrap().read_request(request)
        } else {
            panic!("An L1Cache request was made, but there is no L1Cache configured on this core!")
        }
    }

    pub fn dcache_read(&self, request: MemoryRequest) -> MemoryResponse {
        if self.dcache.is_some() {
            self.dcache.as_ref().unwrap().read_request(request)
        } else {
           panic!("An L1Cache request was made, but there is no L1Cache configured on this core!") 
        }
    }
    
    pub fn icache_write(&mut self, request: MemoryRequest) -> MemoryResponse {
        if self.icache.is_some() {
            self.icache.as_mut().unwrap().send_data_request(request)
        } else {
            panic!("An L1Cache request was made, but there is no L1Cache configured on this core!")
        }
    }

    pub fn dcache_write(&mut self, request: MemoryRequest) -> MemoryResponse {
        if self.dcache.is_some() {
            self.dcache.as_mut().unwrap().send_data_request(request)
        } else {
            panic!("An L1Cache request was made, but there is no L1Cache configured on this core!")
        }
    }

    /// dynamically add stages to the processor creating a custom pipeline
    /// stages should be created before hand and passed here already initialized
    pub fn add_stage(&mut self, stage: PipelineStage) -> &mut Self {
        self.stages.push(stage);
        self
    }

    /// load a binary file containing the code to be executed
    pub fn load_binary(&self, elf_path: String) {
        let data = fs::read(elf_path).expect("Could not read provided elf file path");
        let elf = elf::FileHeader32::<object::Endianness>::parse(&*data).expect("Failed to parse elf");
        
        let endian = elf.endian().expect("Failed to parse endianess");
        assert!(endian == Endianness::Little);

        //read sections
        let sections = elf.sections(endian, &*data).expect("Failed to parse sections of elf file");
        for section in sections.iter() {
            let mut name: String = Default::default();
            sections.section_name(endian, section).unwrap().read_to_string(&mut name).unwrap();
            let data = section.data(endian, &*data).expect("Failed to read section data");
            let address = section.sh_addr.get(endian);
            let size = section.sh_size.get(endian);
            println!("{name} @{:X}:{:X}", address, size);
            println!("Data: {:?}",data);
        }
        
    }

    pub fn get_pc(&self) -> u32 {
        self.program_counter.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn set_pc(&self, pc:u32) {
        self.program_counter.store(pc, std::sync::atomic::Ordering::SeqCst);
    }

    /// start execution of loaded program
    pub fn run(self) {
        //start execution of all stages
        let mut handlers = vec![];
        for pipeline_stage in self.stages {
            handlers.push(pipeline_stage.run());
        }

        //wait for all stages to finish execution
        for h in handlers {
            h.join().unwrap();
        }
    }
}

impl Deref for RiscCore {
    type Target = Registers;
    fn deref(&self) -> &Self::Target {
        &self.registers
    }
}

impl DerefMut for RiscCore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registers
    }
}

#[derive(Debug, Default)]
pub struct Registers([u32; 32]);

impl Registers {
    pub fn read_regs(&self, rs1_address: usize, rs2_address: usize) -> (u32, u32) {
        assert!(rs1_address < 32);
        assert!(rs2_address < 32);
        (self.0[rs1_address], self.0[rs2_address])
    }

    pub fn write_reg(&mut self, rd_address: usize, rd: u32) {
        assert!(rd_address < 32);
        if rd_address > 0 {
            //should never overwrite x0
            self.0[rd_address] = rd;
        }
    }
}
