use crate::risc_soc::pipeline_stage::PipelineData;
use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::pipeline_stage::PipelineStageInterface;
use std::u32;

/// FUNC7 and FUNCT3 field lengths
pub const FUNCT_7L: u8 = 7;
pub const FUNCT_3L: u8 = 3;
pub const FUNCT_7_MASK: u32 = 0b1111111;
pub const FUNCT_3_MASK: u32 = 0b111;

/// OPCODE field lendth
pub const OPCODE_L: u8 = 7;
pub const OPCODE_MASK: u32 = 0b1111111;

/// Register Index size
pub const REG_L: u8 = 5;
pub const REG_MASK: u32 = 0b11111;

// RISC-V Base Instruction Set Opcodes
pub const OP_LUI    :u8 = 0b0110111; // Load Upper Immediate
pub const OP_AUIPC  :u8 = 0b0010111; // Add Upper Immediate to PC
pub const OP_JAL    :u8 = 0b1101111; // Jump and Link
pub const OP_JALR   :u8 = 0b1100111; // Jump and Link Register
pub const OP_BRANCH :u8 = 0b1100011; // Branch Instructions (BEQ, BNE, BLT, etc.)
pub const OP_LOAD   :u8 = 0b0000011; // Load Instructions (LB, LH, LW, LBU, LHU)
pub const OP_STORE  :u8 = 0b0100011; // Store Instructions (SB, SH, SW)
pub const OP_ALU    :u8 = 0b0110011;// ALU Instructions (ADD, SUB, AND, OR, XOR, etc.)
pub const OP_ALUI   :u8 = 0b0010011; // ALU Immediate Instructions (ADDI, ANDI, ORI, XORI, etc.)
pub const OP_FENCE  :u8 = 0b0001111; // Fence
pub const OP_SYSTEM :u8 = 0b1110011; // System Instructions (ECALL, EBREAK, etc.)

pub fn rv32_mcu_decode_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {

    // we are expecting to get an instruction and the the program counter, both being 32-bits
    assert!(pipeline_reg.0.len() == 8);

    // we set the instruction starting at address 0x0 in the received pipeline data
    let instruction = pipeline_reg.get_u32(0x0);
    let pc = pipeline_reg.get_u32(0x4); 
    let opcode = (instruction & OPCODE_MASK) as u8;
    
    // get register indexes
    let rd_address = ((instruction >> OPCODE_L) & REG_MASK) as u8;
    let rs1_address = ((instruction >> (OPCODE_L + REG_L + FUNCT_3L)) & REG_MASK) as u8;
    let rs2_address = ((instruction >> (OPCODE_L + 2*REG_L + FUNCT_3L)) & REG_MASK) as u8; 
    // get func3 and funct7
    let func3 = ((instruction >> (OPCODE_L + REG_L)) & FUNCT_3_MASK) as u8;  
    let func7 = ((instruction >> (OPCODE_L + 3*REG_L + FUNCT_3L)) & FUNCT_7_MASK) as u8;
    
    let branch_or_jump: u8 = (opcode == OP_BRANCH || opcode == OP_JAL || opcode == OP_JALR) as u8;

    let reg_write = match opcode {
       OP_ALUI | OP_LOAD | OP_JALR | OP_ALU | OP_LUI | OP_AUIPC | OP_JAL => 1u8,
       _ => 0u8 
    };

    let mem_read_write = match opcode {
        OP_LOAD => 1u8,
        OP_STORE => 3u8,
        _ => 0u8
    };
    
    // compute immediate based on OPCODE
    let imm: u32 = match opcode {
        // I-type Instructions + Load
        // we convert instruction to i32 in order to use arithmetic right shift
        OP_ALUI | OP_LOAD | OP_JALR => (instruction as i32 >> (OPCODE_L + FUNCT_3L + 2*REG_L)) as u32 & u32::MAX, 
        OP_STORE => ((instruction as i32 >> 20) << REG_L) as u32 | ((instruction >> OPCODE_L) & REG_MASK),
        OP_BRANCH => {
            let instr7 = (instruction >> 7 & 0x1) << 11;
            let instr11_8 = (instruction >> 8 & 0xF) << 1;
            let instr30_25 = (instruction >> 25 & 0x3F) << 5;
            let instr31 = ((instruction as i32 >> 31) as u32) << 12;
            instr31 | instr7 | instr30_25 | instr11_8
        },
        OP_JAL => {
            let instr30_21 = (instruction >> 21 & 0x3FF) << 1;
            let instr20 = (instruction >> 20 & 0x1) << 11;
            let instr19_12 = (instruction >> 12 & 0xFF) << 12;
            let instr31 = ((instruction as i32 >> 31) as u32) << 20; 
            instr31 | instr19_12 | instr20 | instr30_21
        }
        OP_AUIPC | OP_LUI => instruction & 0xFFF,
        OP_ALU => 0u32,
        _ => panic!("Cannot decode this type of opcode: {opcode}") //this MCU cannot execute SYSTEM/FENCE instr
    };

    //leave read of regs at the end
    //first check commit stage(4th in our case) and see if there is a register to commit first as it might be needed for one of the rs
    
    let commit_stage = rv32_core.stages[3].read().unwrap();
    let commit_data = commit_stage.extract_data();
    if !commit_data.0.is_empty() {
        let commit_rd_address = commit_data.get_u8(0x2) & REG_MASK as u8;
        let commit_rd = commit_data.get_u32(0x5); 
        let commit_reg = commit_data.get_u8(0x0);
        if commit_reg == 0x1 {
            rv32_core.write_reg(commit_rd_address as usize, commit_rd);
        }
    }
    let (rs1, rs2) = rv32_core.read_regs(rs1_address as usize, rs2_address as usize);

    //concatanate add data into the pipeline register for next stage
    let mut pipeline_out = vec![];
    pipeline_out.push(opcode);
    pipeline_out.push(func3);
    pipeline_out.push(func7);
    pipeline_out.push(reg_write);
    pipeline_out.push(mem_read_write);
    pipeline_out.push(rd_address);
    pipeline_out.push(branch_or_jump);
    pipeline_out.extend_from_slice(&imm.to_le_bytes());
    pipeline_out.extend_from_slice(&rs1.to_le_bytes());
    pipeline_out.extend_from_slice(&rs2.to_le_bytes());
    pipeline_out.extend_from_slice(&pc.to_le_bytes());
    pipeline_out.push(rs1_address);
    pipeline_out.push(rs2_address);

    PipelineData(pipeline_out)

}