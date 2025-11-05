use crate::risc_soc::risc_soc::RiscCore;
use crossbeam_channel::{Receiver, Sender};


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

    pub fn push_bytes(&mut self, mut data: Vec<u8>) {
        self.0.append(&mut data);
    }

    pub fn size(&self) -> usize {
        self.0.len()
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
    pub size_in: usize,
    pub size_out: usize,
    /// current clock_cycle and instruction in this stage
    pub instruction: Instruction,
    pub clock_cycle: ClockCycle,
    /// current data it consumed and produced during a clock cycle
    pub data_in: PipelineData,
    pub data_out: PipelineData,
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
        size_in: usize,
        size_out: usize,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self;

    /// extract data from current moment
    fn extract_data(&self) -> (&PipelineData,&PipelineData);

    /// check the current instruction and clock cycle of this pipeline stage
    fn get_current_step(&self) -> (ClockCycle, Instruction);

    fn enable_debug(&mut self, debug: bool);
}

impl PipelineStageInterface for PipelineStage {
    type F = fn(&PipelineData, &RiscCore) -> PipelineData;

    fn new(
        name: String,
        index: usize,
        size_in: usize,
        size_out: usize,
        process_fn: Self::F,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self {
        Self {
            name,
            index,
            size_in,
            size_out,
            process_fn,
            debug: false,
            instruction: Instruction(0x0),
            clock_cycle: 0,
            input_channel,
            output_channel,
            data_in: PipelineData(vec![0u8; size_in]),
            data_out: PipelineData(vec![0u8; size_out]),
        }
    }

    fn extract_data(&self) -> (&PipelineData,&PipelineData) {
        (&self.data_in, &self.data_out)
    }

    fn get_current_step(&self) -> (ClockCycle, Instruction) {
        (self.clock_cycle, self.instruction)
    }

    fn enable_debug(&mut self, debug: bool) {
        self.debug = debug
    }
}

