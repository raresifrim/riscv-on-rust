use crate::risc_soc::risc_soc::{RiscCore, WordSize};
use crate::risc_soc::{pipeline_stage::PipelineData, risc_soc::RiscWord};
use crate::rv32i_baremetal::core::WB_STAGE;
use crate::risc_soc::memory_management_unit::{Address, MemoryRequest};
use crate::risc_soc::memory_management_unit::MemoryRequestType;

pub fn rv32_mcu_commit_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {

    let reg_write = pipeline_reg.get_u8(0x0);
    let mem_read_write = pipeline_reg.get_u8(0x1);
    let rd_address = pipeline_reg.get_u8(0x2);
    let func3 = pipeline_reg.get_u8(0x3);
    let alu_out = pipeline_reg.get_u32(0x4);
    let rs2 = pipeline_reg.get_u32(0x8);

    let mut rd_value = alu_out;
    if mem_read_write == 0x1 { //load
        let data_size  = match func3 {
            0x0 | 0x4 => WordSize::BYTE,
            0x1 | 0x5 => WordSize::HALF,
            _ => WordSize::WORD
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

        rd_value = match func3 {
            0x0 => { data[0].cast_signed() as i32 as RiscWord },
            0x4 => { data[0] as RiscWord },
            0x1 => { (((data[1] as u16) << 8) | (data[0] as u16)) as i32 as RiscWord}
            0x5 => { (((data[1] as u16) << 8) | (data[0] as u16)) as RiscWord },
            _   => { ((data[3] as u32) << 24) | ((data[2] as u32) << 16) | ((data[1] as u32) << 8) | (data[0] as u32) as RiscWord }
        }
    } else if mem_read_write == 0x3 { //store
        let (data_size, data)  = match func3 {
            0x0 => { (WordSize::BYTE, rs2 & 0xFF)},
            0x1 => { (WordSize::HALF, rs2 & 0xFFFF)},
            _ =>   { (WordSize::WORD, rs2) }
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

    {
        // send commit info to ID and EX stages
        let mut pipe = vec![];
        pipe.push(reg_write);
        pipe.push(rd_address);
        pipe.extend_from_slice(&rd_value.to_le_bytes());
        let wb_data = PipelineData(pipe);
        let wb_wire_data = &rv32_core.cdb[WB_STAGE];
        wb_wire_data.assign(wb_data);
        println!("WB: data forwarded to CDB");
    }

    pipeline_reg.clone()
}
