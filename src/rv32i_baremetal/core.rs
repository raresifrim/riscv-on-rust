use crossbeam_channel::bounded;

use crate::{risc_soc::{cache::Cache, memory_management_unit::{Address, MemoryDevice, MemoryDeviceType}, pipeline_stage::{PipelineStage, PipelineStageInterface}, risc_soc::RiscCore}, rv32i_baremetal::{decode, fetch, mcu_cache::MCUCache}};
use std::sync::{Arc,RwLock};

pub fn init_core() -> RiscCore {
    let mut rv32i_core = RiscCore::new(4, 1000); //1us clock period
    let start_address = 0x8000_0000;
    let icache = MCUCache::new_with_lines(MemoryDeviceType::L1ICACHE, 64, 1024, start_address);
    let dcache = MCUCache::new_with_lines(MemoryDeviceType::L1DCACHE, 64, 1024, start_address + icache.size() as Address); 
    rv32i_core.add_l1_cache(Box::new(icache), Box::new(dcache));
    
    // add stages and connections between them
    let (if_id_sender, if_id_receiver) = bounded(0);
    let if_stage = PipelineStage::new("IF".to_string(), fetch::rv32_mcu_fetch_stage, None, Some(if_id_sender));
    let id_stage = PipelineStage::new("ID".to_string(), decode::rv32_mcu_decode_stage, Some(if_id_receiver), None);
    rv32i_core.add_stage(if_stage);
    rv32i_core.add_stage(id_stage);
    
    rv32i_core
}

pub fn load_elf(core: &mut RiscCore, path: &str) {
    core.load_binary(path, MemoryDeviceType::L1ICACHE);
}