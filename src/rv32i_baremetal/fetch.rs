use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::pipeline_stage::PipelineStageInterface;
use crate::risc_soc::memory_management_unit::{Address, MemoryRequest, MemoryRequestType};
use crate::risc_soc::risc_soc::WordSize;
use crate::risc_soc::pipeline_stage::PipelineData;

pub fn rv32_mcu_fetch_stage(_pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {

    // get current PC and update next one
    let mut current_pc = rv32_core.get_pc();

    let commit_stage = rv32_core.stages[3].read().unwrap();
    let commit_data = commit_stage.extract_data();
    if !commit_data.0.is_empty() {
        let commit_branch_or_jump = commit_data.get_u8(0x3); 
        let commit_take_jump = commit_data.get_u8(0x4);
        let commit_pc = commit_data.get_u32(0x9);

        if commit_branch_or_jump & commit_take_jump == 0x1 {
           current_pc = commit_pc;
        }
    }
    
    rv32_core.set_pc(current_pc + 4); //set next pc

    //get instruction from the current address
    let request = MemoryRequest{request_type: MemoryRequestType::READ, data_address: current_pc as Address, data_size: WordSize::WORD, data:None};
    let response = rv32_core.icache_request(request);
    let mut instruction = response.data;
    instruction.extend_from_slice(&current_pc.to_le_bytes());

    PipelineData(instruction)
}