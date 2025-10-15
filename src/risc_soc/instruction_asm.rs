use instruction_decoder::Decoder;

pub fn rv32_asm(instr_bin: u32) -> String {
    match Decoder::new(&[include_str!("../../instruction-decoder/toml/RV32I.toml").to_string()]) {
        Ok(test_decoder) => {
            if let Ok(iform) = test_decoder.decode_from_u32(instr_bin, 32) {
               iform 
            } else {
                panic!("Could not decode {:X} into asm!", instr_bin);
            }
        }
        Err(error_stacks) => {
            println!("Errors in ../toml/RV32I.toml:");
            for error in &error_stacks[0] {
                println!("\t{error}");
            }
            panic!("Could not decode {:X} into asm!", instr_bin);
        }
    }
}