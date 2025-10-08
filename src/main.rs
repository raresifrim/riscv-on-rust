mod rv32;
use rv32::*;
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    fmt::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_writer(std::io::stderr)
        .compact()
        .init();

    tracing::info!("Initializing RISCV32 runtime environment");
}
