mod risc_soc;
mod rv32i_baremetal;
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    fmt::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_writer(std::io::stderr)
        .compact()
        .init();

    tracing::info!("Initializing RISCV32 runtime environment");
    let mut rv32i_core = rv32i_baremetal::core::init_core(None);
    //rv32i_baremetal::core::load_elf(&mut rv32i_core, "./qemu_playground/test_microblaze.elf");
    rv32i_baremetal::core::load_elf(&mut rv32i_core, "./isa_tests/memory.elf");
    rv32i_core.run();
}
