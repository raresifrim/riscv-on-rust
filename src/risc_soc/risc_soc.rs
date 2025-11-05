use super::pipeline_stage::*;
use crate::risc_soc::cache::Cache;
use crate::risc_soc::cdb::CommonDataBus;
use crate::risc_soc::memory_management_unit::{
    Address, MemoryDeviceType, MemoryManagementUnit, MemoryRequest,
    MemoryResponse, MemoryResponseType,
};
use object::read::elf::{FileHeader, SectionHeader};
use object::{Endianness, elf};
use std::fmt::Debug;
use std::fs;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex, RwLock};

/// type used to represent data inside the RiscCore (defaulted to u32 for RV32)
/// can be overwritten to u64 if RV64 is intended for implementation
pub type RiscWord = u32;

/// sizes of the supported words in bytes
#[derive(Debug, Clone, Copy)]
pub enum WordSize {
    BYTE = 1,
    HALF = 2,
    WORD = 4,
    DOUBLE = 8,
}

/// should usually represent main control signals such as a reset and enable
type PipelineControlSignals = Vec<AtomicBool>;
const RESET_SIGNAL:usize = 0x0;
const ENABLE_SIGNAL: usize= 0x1;

pub struct RiscCore {
    pub debug: bool,
    pub stages: Vec<Arc<Mutex<PipelineStage>>>,
    pub icache: Option<Arc<RwLock<Box<dyn Cache + Send + Sync>>>>,
    pub dcache: Option<Arc<RwLock<Box<dyn Cache + Send + Sync>>>>,
    pub registers: Registers,
    pub program_counter: AtomicU64,
    pub mmu: Arc<RwLock<MemoryManagementUnit>>,
    pub clock_period: Option<u128>, //nanoseconds
    pub cdb: CommonDataBus,
    pub pipeline_control_signals: Vec<PipelineControlSignals>
}

impl RiscCore {
    pub fn new(num_stages: usize, clock_period: Option<u128>, debug: bool) -> Self {
        // create an empty array of stages
        let stages = Vec::with_capacity(num_stages);
        let pipeline_control_signals = Vec::with_capacity(num_stages);
        let cdb = CommonDataBus::new(num_stages, clock_period, debug);
        Self {
            stages,
            icache: None,
            dcache: None,
            registers: Registers::default(),
            program_counter: AtomicU64::new(0x8000_0000),
            mmu: Arc::new(RwLock::new(MemoryManagementUnit::default())),
            cdb,
            clock_period,
            debug,
            pipeline_control_signals
        }
    }

    pub fn enable_debug(&mut self, debug: bool){
        self.debug = debug;
        for cdb_lane in &mut self.cdb.bus {
            for wire in cdb_lane.1 {
                wire.enable_debug(debug);
            }
        }
        for pipeline_stage in &self.stages{
            let mut lock = pipeline_stage.lock().unwrap();
            lock.enable_debug(debug);
        }
    }

    pub fn add_l1_cache(
        &mut self,
        icache: Box<dyn Cache + Send + Sync>,
        dcache: Box<dyn Cache + Send + Sync>,
    ) -> &mut Self {
        self.icache = Some(Arc::new(RwLock::new(icache)));
        self.dcache = Some(Arc::new(RwLock::new(dcache)));
        self
    }

    pub fn add_mmu(&mut self, mmu: MemoryManagementUnit) {
        self.mmu = Arc::new(RwLock::new(mmu));
    }

    pub fn set_clock_period(&mut self, nanosecs: u128) {
        self.clock_period = Some(nanosecs);
    }

    pub fn icache_request(&self, request: MemoryRequest) -> MemoryResponse {
        if self.icache.is_some() {
            let cache_response = self
                .icache.as_ref()
                .unwrap()
                .write()
                .unwrap()
                .send_data_request(request.clone());
            if cache_response.status == MemoryResponseType::CacheHit {
                cache_response
            } else {
                self.mmu.write().unwrap().process_memory_request(request)
            }
        } else {
            panic!("An L1Cache request was made, but there is no L1Cache configured on this core!")
        }
    }

    pub fn dcache_request(&self, request: MemoryRequest) -> MemoryResponse {
        if self.dcache.is_some() {
            let cache_response = self
                .dcache.as_ref()
                .unwrap()
                .write()
                .unwrap()
                .send_data_request(request.clone());
            if cache_response.status == MemoryResponseType::CacheHit {
                cache_response
            } else {
                self.mmu.write().unwrap().process_memory_request(request)
            }
        } else {
            panic!("An L1Cache request was made, but there is no L1Cache configured on this core!")
        }
    }

    /// dynamically add stages to the processor creating a custom pipeline
    /// stages should be created before hand and passed here already initialized
    pub fn add_stage(&mut self, mut stage: PipelineStage) -> &mut Self {
        if self.stages.len() + 1 > self.stages.capacity() {
            panic!("Trying to add more stages then configured for current core!");
        }
        stage.enable_debug(self.debug);
        self.stages.push(Arc::new(Mutex::new(stage)));
        let mut control_signals = vec![];
        control_signals.push(AtomicBool::new(false)); //reset
        control_signals.push(AtomicBool::new(true)); //enable
        self.pipeline_control_signals.push(control_signals);
        self
    }

    pub fn reset_stage(&self, stage_index: usize, reset_value: bool) {
        let stage_control_signals = &self.pipeline_control_signals[stage_index];
        stage_control_signals[RESET_SIGNAL].store(reset_value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn enable_stage(&self, stage_index: usize, enable_value: bool) {
        let stage_control_signals = &self.pipeline_control_signals[stage_index];
        stage_control_signals[ENABLE_SIGNAL].store(enable_value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_stage_reset(&self, stage_index: usize) -> bool {
        let stage_control_signals = &self.pipeline_control_signals[stage_index];
        stage_control_signals[RESET_SIGNAL].load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn is_stage_enabled(&self, stage_index: usize) -> bool {
        let stage_control_signals = &self.pipeline_control_signals[stage_index];
        stage_control_signals[ENABLE_SIGNAL].load(std::sync::atomic::Ordering::SeqCst)
    }

    /// load a binary file containing the code to be executed
    pub fn load_binary(&mut self, elf_path: &str, memory_device: MemoryDeviceType) {
        let data = fs::read(elf_path).expect("Could not read provided elf file path");
        let elf =
            elf::FileHeader32::<object::Endianness>::parse(&*data).expect("Failed to parse elf");

        let endian = elf.endian().expect("Failed to parse endianess");
        assert!(endian == Endianness::Little);

        //read sections
        let sections = elf
            .sections(endian, &*data)
            .expect("Failed to parse sections of elf file");
        let section_headers = sections.iter().filter(|x| {
            let mut name: String = Default::default();
            sections
                .section_name(endian, x)
                .unwrap()
                .read_to_string(&mut name)
                .unwrap();
            name.contains(".text")
                || name.contains(".data")
                || name.contains(".sdata")
                || name.contains(".rodata")
                || name.contains(".bss")
                || name.contains(".sbss")
        });
        for section in section_headers {
            let mut name: String = Default::default();
            sections
                .section_name(endian, section)
                .unwrap()
                .read_to_string(&mut name)
                .unwrap();
            let data = section
                .data(endian, &*data)
                .expect("Failed to read section data");
            let address = section.sh_addr.get(endian) as Address;
            //let size = section.sh_size.get(endian);
            //println!("{name} @{:X}:{:X}", address, size);
            //println!("Data: {:?}", data);

            if memory_device < MemoryDeviceType::L2CACHE {
                // in the case where we are using cache memories as the only level of memory
                // we split the sections as .text in icache and everything else in dcache
                assert!(self.icache.is_some() && self.dcache.is_some());
                let mut icache = self.icache.as_ref().unwrap().write().unwrap();
                let mut dcache = self.dcache.as_ref().unwrap().write().unwrap();
                if name.contains(".text") {
                    let (start, end) = icache.start_end_addresses();
                    let cache_size = icache.size();
                    assert!(
                        address >= start
                            && address < end
                            && (address - start) as usize + data.len() < cache_size
                    );
                    icache.init_mem(address - start, data);
                } else {
                    let (start, end) = dcache.start_end_addresses();
                    let cache_size = dcache.size();
                    assert!(
                        address >= start
                            && address < end
                            && (address - start) as usize + data.len() < cache_size
                    );
                    dcache.init_mem(address - start, data);
                }

            } else {
                //map to the selected memory device (ex. DRAM)
                // here, usually all sections will be mapped in same memory region
                let mut mmu = self.mmu.write().unwrap();
                mmu.init_section_into_memory(address as Address, data);
            }
        }
    }

    pub fn get_pc(&self) -> RiscWord {
        self.program_counter
            .load(std::sync::atomic::Ordering::SeqCst) as RiscWord
    }

    pub fn set_pc(&self, pc: RiscWord) {
        self.program_counter
            .store(pc as u64, std::sync::atomic::Ordering::SeqCst);
    }

    #[inline]
    fn trace_asm_instr(&self, stage: &mut PipelineStage, print_asm: bool, disassmble: bool) {
        use crate::risc_soc::instruction_asm::rv32_asm;
        if print_asm {
            // handle the print/log of the current instruction
            let mut instr_bin = stage.instruction.0;
            if stage.index == 0x0 && !stage.data_out.is_empty() {
                //special case for first stage in pipeline
                instr_bin = stage.data_out.get_u32(0x0);
                stage.instruction = Instruction(instr_bin);
            }

            if disassmble {
                let asm_instr = rv32_asm(instr_bin);
                if self.debug {
                    println!(
                        "Pipeline Stage {} @ClockCycle {} -> Instruction:{}(0x{:X})",
                        stage.name, stage.clock_cycle, asm_instr, stage.instruction.0
                    );
                } else {
                    tracing::info!(
                        "Pipeline Stage {} @ClockCycle {} -> Instruction:{}(0x{:X})",
                        stage.name,
                        stage.clock_cycle,
                        asm_instr,
                        stage.instruction.0
                    );
                }
            } else {
                if self.debug {
                    println!(
                        "Pipeline Stage {} @ClockCycle {} -> Instruction: 0x{:X}",
                        stage.name, stage.clock_cycle, stage.instruction.0
                    );
                } else {
                    tracing::info!(
                        "Pipeline Stage {} @ClockCycle {} -> Instruction: 0x{:X}",
                        stage.name,
                        stage.clock_cycle,
                        stage.instruction.0
                    );
                }
            }
        }
    }

    /// start execution of loaded program
    /// if running in debug mode it will run a single instruction through all pipeline stages and the run function must be called for each new instruction
    pub fn run(&mut self, num_clock_cycles: Option<u64>) {
        //start execution of all stages
        use std::thread::sleep;
        use std::time::Instant;
        use std::sync::Barrier;

        let barrier = Barrier::new(self.stages.len());
        std::thread::scope(|s| {
                        
            for arc_stage in &self.stages {
                s.spawn(|| {
                    let clock_period = self.clock_period;
                    let mut stage = arc_stage.lock().unwrap();
                    loop {
                        
                        self.cdb.clear(stage.index); //clear all wires of current stage before new clock edge so that we can react to a change
                        barrier.wait(); //clock boundary
                        let pipeline_payload;
                        
                        // read from previous pipeline stage if available
                        if stage.input_channel.is_some() {
                            match stage.input_channel.as_ref().unwrap().try_recv() {
                                Ok(data_input) => {
                                    stage.instruction = data_input.instruction;
                                    stage.data_in = data_input.data;
                                },

                                Err(e) => {
                                    match e {
                                        crossbeam_channel::TryRecvError::Empty => {},
                                        crossbeam_channel::TryRecvError::Disconnected => {
                                            panic!("No preceding pipeline stage found anymore!")
                                        }
                                    }
                                    
                                }
                            }
                        } else {
                            stage.instruction = Instruction(0x0);
                            stage.data_in = PipelineData(vec![]); 
                        };
    
                        let period_start = Instant::now();
                        let data_output = (stage.process_fn)(&stage.data_in, self);            
                        let elapsed_period = period_start.elapsed();
                        
                        barrier.wait(); //clock boundary
                        
                        //chech if a reset or a stall was asserted 
                        let reset = self.is_stage_reset(stage.index);
                        let enabled = self.is_stage_enabled(stage.index);
                        if reset {
                            // reset the output of the current pipeline stage
                            stage.data_out = PipelineData(vec![0u8; stage.size_out]);
                            stage.instruction = Instruction(0x0);
                        } else if enabled {
                            //update output of pipeline stage if no stall was asserted
                            stage.data_out = data_output;
                            if stage.index == 0x0 {
                                self.set_pc(self.get_pc() + 4);
                            }
                        } 

                        self.trace_asm_instr(&mut stage, true, true);

                        let period = elapsed_period.as_nanos();
                        tracing::info!("Stage {} delay time: {} ns", stage.name, period);
                        if clock_period.is_some() {
                            let clock_period = clock_period.unwrap();
                            if period < clock_period {
                                //complete remainder of clock period
                                sleep(std::time::Duration::from_nanos((clock_period - period) as u64));
                            } else {
                                // otherwise treat it as a warning
                                tracing::warn!(
                                    "Pipeline stage {} execution time is taking longer then the configured clock period by {} nanosecs.",
                                    stage.name, 
                                    period - clock_period
                                );
                            }
                        }

                        pipeline_payload = PipelinePayload {
                            instruction: stage.instruction,
                            data: stage.data_out.clone(),
                        };

                        if num_clock_cycles.is_some() && stage.clock_cycle == num_clock_cycles.unwrap() {
                            break;
                        }
                                
                        //send to next pipeline stage if available
                        match stage.output_channel {
                            Some(ref pipline_output) => match pipline_output.send(pipeline_payload) {
                                Ok(_) => {}
                                Err(e) => {
                                    tracing::info!("{e}");
                                    return;
                                }
                            },
                            None => {}
                        }
                        
                        stage.clock_cycle += 1;
                        if self.debug {
                            break;
                        }

                    }
                });
            }
        });
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
pub struct Registers([AtomicU64; 32]);

impl Registers {
    pub fn read_regs(&self, rs1_address: usize, rs2_address: usize) -> (RiscWord, RiscWord) {
        assert!(rs1_address < 32);
        assert!(rs2_address < 32);
        (
            self.0[rs1_address].load(std::sync::atomic::Ordering::SeqCst) as RiscWord, 
            self.0[rs2_address].load(std::sync::atomic::Ordering::SeqCst) as RiscWord
        )
    }

    pub fn write_reg(&self, rd_address: usize, rd: RiscWord) {
        assert!(rd_address < 32);
        if rd_address > 0 {
            //should never overwrite x0
            self.0[rd_address].store(rd as u64, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

use std::fmt::Display;
impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.0.len() {
            write!(f, "x{i}={:X}\n", (self.0[i].load(std::sync::atomic::Ordering::SeqCst) as RiscWord).cast_signed())?;
        }
        Ok(())
    }    
}
