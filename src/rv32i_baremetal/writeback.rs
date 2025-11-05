use crate::risc_soc::risc_soc::{RiscCore};
use crate::risc_soc::{pipeline_stage::PipelineData};
use crate::rv32i_baremetal::core::{ID_STAGE, EX_STAGE, WB_STAGE};

pub fn rv32_mcu_commit_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {
    let reg_write = pipeline_reg.get_u8(0x0);
    let reg_src = pipeline_reg.get_u8(0x1);
    let rd_address = pipeline_reg.get_u8(0x2);
    let alu_out = pipeline_reg.get_u32(0x3);
    let mem_out = pipeline_reg.get_u32(0x7);

    let rd_value;
    if reg_src == 0x1 {
        rd_value = mem_out;
    } else {
        rd_value = alu_out;
    }

    // send commit info to ID and EX stages
    let mut pipe = vec![];
    pipe.push(reg_write);
    pipe.push(rd_address);
    pipe.extend_from_slice(&rd_value.to_le_bytes());
    let wb_data = PipelineData(pipe);
    rv32_core.cdb.assign(WB_STAGE, ID_STAGE, wb_data.clone());
    rv32_core.cdb.assign(WB_STAGE, EX_STAGE, wb_data.clone());

    PipelineData(vec![])
}
