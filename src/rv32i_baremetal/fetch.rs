use crate::risc_soc::memory_management_unit::{Address, MemoryRequest, MemoryRequestType};
use crate::risc_soc::pipeline_stage::PipelineData;
use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::risc_soc::WordSize;
use crate::rv32i_baremetal::core::{IF_STAGE, MEM_STAGE};

pub fn rv32_mcu_fetch_stage(_pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {
    // get current PC and update next one only if we are not asserted to stall
    let mut current_pc = rv32_core.get_pc();

    // Comb logic coming from MEM stage
    let mem_data = rv32_core.cdb.pull(MEM_STAGE, IF_STAGE);
    let branch_or_jump = mem_data.get_u8(0x0);
    let take_jump = mem_data.get_u8(0x1);
    let pc = mem_data.get_u32(0x2);
    if branch_or_jump & take_jump == 0x1 {
        println!("branch taken");
        current_pc = pc;
        rv32_core.set_pc(current_pc);
    }

    //get instruction from the current address
    let request = MemoryRequest {
        request_type: MemoryRequestType::READ,
        data_address: current_pc as Address,
        data_size: WordSize::WORD,
        data: None,
    };
    let response = rv32_core.icache_request(request);
    let mut instruction = response.data;
    instruction.extend_from_slice(&current_pc.to_le_bytes());

    return PipelineData(instruction);
}
