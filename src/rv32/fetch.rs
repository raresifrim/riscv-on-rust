use crate::{pipeline_stage::PipelineData, rv32::cache::CacheDataRequest};
use crate::rv32::rv32::RV32Core;
use crate::cache::MemoryRequestType;
use std::sync::{Arc, RwLock};
use std::thread::{self, sleep};
use crate::rv32::rv32::WordSize;

pub fn rv32_fetch_stage(pipeline_reg: &PipelineData, rv32_core: &Arc<RwLock<RV32Core>>) -> PipelineData {
    
    //as the fetch stage is the one that dictates th entire flow
    //we emulate a system clock here with a period of 1ms 
    sleep(std::time::Duration::from_millis(1));

    let core = rv32_core.read().unwrap();
    // get current PC and update next one
    let current_pc = core.get_pc();     
    core.set_pc(current_pc + 4);

    //get instruction from the current address
    let request = CacheDataRequest{request_type: MemoryRequestType::READ, data_address: current_pc as usize, data_size: WordSize::WORD, data:None};
    let response = core.icache_request(request);
    let mut instruction = response.data;
    instruction.extend_from_slice(&current_pc.to_le_bytes());

    PipelineData(instruction)
}