use crate::risc_soc::risc_soc::RiscCore;
use std::sync::{Arc, RwLock};
use crate::risc_soc::memory_management_unit::{Address, MemoryRequest, MemoryRequestType};
use crate::risc_soc::risc_soc::WordSize;
use crate::risc_soc::pipeline_stage::PipelineData;

pub fn rv32_mcu_fetch_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {

    // get current PC and update next one
    let current_pc = rv32_core.get_pc(); 
    tracing::info!("Current PC: {:X}", current_pc);    
    rv32_core.set_pc(current_pc + 4);

    //get instruction from the current address
    let request = MemoryRequest{request_type: MemoryRequestType::READ, data_address: current_pc as Address, data_size: WordSize::WORD, data:None};
    let response = rv32_core.icache_request(request);
    let mut instruction = response.data;
    instruction.extend_from_slice(&current_pc.to_le_bytes());

    PipelineData(instruction)
}