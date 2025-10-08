use super::pipeline_stage::*;
use crate::rv32::cache::{Cache, CacheDataRequest, CacheDataResponse};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;

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
pub struct RV32Core {
    stages: Vec<PipelineStage>,
    icache: Cache,
    dcache: Cache,
    registers: Registers,
    program_counter: AtomicU32
}

impl RV32Core {
    pub fn new(
        num_stages: usize,
        icache_width: usize,
        icache_legth: usize,
        dcache_width: usize,
        dcache_legth: usize,
    ) -> Self {
        // initialize cache memories with a default data address
        let icache = Cache::new(icache_width, icache_legth, 0x8000_0000);
        let dcache = Cache::new(dcache_width, dcache_legth, 0x8000_0000 + icache.size());
        // create an empty array of stages
        let stages = Vec::with_capacity(num_stages);
        Self {
            stages,
            icache,
            dcache,
            registers: Registers::default(),
            program_counter: AtomicU32::new(0x8000_0000)
        }
    }

    /// dynamically add stages to the processor creating a custom pipeline
    /// stages should be created before hand and passed here already initialized
    pub fn add_stage(&mut self, stage: PipelineStage) -> &mut Self {
        self.stages.push(stage);
        self
    }

    /// load a binary file containing the code to be executed
    pub fn load_binary() {
        todo!()
    }

    pub fn icache_request(&self, request: CacheDataRequest) -> CacheDataResponse {
        self.icache.read_request(request)
    }

    pub fn dcache_request(&mut self, request: CacheDataRequest) -> Option<CacheDataResponse> {
        self.dcache.send_data_request(request)
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

impl Deref for RV32Core {
    type Target = Registers;
    fn deref(&self) -> &Self::Target {
        &self.registers
    }
}

impl DerefMut for RV32Core {
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
