use crate::risc_soc::memory_management_unit::MemoryRequestType;
use crate::risc_soc::memory_management_unit::{Address, MemoryRequest};
use crate::risc_soc::risc_soc::{RiscCore, WordSize};
use crate::risc_soc::{pipeline_stage::PipelineData, risc_soc::RiscWord};
use crate::rv32i_baremetal::core::{EX_STAGE, IF_STAGE, MEM_STAGE, ID_STAGE};

pub fn rv32_mcu_mem_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {
    
    let reg_write = pipeline_reg.get_u8(0x0);
    let mem_read_write = pipeline_reg.get_u8(0x1);
    let rd_address = pipeline_reg.get_u8(0x2);
    let func3 = pipeline_reg.get_u8(0x3);
    let alu_out = pipeline_reg.get_u32(0x4);
    let rs2 = pipeline_reg.get_u32(0x8);
    let branch_or_jump = pipeline_reg.get_u8(0xC);
    let take_jump = pipeline_reg.get_u8(0xD);
    let pc = pipeline_reg.get_u32(0xE);

    //send info about branch to IF and ID
    let mut if_data = vec![];
    if_data.push(branch_or_jump);
    if_data.push(take_jump);
    if_data.extend_from_slice(&pc.to_le_bytes());
    let wire_data = PipelineData(if_data);
    rv32_core.cdb.assign(MEM_STAGE, IF_STAGE, wire_data.clone());
    rv32_core.cdb.assign(MEM_STAGE, ID_STAGE, wire_data);

    // send MEM info to EX stage for forwarding
    let mut ex_data = vec![];
    ex_data.push(reg_write);
    ex_data.push(rd_address);
    ex_data.extend_from_slice(&alu_out.to_le_bytes());
    let ex_data = PipelineData(ex_data);
    rv32_core.cdb.assign(MEM_STAGE, EX_STAGE, ex_data);

    let mut mem_value = 0x0;
    let mut reg_src = 0x0;
    if mem_read_write == 0x1 {
        //load
        let data_size = match func3 {
            0x0 | 0x4 => WordSize::BYTE,
            0x1 | 0x5 => WordSize::HALF,
            _ => WordSize::WORD,
        };
        
        //get instruction from the current address
        let request = MemoryRequest {
            request_type: MemoryRequestType::READ,
            data_address: alu_out as Address,
            data_size,
            data: None,
        };
        
        let response = rv32_core.dcache_request(request);
        let data = response.data;
        assert!(data.len() == data_size as usize);

        mem_value = match func3 {
            0x0 => data[0].cast_signed() as i32 as RiscWord,
            0x4 => data[0] as RiscWord,
            0x1 => (((data[1] as u16) << 8) | (data[0] as u16)) as i32 as RiscWord,
            0x5 => (((data[1] as u16) << 8) | (data[0] as u16)) as RiscWord,
            _ => {
                ((data[3] as u32) << 24)
                    | ((data[2] as u32) << 16)
                    | ((data[1] as u32) << 8)
                    | (data[0] as u32) as RiscWord
            }
        };
        reg_src = 0x1;
    } else if mem_read_write == 0x3 {
        //store
        let (data_size, data) = match func3 {
            0x0 => (WordSize::BYTE, rs2 & 0xFF),
            0x1 => (WordSize::HALF, rs2 & 0xFFFF),
            _ => (WordSize::WORD, rs2),
        };
        
        //get instruction from the current address
        let request = MemoryRequest {
            request_type: MemoryRequestType::WRITE,
            data_address: alu_out as Address,
            data_size,
            data: Some(data.to_le_bytes().to_vec()),
        };

        rv32_core.dcache_request(request);
    }

    let mut pipeline_out = vec![];
    pipeline_out.push(reg_write);
    pipeline_out.push(reg_src);
    pipeline_out.push(rd_address);
    pipeline_out.extend_from_slice(&alu_out.to_le_bytes());
    pipeline_out.extend_from_slice(&mem_value.to_le_bytes());

    PipelineData(pipeline_out)
}
