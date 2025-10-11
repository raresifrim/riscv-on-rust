use super::{pipeline_stage::PipelineData};
use crate::risc_soc::risc_soc::RiscCore;
use std::sync::{Arc, RwLock};
use std::thread::{sleep};
use crate::risc_soc::memory_management_unit::{Address, MemoryRequest, MemoryRequestType};
use crate::risc_soc::risc_soc::WordSize;

pub fn rv32_mcu_fetch_stage(pipeline_reg: &PipelineData, rv32_core: &Arc<RwLock<RiscCore>>) -> PipelineData {
    
    //as the fetch stage is the one that dictates th entire flow
    //we emulate a system clock here with a period of 1ms 
    sleep(std::time::Duration::from_millis(1));

    let core = rv32_core.read().unwrap();
    // get current PC and update next one
    let current_pc = core.get_pc();     
    core.set_pc(current_pc + 4);

    //get instruction from the current address
    let request = MemoryRequest{request_type: MemoryRequestType::READ, data_address: current_pc as Address, data_size: WordSize::WORD, data:None};
    let response = core.icache_read(request);
    let mut instruction = response.data;
    instruction.extend_from_slice(&current_pc.to_le_bytes());

    PipelineData(instruction)
}