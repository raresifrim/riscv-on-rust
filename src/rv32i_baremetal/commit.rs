use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::{pipeline_stage::PipelineData, risc_soc::RiscWord};

pub fn rv32_mcu_commit_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {

    PipelineData(vec![]) //nothing to send here
}