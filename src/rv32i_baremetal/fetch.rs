use crate::risc_soc::memory_management_unit::{Address, MemoryRequest, MemoryRequestType};
use crate::risc_soc::pipeline_stage::PipelineData;
use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::risc_soc::WordSize;
use crate::rv32i_baremetal::core::EX_STAGE;

pub fn rv32_mcu_fetch_stage(_pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {
    // get current PC and update next one
    let mut current_pc = rv32_core.get_pc();

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

    // Comb logic coming from EX stage
    let ex_wire_data = &rv32_core.cdb[EX_STAGE];
    let ex_data = ex_wire_data.read();

    if !ex_data.is_empty() {
        let ex_branch_or_jump = ex_data.get_u8(0x0);
        let ex_take_jump = ex_data.get_u8(0x1);
        let ex_pc = ex_data.get_u32(0x2);

        if ex_branch_or_jump & ex_take_jump == 0x1 {
            current_pc = ex_pc;

            //get instruction from the new address
            let request = MemoryRequest {
                request_type: MemoryRequestType::READ,
                data_address: current_pc as Address,
                data_size: WordSize::WORD,
                data: None,
            };
            let response = rv32_core.icache_request(request);
            instruction = response.data; //overwrite instr with new value
            instruction.extend_from_slice(&current_pc.to_le_bytes());
        }
    }

    rv32_core.set_pc(current_pc + 4); //set next pc

    PipelineData(instruction)
}
