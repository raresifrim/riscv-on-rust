use super::instruction_asm::rv32_asm;
use crate::risc_soc::risc_soc::RiscCore;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Instant;

#[derive(Debug, Default)]
pub struct PipelineData(pub Vec<u8>);

impl PipelineData {
    pub fn get_u8(&self, address: usize) -> u8 {
        assert!(address < self.0.len());
        let value: u8 = self.0[address];
        value
    }

    pub fn get_u16(&self, address: usize) -> u16 {
        assert!(address + 2 <= self.0.len());
        let mut value: u16 = 0x0;
        for i in 0..2 {
            value |= (self.0[address + i] as u16) << i*8;
        }
        value
    }

    pub fn get_u32(&self, address: usize) -> u32 {
        assert!(address + 4 <= self.0.len());
        let mut value: u32 = 0x0;
        for i in 0..4 {
            value |= (self.0[address + i] as u32) << i*8;
        }
        value
    }

    pub fn get_u64(&self, address: usize) -> u64 {
        assert!(address + 8 <= self.0.len());
        let mut value: u64 = 0x0;
        for i in 0..8 {
            value |= (self.0[address + i] as u64) << i*8;
        }
        value
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Instruction(pub u32);

pub type ClockCycle = u64;

#[derive(Debug, Default)]
pub struct PipelinePayload {
    pub clock_cycle: ClockCycle,
    pub instruction: Instruction,
    pub data: PipelineData,
}

pub struct PipelineStage {
    /// name identifier for the pipeline stage
    pub name: String,
    /// current clock_cycle and instruction in this stage
    pub instruction: Instruction,
    pub clock_cycle: ClockCycle,
    /// current data it processes
    pub data: PipelineData,
    /// input and output to previous and next stages
    /// stages like fetch and commit may not have any previous or next stage
    /// so we define the input and output channels as optional
    pub input_channel: Option<Receiver<PipelinePayload>>,
    pub output_channel: Option<Sender<PipelinePayload>>,
    /// function that runs inside the pipeline stage
    pub process_fn: fn(&PipelineData, &RiscCore) -> PipelineData,
}

pub trait PipelineStageInterface {
    type F: Fn(&PipelineData, &RiscCore) -> PipelineData + Send + 'static;

    fn new(
        name: String,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self;

    /// spawn thread that will run this stage
    /// you must pass a function that will process the input data into this stage
    /// and produce new data for the putput of this stage
    //fn run(self, core: Arc<RwLock<RiscCore>>) -> std::thread::JoinHandle<()>;

    /// extract data from current moment
    fn extract_data(&self) -> &PipelineData;

    /// check the current instruction and clock cycle of this pipeline stage
    fn get_current_step(&self) -> (ClockCycle, Instruction);
}

impl PipelineStageInterface for PipelineStage {
    type F = fn(&PipelineData, &RiscCore) -> PipelineData;

    fn new(
        name: String,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self {
        Self {
            name,
            process_fn,
            instruction: Instruction(0x0),
            clock_cycle: 0,
            input_channel,
            output_channel,
            data: PipelineData(vec![]),
        }
    }

    fn extract_data(&self) -> &PipelineData {
        &self.data
    }

    fn get_current_step(&self) -> (ClockCycle, Instruction) {
        (self.clock_cycle, self.instruction)
    }
}
