use std::ops::{BitAnd, Shl, Shr};
use crate::chip8_instruction_set::Instruction::{AddToReg, AddWithCarry, AndRegister, DrawSprite, FillRegisters, GetSpriteDataAddress, IncrementIWithReg, JumpToAddress, JumpWithOffset, MoveValue, OrRegister, RandWithMask, ReadDelayTimer, ShiftLeft, ShiftRight, SkipFollowingIfRegEq, SkipFollowingIfRegEqReg, SkipFollowingIfRegNeq, SkipIfKeyNotPressed, SkipIfKeyPressed, SkipIfNE, StoreAddressToI, StoreBCD, StoreRegisters, StoreToReg, SubWithCarry, SubWithCarry2, WaitForKey, WriteDelayTimer, WriteSoundTimer, XorRegister};

pub type RegisterTo = u8;
pub type Register = u8;
pub type Address = u16;
pub type Value = u8;
pub type RawInstruction = (u8, u8);

#[derive(Debug)]
pub enum Instruction {
    ExecSubroutineML(Address),
    ClearScreen,
    ReturnFromSubroutine,
    JumpToAddress(Address),
    ExecSubroutine(Address),
    SkipFollowingIfRegEq(Register, Value),
    SkipFollowingIfRegNeq(Register, Value),
    SkipFollowingIfRegEqReg(Register, Register),
    StoreToReg(RegisterTo, Value),
    AddToReg(RegisterTo, Value),
    MoveValue(RegisterTo, Register),
    OrRegister(RegisterTo, Register),
    AndRegister(RegisterTo, Register),
    XorRegister(RegisterTo, Register),
    AddWithCarry(RegisterTo, Register),
    SubWithCarry(RegisterTo, Register),
    ShiftRight(RegisterTo, Register),
    SubWithCarry2(RegisterTo, Register),
    ShiftLeft(RegisterTo, Register),
    SkipIfNE(Register, Register),
    StoreAddressToI(Address),
    JumpWithOffset(Address),
    RandWithMask(RegisterTo, Value),
    DrawSprite(Register, RegisterTo, Value),
    SkipIfKeyPressed(Register),
    SkipIfKeyNotPressed(Register),
    ReadDelayTimer(RegisterTo),
    WaitForKey(RegisterTo),
    WriteDelayTimer(Register),
    WriteSoundTimer(Register),
    IncrementIWithReg(Register),
    GetSpriteDataAddress(Register),
    StoreBCD(Register),
    StoreRegisters(Register),
    FillRegisters(Register)
}

impl Instruction {
    pub(crate) fn decode(instruction: RawInstruction) -> Option<Self> {
        match instruction.0.shr(4u8) {
            0 => Self::decode_0_class_instruction(instruction),
            1 => Self::decode_1_class_instruction(instruction),
            2 => Self::decode_2_class_instruction(instruction),
            3 => Self::decode_3_class_instruction(instruction),
            4 => Self::decode_4_class_instruction(instruction),
            5 => Self::decode_5_class_instruction(instruction),
            6 => Self::decode_6_class_instruction(instruction),
            7 => Self::decode_7_class_instruction(instruction),
            8 => Self::decode_8_class_instruction(instruction),
            9 => Self::decode_9_class_instruction(instruction),
            0xA => Self::decode_a_class_instruction(instruction),
            0xB => Self::decode_b_class_instruction(instruction),
            0xC => Self::decode_c_class_instruction(instruction),
            0xD => Self::decode_d_class_instruction(instruction),
            0xE => Self::decode_e_class_instruction(instruction),
            0xF => Self::decode_f_class_instruction(instruction),
            _ => None
        }
    }

    fn get_instruction_class(upper_byte: u8) -> u8 {
        upper_byte.rotate_right(4)
    }

    fn get_address(instruction: RawInstruction) -> Address {
        ((instruction.0.bitand(0x0f) as u16).shl(8) + instruction.1 as u16) as Address
    }

    fn get_registers(instruction: RawInstruction) -> (Register, Register) {
        (instruction.0.bitand(0x0f) as Register, instruction.1.shr(4) as Register)
    }

    fn decode_0_class_instruction(instruction: RawInstruction) -> Option<Instruction> {
        match instruction.0 {
            0x00 => {
                match instruction.1 {
                    0xE0 => Some(Instruction::ClearScreen),
                    0xEE => Some(Instruction::ReturnFromSubroutine),
                    _ => None
                }
            },
            0x02..=0x0f => Some(Instruction::ExecSubroutineML(Instruction::get_address(instruction))),
            _ => None
        }
    }

    fn decode_1_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x12..=0x1F => Some(Instruction::JumpToAddress(Instruction::get_address(instruction))),
            _ => None
        }
    }

    fn decode_2_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x22..=0x2f => Some(Instruction::ExecSubroutine(Instruction::get_address(instruction))),
            _ => None
        }
    }

    fn decode_3_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x30..=0x3f => Some(SkipFollowingIfRegEq(Instruction::get_registers(instruction).0, instruction.1)),
            _ => None
        }
    }

    fn decode_4_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x40..=0x4f => Some(SkipFollowingIfRegNeq(Instruction::get_registers(instruction).0, instruction.1)),
            _ => None
        }
    }

    fn decode_5_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        let registers = Instruction::get_registers(instruction);
        match instruction.0 {
            0x50..=0x5f => if instruction.1.bitand(0x0f) == 0 {Some(SkipFollowingIfRegEqReg(registers.0, registers.1))} else {None},
            _ => None
        }
    }

    fn decode_6_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x60..=0x6f => Some(StoreToReg(Instruction::get_registers(instruction).0, instruction.1)),
            _ => None
        }

    }

    fn decode_7_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x70..=0x7f => Some(AddToReg(Instruction::get_registers(instruction).0, instruction.1)),
            _ => None
        }
    }

    fn decode_8_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        let registers = Instruction::get_registers(instruction);
        match instruction.0 {
            0x80..=0x8f => match instruction.1.bitand(0x0f) {
                0 => Some(MoveValue(registers.0, registers.1)),
                1 => Some(OrRegister(registers.0, registers.1)),
                2 => Some(AndRegister(registers.0, registers.1)),
                3 => Some(XorRegister(registers.0, registers.1)),
                4 => Some(AddWithCarry(registers.0, registers.1)),
                5 => Some(SubWithCarry(registers.0, registers.1)),
                6 => Some(ShiftRight(registers.0, registers.1)),
                7 => Some(SubWithCarry2(registers.0, registers.1)),
                0xE => Some(ShiftLeft(registers.0, registers.1)),
                _ => None
            },
            _ => None
        }
    }

    fn decode_9_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0x90..=0x9F => {
                if instruction.1.bitand(0x0F) == 0 {
                    let reg = Instruction::get_registers(instruction);
                    Some(SkipIfNE(reg.0, reg.1))
                }else{
                    None
                }
            },
            _ => None
        }
    }

    fn decode_a_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0xA2..=0xAF => {
                Some(
                    StoreAddressToI(Instruction::get_address(instruction))
                )
            },
            _ => None
        }
    }

    fn decode_b_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0xB2..=0xBF => {
                Some(
                    JumpWithOffset(Instruction::get_address(instruction))
                )
            },
            _ => None
        }
    }

    fn decode_c_class_instruction(instruction: RawInstruction) -> Option<Instruction> {
        match instruction.0 {
            0xC0..=0xCF => {
                Some(RandWithMask(Instruction::get_registers(instruction).0, instruction.1 as Value))
            },
            _ => None
        }
    }

    fn decode_d_class_instruction(instruction: RawInstruction) -> Option<Instruction> {
        match instruction.0 {
            0xD0..=0xDF => {
                let regs = Instruction::get_registers(instruction);
                Some(DrawSprite(regs.0, regs.1, instruction.1.bitand(0x0f)))
            },
            _ => None
        }
    }

    fn decode_e_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0xE0..=0xEF => {
                match instruction.1 {
                    0x9E => Some(SkipIfKeyPressed(Instruction::get_registers(instruction).0)),
                    0xA1 => Some(SkipIfKeyNotPressed(Instruction::get_registers(instruction).0)),
                    _ => None
                }
            },
            _ => None
        }
    }

    fn decode_f_class_instruction(instruction: RawInstruction) -> Option<Instruction>{
        match instruction.0 {
            0xF0..=0xFF => {
                let reg = Instruction::get_registers(instruction).0;
                match instruction.1 {
                    0x07 => Some(ReadDelayTimer(reg)),
                    0x0A => Some(WaitForKey(reg)),
                    0x15 => Some(WriteDelayTimer(reg)),
                    0x18 => Some(WriteSoundTimer(reg)),
                    0x1E => Some(IncrementIWithReg(reg)),
                    0x29 => Some(GetSpriteDataAddress(reg)),
                    0x33 => Some(StoreBCD(reg)),
                    0x55 => Some(StoreRegisters(reg)),
                    0x65 => Some(FillRegisters(reg)),
                    _ => None
                }
            },
            _ => None
        }
    }
}