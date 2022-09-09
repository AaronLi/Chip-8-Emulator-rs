use std::collections::HashMap;
use std::ops::{Add, BitAnd, BitOr, BitXor, Shl, Shr};
use lazy_static::lazy_static;
use log::{error, info, warn};
use minifb::Key;
use rand::{Rng, thread_rng};
use raqote::{Color, DrawOptions, DrawTarget, SolidSource, Source};
use crate::chip8_instruction_set::{Address, Instruction, RawInstruction};

const SPRITES: [[u8; 5]; 16] = [
    [0xf0, 0x90, 0x90, 0x90, 0xf0], //0
    [0x20, 0x60, 0x20, 0x20, 0x70], //1
    [0xf0, 0x10, 0xf0, 0x80, 0xf0], //2
    [0xf0, 0x10, 0xf0, 0x10, 0xf0], //3
    [0x90, 0x90, 0xf0, 0x10, 0x10], //4
    [0xf0, 0x80, 0xf0, 0x10, 0xf0], //5
    [0xf0, 0x80, 0xf0, 0x90, 0xf0], //6
    [0xf0, 0x10, 0x20, 0x40, 0x40], //7
    [0xf0, 0x90, 0xf0, 0x90, 0xf0], //8
    [0xf0, 0x90, 0xf0, 0x10, 0xf0], //9
    [0xf0, 0x90, 0xf0, 0x90, 0x90], //a
    [0xe0, 0x90, 0xe0, 0x90, 0xe0], //b
    [0xf0, 0x80, 0x80, 0x80, 0xf0], //c
    [0xe0, 0x90, 0x90, 0x90, 0xe0], //d
    [0xf0, 0x80, 0xf0, 0x80, 0xf0], //e
    [0xf0, 0x80, 0xf0, 0x80, 0x80], //f
];

pub struct Chip8 {
    display: DrawTarget,
    display_color: Color,
    display_scale: u32,
    memory: Vec<u8>,
    stack_memory: Vec<Address>,
    instruction_pointer: Address,
    registers: [u8; 16],
    keys: [bool; 16],
    address_register: Address,
    delay_timer: u8,
    sound_timer: u8,
    keymap: HashMap<Key, u8>
}

impl Chip8 {
    pub fn new(memory: usize, stack_memory: usize, display_scale: u32, display_color: Color, keymap: HashMap<Key, u8>) -> Self {
        Chip8{
            display: DrawTarget::new(64 * display_scale as i32, 32 * display_scale as i32),
            display_color,
            display_scale,
            memory: vec![0; memory],
            stack_memory: vec![0; stack_memory],
            registers: [0; 16],
            address_register: 0,
            keys: [false; 16],
            instruction_pointer: 0x200 as Address,
            delay_timer: 0,
            sound_timer: 0,
            keymap
        }
    }

    pub fn get_screen_size(&self) -> (usize, usize) {
        (self.display.width() as usize, self.display.height() as usize)
    }

    pub fn get_screen_buffer(&self) -> &[u32] {
        self.display.get_data()
    }

    fn get_instruction_mut(&mut self, address: u16) -> (&mut u8, &mut u8){
        let (lower, upper) = self.memory.split_at_mut(address as usize + 1);
        (lower.last_mut().unwrap(), upper.first_mut().unwrap())
    }

    fn get_instruction(&self, address: u16) -> RawInstruction {
        (self.memory[address as usize], self.memory[address as usize + 1])
    }

    pub fn load(&mut self, program: &[u8]) {
        self.memory = vec![0; self.memory.len()];
        for (i, b) in program.iter().enumerate() {
            program.iter().enumerate().for_each(|(i, v)|self.memory[0x200 + i] = *v)
        }
        self.address_register = 0;
        self.instruction_pointer = 0x200;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.display.clear(SolidSource::from(Color::new(255, 0, 0, 0)));
        SPRITES.iter().flatten().enumerate().for_each(|(i, b)|self.memory[i] = *b);
    }

    pub fn tick(&mut self) {
    let to_execute = self.get_instruction(self.instruction_pointer);
    match Instruction::decode(to_execute) {
            None => error!("Unknown instruction encountered {:02x}{:02x}", to_execute.0, to_execute.1),
            Some(instruction) => self.execute(&instruction)
        }
        self.instruction_pointer += 2;
    }

    pub fn decrement_time(&mut self){
        self.delay_timer = self.delay_timer.checked_sub(1).unwrap_or(0)
    }

    pub fn set_pressed(&mut self, key: &Key, pressed: bool){
        if let Some(v) = self.keymap.get(key) {
            self.keys[*v as usize] = pressed;
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::ExecSubroutineML(_) => warn!("Not implemented {:?}", instruction),
            Instruction::ClearScreen => {
                self.display.clear(SolidSource::from(Color::new(255, 0, 0, 0)));
            },
            Instruction::ReturnFromSubroutine => {
                self.instruction_pointer = self.stack_memory.pop().expect("Popped from empty stack");
            }
            Instruction::JumpToAddress(addr) => {
                self.instruction_pointer = *addr;
                self.instruction_pointer -= 2;
            }
            Instruction::ExecSubroutine(addr) => {
                self.stack_memory.push(self.instruction_pointer);
                self.instruction_pointer = *addr;
                self.instruction_pointer -= 2;
            }
            Instruction::SkipFollowingIfRegEq(reg0, value) => {
                if self.registers[*reg0 as usize] == *value {
                    self.instruction_pointer += 2
                }
            }
            Instruction::SkipFollowingIfRegNeq(reg0, value) => {
                if self.registers[*reg0 as usize] != *value {
                    self.instruction_pointer += 2
                }
            }
            Instruction::SkipFollowingIfRegEqReg(reg0, reg1) => {
                if self.registers[*reg0 as usize] == self.registers[*reg1 as usize] {
                    self.instruction_pointer += 2
                }
            }
            Instruction::StoreToReg(reg0, value) => {
                self.registers[*reg0 as usize] = *value
            }
            Instruction::AddToReg(reg0, value) => {
                let (new_value, _) = self.registers[*reg0 as usize].overflowing_add(*value);
                self.registers[*reg0 as usize] = new_value
            }
            Instruction::MoveValue(reg0, reg1) => {
                self.registers[*reg0 as usize] = self.registers[*reg1 as usize]
            }
            Instruction::OrRegister(reg0, reg1) => {
                self.registers[*reg0 as usize] = self.registers[*reg0 as usize].bitor(self.registers[*reg1 as usize])
            }
            Instruction::AndRegister(reg0, reg1) => {
                self.registers[*reg0 as usize] = self.registers[*reg0 as usize].bitand(self.registers[*reg1 as usize])
            }
            Instruction::XorRegister(reg0, reg1) => {
                self.registers[*reg0 as usize] = self.registers[*reg0 as usize].bitxor(self.registers[*reg1 as usize])
            }
            Instruction::AddWithCarry(reg0, reg1) => {
                let (new_value, overflow) = self.registers[*reg0 as usize].overflowing_add(self.registers[*reg1 as usize]);
                self.registers[*reg0 as usize] = new_value;
                if overflow {
                    self.registers[0x0F] = 1
                }else{
                    self.registers[0x0F] = 0
                }
            }
            Instruction::SubWithCarry(reg0, reg1) => {
                let (new_value, overflow) = self.registers[*reg0 as usize].overflowing_sub(self.registers[*reg1 as usize]);
                self.registers[*reg0 as usize] = new_value;
                if overflow {
                    self.registers[0x0F] = 0
                }else{
                    self.registers[0x0F] = 1
                }
            }
            Instruction::ShiftRight(reg0, reg1) => {
                let lsb = self.registers[*reg1 as usize].bitand(0b1);
                self.registers[*reg0 as usize] = self.registers[*reg1 as usize].shr(1);
                self.registers[0xF] = lsb;
            }
            Instruction::SubWithCarry2(reg0, reg1) => {
                let (new_value, overflow) = self.registers[*reg1 as usize].overflowing_sub(self.registers[*reg0 as usize]);
                self.registers[*reg0 as usize] = new_value;
                if overflow {
                    self.registers[0xF] = 0
                }else{
                    self.registers[0xF] = 1
                }
            }
            Instruction::ShiftLeft(reg0, reg1) => {
                let msb = self.registers[*reg1 as usize].shr(7);
                self.registers[*reg0 as usize] = self.registers[*reg1 as usize].shl(1);
                self.registers[0xF] = msb
            }
            Instruction::SkipIfNE(reg0, reg1) => {
                if self.registers[*reg0 as usize] != self.registers[*reg1 as usize] {
                    self.instruction_pointer += 2
                }
            }
            Instruction::StoreAddressToI(addr) => {
                self.address_register = *addr;
            }
            Instruction::JumpWithOffset(addr) => {
                self.instruction_pointer = *addr + self.registers[0] as u16;
                self.instruction_pointer -= 2;
            }
            Instruction::RandWithMask(reg0, mask) => {
                self.registers[*reg0 as usize] = thread_rng().gen::<u8>().bitand(mask)
            }
            Instruction::DrawSprite(reg0, reg1, len) => {
                let x = self.registers[*reg0 as usize];
                let y = self.registers[*reg1 as usize];
                let sprite_address = self.address_register;
                info!("Drawing sprite at address {:x} to {}, {}", sprite_address, x, y);
                let sprite_data = (sprite_address..(sprite_address + *len as u16))
                    .map(|address| self.memory[address as usize]).collect::<Vec<u8>>();
                for (row_num, row) in sprite_data.iter().enumerate() {
                    let mut row_bits: u8 = (*row);
                    let mut column_off = 0;
                    while column_off < 8 {
                        if row_bits.shr(7) == 1u8 {
                            self.display.fill_rect(
                                ((x + column_off) as u32 * self.display_scale) as f32,
                                ((y.overflowing_add(row_num as u8).0) as u32 * self.display_scale) as f32,
                                self.display_scale as f32,
                                self.display_scale as f32,
                                &Source::Solid(SolidSource::from(self.display_color)),
                                &DrawOptions::default()
                            );
                        }else{
                            self.display.fill_rect(
                                ((x + column_off) as u32 * self.display_scale) as f32,
                                ((y.overflowing_add(row_num as u8).0) as u32 * self.display_scale) as f32,
                                self.display_scale as f32,
                                self.display_scale as f32,
                                &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)),
                                &DrawOptions::default()
                            );
                        }
                        column_off += 1;
                        row_bits = row_bits.shl(1)
                    }
                }

            }
            Instruction::SkipIfKeyPressed(reg0) => {
                if self.keys[self.registers[*reg0 as usize] as usize] {
                    self.instruction_pointer += 2;
                }
            }
            Instruction::SkipIfKeyNotPressed(reg0) => {
                if !self.keys[self.registers[*reg0 as usize] as usize] {
                    self.instruction_pointer += 2;
                }
            }
            Instruction::ReadDelayTimer(reg0) => {
                self.registers[*reg0 as usize] = self.delay_timer
            }
            Instruction::WaitForKey(reg0) => {
                let pressed = self.keys.iter().enumerate().filter(|(key, x)|**x).map(|(key, _)| key).collect::<Vec<usize>>();
                if let Some(key) = pressed.first() {
                    self.registers[*reg0 as usize] = *key as u8
                }else{
                    self.instruction_pointer -= 2;
                }

            }
            Instruction::WriteDelayTimer(reg0) => {
                self.delay_timer = self.registers[*reg0 as usize]
            }
            Instruction::WriteSoundTimer(reg0) => {
                self.sound_timer = self.registers[*reg0 as usize]
            }
            Instruction::IncrementIWithReg(reg0) => {
                self.address_register += self.registers[*reg0 as usize] as u16
            }
            Instruction::GetSpriteDataAddress(reg0) => {
                let sprite_num = self.registers[*reg0 as usize];
                let address = sprite_num as u16 * 5;
                info!("Address for sprite {} is {:x}", sprite_num, address);
                self.address_register = address;
            }
            Instruction::StoreBCD(reg0) => {
                let v = self.registers[*reg0 as usize];
                let digits = (0..=2).map(
                    |idx|{(v as i32/10_i32.pow(idx) % 10) as u8}
                ).collect::<Vec<u8>>();

                digits.iter().rev().enumerate().for_each(|(i, d)|self.memory[self.address_register as usize + i] = *d)
            }
            Instruction::StoreRegisters(reg0) => {
                self.registers[0..=*reg0 as usize].iter().enumerate().for_each(|(i, value)|self.memory[self.address_register as usize + i] = *value);
                self.address_register += *reg0 as u16 + 1;
            }
            Instruction::FillRegisters(reg0) => {
                self.registers[0..=*reg0 as usize].iter_mut().enumerate().for_each(|(i, value)|*value = self.memory[self.address_register as usize + i]);
                self.address_register += *reg0 as u16 + 1;
            }

        }
    }

    pub fn disassemble(&self) -> Vec<Option<Instruction>> {
        (0x200..self.memory.len())
            .step_by(2)
            .map(|address|self.get_instruction(address as u16))
            .map(|instruction| Instruction::decode(instruction)).collect()
    }
}