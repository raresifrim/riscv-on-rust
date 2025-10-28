use crate::risc_soc::risc_soc::RiscCore;
use crate::risc_soc::{pipeline_stage::PipelineData, risc_soc::RiscWord};
use crate::rv32i_baremetal::core::{EX_STAGE, WB_STAGE};
use crate::rv32i_baremetal::decode::REG_MASK;
use crate::rv32i_baremetal::decode::{
    OP_ALU, OP_ALUI, OP_AUIPC, OP_BRANCH, OP_JAL, OP_JALR, OP_LOAD, OP_LUI, OP_STORE,
};
use std::u32;

pub fn rv32_mcu_execute_stage(pipeline_reg: &PipelineData, rv32_core: &RiscCore) -> PipelineData {
    let opcode = pipeline_reg.get_u8(0x0);
    let func3 = pipeline_reg.get_u8(0x1);
    let func7 = pipeline_reg.get_u8(0x2);
    let reg_write = pipeline_reg.get_u8(0x3);
    let mem_read_write = pipeline_reg.get_u8(0x4);
    let rd_address = pipeline_reg.get_u8(0x5);
    let branch_or_jump = pipeline_reg.get_u8(0x6);

    let imm = pipeline_reg.get_u32(0x7);
    let mut rs1 = pipeline_reg.get_u32(0xB);
    let mut rs2 = pipeline_reg.get_u32(0xF);
    let mut pc = pipeline_reg.get_u32(0x13);

    let rs1_address = pipeline_reg.get_u8(0x17);
    let rs2_address = pipeline_reg.get_u8(0x18);

    //Critical region where we wait for notify from WB
    // wait for WB stage to get latest values for our registers
    let wb_wire_data = &rv32_core.cdb[WB_STAGE];
    let wb_data = wb_wire_data.read();

    if !wb_data.0.is_empty() {
        let wb_reg_write = wb_data.get_u8(0x0);
        let wb_rd_address = wb_data.get_u8(0x1) & REG_MASK as u8;
        let wb_rd_value = wb_data.get_u32(0x2);
        if wb_reg_write == 0x1 && wb_rd_address == rs1_address {
            rs1 = wb_rd_value;
        }
        if wb_reg_write == 0x1 && wb_rd_address == rs2_address {
            rs2 = wb_rd_value;
        }
    }

    let mut take_jump: u8 = 0u8;
    let mut alu_out: u32 = 0u32;

    match opcode {
        OP_ALU => {
            if func3 == 0b0 && func7 == 0b0 {
                //add
                alu_out = ((rs1 as i32) + (rs2 as i32)) as RiscWord;
            } else if func3 == 0b000 && func7 == 0b0100000 {
                //sub
                alu_out = (rs1 as i32 - rs2 as i32) as RiscWord;
            } else if func3 == 0b001 {
                //sll
                alu_out = rs1 << rs2;
            } else if func3 == 0b010 {
                //slt
                alu_out = ((rs1 as i32) < (rs2 as i32)) as RiscWord;
            } else if func3 == 0b011 {
                //sltu
                alu_out = (rs1 < rs2) as RiscWord;
            } else if func3 == 0b100 {
                //xor
                alu_out = rs1 ^ rs2;
            } else if func3 == 0b101 && func7 == 0b0 {
                //srl
                alu_out = rs1 >> rs2;
            } else if func3 == 0b101 && func7 == 0b0100000 {
                //sra
                alu_out = (rs1 as i32 >> rs2) as RiscWord;
            } else if func3 == 0b110 {
                //or
                alu_out = rs1 | rs2;
            } else if func3 == 0b111 {
                //and
                alu_out = rs1 & rs2;
            }
        }
        OP_ALUI => {
            if func3 == 0b0 {
                //add
                alu_out = ((rs1 as i32) + (imm as i32)) as RiscWord;
            } else if func3 == 0b001 {
                //slli
                alu_out = rs1 << imm;
            } else if func3 == 0b010 {
                //slti
                alu_out = ((rs1 as i32) < (imm as i32)) as RiscWord;
            } else if func3 == 0b011 {
                //sltiu
                alu_out = (rs1 < imm) as RiscWord;
            } else if func3 == 0b100 {
                //xori
                alu_out = rs1 ^ imm;
            } else if func3 == 0b101 && func7 == 0b0 {
                //srli
                alu_out = rs1 >> imm;
            } else if func3 == 0b101 && func7 == 0b0100000 {
                //srai
                alu_out = (rs1 as i32 >> imm) as RiscWord;
            } else if func3 == 0b110 {
                //ori
                alu_out = rs1 | imm;
            } else if func3 == 0b111 {
                //andi
                alu_out = rs1 & imm;
            }
        }
        OP_JAL => {
            alu_out = pc + 4;
            pc = (pc as i32 + imm as i32) as RiscWord;
            take_jump = 0x1;
        }
        OP_JALR => {
            alu_out = pc + 4;
            pc = ((rs1 as i32) + (imm as i32)) as RiscWord;
            take_jump = 0x1;
        }
        OP_LOAD | OP_STORE => {
            alu_out = (rs1 as i32 + imm as i32) as RiscWord;
        }
        OP_BRANCH => {
            pc = (pc as i32 + imm as i32) as RiscWord;
            if func3 == 0b000 {
                //beq
                take_jump = (rs1 == rs2) as u8;
            } else if func3 == 0b001 {
                //bne
                take_jump = (rs1 != rs2) as u8;
            } else if func3 == 0b100 {
                //blt
                take_jump = ((rs1 as i32) < (rs2 as i32)) as u8;
            } else if func3 == 0b101 {
                //bge
                take_jump = ((rs1 as i32) >= (rs2 as i32)) as u8;
            } else if func3 == 0b110 {
                //bltu
                take_jump = (rs1 < rs2) as u8;
            } else if func3 == 0b111 {
                //bgeu
                take_jump = (rs1 >= rs2) as u8;
            }
        }
        OP_LUI => {
            alu_out = imm;
        }
        OP_AUIPC => {
            alu_out = (pc as i32 + imm as i32) as RiscWord;
        }
        _ => {}
    }

    //send info about branch to IF
    let mut if_pipe = vec![];
    if_pipe.push(branch_or_jump);
    if_pipe.push(take_jump);
    if_pipe.extend_from_slice(&pc.to_le_bytes());
    let ex_data = PipelineData(if_pipe);
    let ex_wire_data = &rv32_core.cdb[EX_STAGE];
    ex_wire_data.assign(ex_data);

    let mut pipeline_out = vec![];
    pipeline_out.push(reg_write);
    pipeline_out.push(mem_read_write);
    pipeline_out.push(rd_address);
    pipeline_out.push(func3);
    pipeline_out.extend_from_slice(&alu_out.to_le_bytes());
    pipeline_out.extend_from_slice(&rs2.to_le_bytes());

    PipelineData(pipeline_out)
}
