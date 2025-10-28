use crossbeam_channel::bounded;
use crate::{risc_soc::{cache::Cache, memory_management_unit::{Address, MemoryDevice, MemoryDeviceType}, pipeline_stage::{PipelineStage, PipelineStageInterface}, risc_soc::RiscCore}, rv32i_baremetal::{commit, decode, execute, fetch, mcu_cache::MCUCache, uart::UART}};

pub const IF_STAGE: usize = 0x0;
pub const ID_STAGE: usize = 0x1;
pub const EX_STAGE: usize = 0x2;
pub const WB_STAGE: usize = 0x3;

pub fn init_core(clock_period: Option<u128>) -> RiscCore {
    let mut rv32i_core = RiscCore::new(4, clock_period, false); //1us clock period
    let start_address = 0x8000_0000;
    let icache = MCUCache::new_with_lines(MemoryDeviceType::L1ICACHE, 64, 1024, start_address);
    let dcache = MCUCache::new_with_lines(MemoryDeviceType::L1DCACHE, 64, 1024, start_address + icache.size() as Address); 
    rv32i_core.add_l1_cache(Box::new(icache), Box::new(dcache));
    
    // add stages and connections between them
    // bound channels to one entry to mimic the behaviour of a single pipeline reg
    let (if_id_sender, if_id_receiver) = bounded(1);
    let (id_ex_sender, id_ex_receiver) = bounded(1);
    let (ex_wb_sender, ex_wb_receiver) = bounded(1);
    let if_stage = PipelineStage::new("IF".to_string(), IF_STAGE, 0usize, fetch::rv32_mcu_fetch_stage, None, Some(if_id_sender));
    let id_stage = PipelineStage::new("ID".to_string(), ID_STAGE,  8usize, decode::rv32_mcu_decode_stage, Some(if_id_receiver), Some(id_ex_sender));
    let ex_stage= PipelineStage::new("EX".to_string(), EX_STAGE,  25usize, execute::rv32_mcu_execute_stage, Some(id_ex_receiver), Some(ex_wb_sender));
    let wb_stage= PipelineStage::new("WB".to_string(), WB_STAGE,  12usize, commit::rv32_mcu_commit_stage, Some(ex_wb_receiver), None);
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


#[cfg(test)]
mod tests {
    
     #[test]
    fn test_add() {
        let mut rv32i_core = super::init_core(None);
        rv32i_core.enable_debug(true);
        super::load_elf(&mut rv32i_core, "./isa_tests/add.elf");
        for _i in 0..11{
            rv32i_core.run();
        }
        println!("{}", rv32i_core.registers);
    }

    #[test]
    fn test_branch() {
        let mut rv32i_core = super::init_core(None);
        rv32i_core.enable_debug(true);
        super::load_elf(&mut rv32i_core, "./isa_tests/branch.elf");
        for _i in 0..15{
            rv32i_core.run();
        }
        println!("{}", rv32i_core.registers);
    }

    #[test]
    fn test_jump() {
        let mut rv32i_core = super::init_core(None);
        rv32i_core.enable_debug(true);
        super::load_elf(&mut rv32i_core, "./isa_tests/jump_and_return.elf");
        for _i in 0..15{
            rv32i_core.run();
        }
        println!("{}", rv32i_core.registers);
    }

    #[test]
    fn test_memory() {
        let mut rv32i_core = super::init_core(None);
        rv32i_core.enable_debug(true);
        super::load_elf(&mut rv32i_core, "./isa_tests/memory.elf");
        for _i in 0..48{
            rv32i_core.run();
        }
        rv32i_core.dcache.unwrap().read().unwrap().debug(0x8001_0000, 0x8001_0010).unwrap();
    }
}