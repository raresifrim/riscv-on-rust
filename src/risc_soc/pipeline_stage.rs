use crate::risc_soc::risc_soc::RiscCore;
use crossbeam_channel::{Receiver, Sender};
use std::{ops::{Deref, DerefMut}, sync::Barrier};


#[derive(Debug, Clone)]
pub struct PipelineData(pub Vec<u8>);

impl Default for PipelineData{
    fn default() -> Self {
        Self(vec![])
    }
}

impl PipelineData {

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

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
    pub instruction: Instruction,
    pub data: PipelineData,
}

pub struct PipelineStage {
    /// name identifier and index for the pipeline stage
    pub name: String,
    pub index: usize,
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
    /// function that runs inside the pipeline stage and produces data for the next stage
    pub process_fn: fn(&PipelineData, &RiscCore) -> PipelineData,
    pub debug: bool,
}

pub trait PipelineStageInterface {
    type F: Fn(&PipelineData, &RiscCore) -> PipelineData + Send + 'static;

    fn new(
        name: String,
        index: usize,
        size: usize,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self;

    fn execute_once(&self, data: &PipelineData, core: &RiscCore, barrier: &Barrier) -> PipelineData;

    /// extract data from current moment
    fn extract_data(&self) -> &PipelineData;

    /// check the current instruction and clock cycle of this pipeline stage
    fn get_current_step(&self) -> (ClockCycle, Instruction);

    fn enable_debug(&mut self, debug: bool);
}

impl PipelineStageInterface for PipelineStage {
    type F = fn(&PipelineData, &RiscCore) -> PipelineData;

    fn new(
        name: String,
        index: usize,
        size: usize,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self {
        Self {
            name,
            index,
            process_fn,
            debug: false,
            instruction: Instruction(0x0),
            clock_cycle: 0,
            input_channel,
            output_channel,
            data: PipelineData(vec![0u8; size]),
        }
    }

    fn extract_data(&self) -> &PipelineData {
        &self.data
    }

    fn get_current_step(&self) -> (ClockCycle, Instruction) {
        (self.clock_cycle, self.instruction)
    }

    fn execute_once(&self, data: &PipelineData, core: &RiscCore, barrier: &Barrier) -> PipelineData {
        barrier.wait(); //clock boundary
        let pipe_reg = (self.process_fn)(data, core); //actual pipeline 
        barrier.wait(); //clock boundary
        return  pipe_reg;
    }

    fn enable_debug(&mut self, debug: bool) {
        self.debug = debug
    }
}

impl Deref for PipelineStage {
    type Target = PipelineData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for PipelineStage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}