use super::instruction_asm::rv32_asm;
use crate::risc_soc::risc_soc::RiscCore;
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;

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
            value |= (self.0[address + i] as u16) << i;
        }
        value
    }

    pub fn get_u32(&self, address: usize) -> u32 {
        assert!(address + 4 <= self.0.len());
        let mut value: u32 = 0x0;
        for i in 0..4 {
            value |= (self.0[address + i] as u32) << i;
        }
        value
    }

    pub fn get_u64(&self, address: usize) -> u64 {
        assert!(address + 8 <= self.0.len());
        let mut value: u64 = 0x0;
        for i in 0..8 {
            value |= (self.0[address + i] as u64) << i;
        }
        value
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Instruction(u32);

pub type ClockCycle = u64;

#[derive(Debug, Default)]
pub struct PipelinePayload {
    clock_cycle: ClockCycle,
    instruction: Instruction,
    data: PipelineData,
}

#[derive(Debug)]
pub struct PipelineStage {
    /// name identifier for the pipeline stage
    name: String,
    /// current clock_cycle and instruction in this stage
    instruction: Instruction,
    clock_cycle: ClockCycle,
    /// current data it processes
    data: PipelineData,
    /// input and output to previous and next stages
    /// stages like fetch and commit may not have any previous or next stage
    /// so we define the input and output channels as optional
    input_channel: Option<Receiver<PipelinePayload>>,
    output_channel: Option<Sender<PipelinePayload>>,
    /// function that runs inside the pipeline stage
    process_fn: fn(&PipelineData, &Arc<RiscCore>) -> PipelineData,
    /// reference to entire core, as pipeline stages can directly access data between them (ex. hazard detection or forwarding)
    core: Arc<RiscCore>,
}

pub trait PipelineStageInterface {
    type F: Fn(&PipelineData, &Arc<RiscCore>) -> PipelineData + Send + 'static;

    fn new(
        name: String,
        process_fn: Self::F,
        core: Arc<RiscCore>,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self;

    /// spawn thread that will run this stage
    /// you must pass a function that will process the input data into this stage
    /// and produce new data for the putput of this stage
    fn run(self) -> std::thread::JoinHandle<()>;

    /// extract data from current moment
    fn extract_data(&self) -> &PipelineData;

    /// check the current instruction and clock cycle of this pipeline stage
    fn get_current_step(&self) -> (ClockCycle, Instruction);
}

impl PipelineStageInterface for PipelineStage {
    type F = fn(&PipelineData, &Arc<RiscCore>) -> PipelineData;

    fn new(
        name: String,
        process_fn: Self::F,
        core: Arc<RiscCore>,
        input_channel: Option<Receiver<PipelinePayload>>,
        output_channel: Option<Sender<PipelinePayload>>,
    ) -> Self {
        Self {
            name,
            process_fn,
            core,
            instruction: Instruction(0x0),
            clock_cycle: 0,
            input_channel,
            output_channel,
            data: PipelineData(vec![]),
        }
    }

    fn run(mut self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            loop {
                let pipeline_payload;
                // read from previous pipeline stage if available
                match self.input_channel {
                    Some(ref pipeline_input) => match pipeline_input.recv() {
                        Ok(data_input) => {
                            self.instruction = data_input.instruction;
                            self.data = data_input.data;
                            self.clock_cycle = data_input.clock_cycle;

                            let asm_instr = rv32_asm(self.instruction.0);
                            tracing::info!(
                                "Pipeline Stage {} @ClockCycle {} -> Instruction:{}",
                                self.name,
                                self.clock_cycle,
                                asm_instr
                            );

                            let data_output = (self.process_fn)(&self.data, &self.core);

                            pipeline_payload = PipelinePayload {
                                instruction: self.instruction,
                                clock_cycle: data_input.clock_cycle + 1,
                                data: data_output,
                            };
                        }
                        Err(e) => {
                            tracing::info!("{e}");
                            return;
                        }
                    },

                    None => {
                        // if there is no input channel, this would mean we are in the fetch stage
                        let data_output = (self.process_fn)(&self.data, &self.core);
                        self.instruction = Instruction(data_output.get_u32(0x0));

                        let asm_instr = rv32_asm(self.instruction.0);
                        tracing::info!(
                            "Pipeline Stage {} @ClockCycle {} -> Instruction:{}",
                            self.name,
                            self.clock_cycle,
                            asm_instr
                        );

                        self.clock_cycle = self.clock_cycle + 1;
                        pipeline_payload = PipelinePayload {
                            instruction: self.instruction,
                            clock_cycle: self.clock_cycle,
                            data: data_output,
                        };
                    }
                };

                //send to next pipeline stage if available
                match self.output_channel {
                    Some(ref pipline_output) => match pipline_output.send(pipeline_payload) {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::info!("{e}");
                            return;
                        }
                    },
                    None => {}
                }
            }
        })
    }

    fn extract_data(&self) -> &PipelineData {
        &self.data
    }

    fn get_current_step(&self) -> (ClockCycle, Instruction) {
        (self.clock_cycle, self.instruction)
    }
}
