use crossbeam_channel::bounded;
use crate::{risc_soc::{cache::Cache, memory_management_unit::{Address, MemoryDevice, MemoryDeviceType}, pipeline_stage::{PipelineStage, PipelineStageInterface}, risc_soc::RiscCore}, rv32i_baremetal::{commit, decode, execute, fetch, mcu_cache::MCUCache, uart::UART}};


pub fn init_core() -> RiscCore {
    let mut rv32i_core = RiscCore::new(4, 1000); //1us clock period
    let start_address = 0x8000_0000;
    let icache = MCUCache::new_with_lines(MemoryDeviceType::L1ICACHE, 64, 1024, start_address);
    let dcache = MCUCache::new_with_lines(MemoryDeviceType::L1DCACHE, 64, 1024, start_address + icache.size() as Address); 
    rv32i_core.add_l1_cache(Box::new(icache), Box::new(dcache));
    
    // add stages and connections between them
    let (if_id_sender, if_id_receiver) = bounded(0);
    let (id_ex_sender, id_ex_receiver) = bounded(0);
    let (ex_wb_sender, ex_wb_receiver) = bounded(0);
    let if_stage = PipelineStage::new("IF".to_string(), fetch::rv32_mcu_fetch_stage, None, Some(if_id_sender));
    let id_stage = PipelineStage::new("ID".to_string(), decode::rv32_mcu_decode_stage, Some(if_id_receiver), Some(id_ex_sender));
    let ex_stage= PipelineStage::new("EX".to_string(), execute::rv32_mcu_execute_stage, Some(id_ex_receiver), Some(ex_wb_sender));
    let wb_stage= PipelineStage::new("WB".to_string(), commit::rv32_mcu_commit_stage, Some(ex_wb_receiver), None);
    rv32i_core.add_stage(if_stage);
    rv32i_core.add_stage(id_stage);
    rv32i_core.add_stage(ex_stage);
    rv32i_core.add_stage(wb_stage);
    tracing::info!("Configured RV32I core with {} stages", rv32i_core.stages.len());

    { 
        let mut  mmu= rv32i_core.mmu.write().unwrap();
        let uart_device = UART::new(MemoryDeviceType::UART0, 0x4060_0000, 0x4060_0100);
        mmu.add_memory_device(Box::new(uart_device));
    }
    
    rv32i_core
}

pub fn load_elf(core: &mut RiscCore, path: &str) {
    core.load_binary(path, MemoryDeviceType::L1ICACHE);
}