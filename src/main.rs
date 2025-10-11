mod risc_soc;
mod rv32_baremetal;
use tracing_subscriber::{EnvFilter, fmt};

use crate::risc_soc::risc_soc::RiscCore;

fn main() {
    fmt::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_writer(std::io::stderr)
        .compact()
        .init();

    tracing::info!("Initializing RISCV32 runtime environment");

    let rv32i_core = RiscCore::new(4);
    rv32i_core.load_binary("./qemu_playground/test_microblaze.elf".to_string());
}
