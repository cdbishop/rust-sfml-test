extern crate rand;

use std::fs::File;
use std::io::Read;
use std::num::Wrapping;

pub trait Executable {
    fn cycle(&mut self);
}

pub struct Chip8 {
    pub opcode: u16,
    pub memory: Vec<u8>,
    pub registers: Vec<Wrapping<u8>>,
    pub index: u16,
    pub pc: u16,
    pub gfx: Vec<u8>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: Vec<u16>,
    pub sp: u16,
    pub key: Vec<u8>,
    pub draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let cpu = Chip8 {
            opcode: 0,
            memory: vec![0; 4096],
            registers: vec![Wrapping(0); 16],
            index: 0,
            pc: 0x200,
            gfx: vec![0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: vec![0; 16],
            sp: 0,
            key: vec![0; 16],
            draw_flag: false
        };

        // TODO: load font
        cpu
    }

    pub fn loadrom(&mut self, filename: String) {
        let mut file = File::open(filename).unwrap();        
        file.read(&mut self.memory[0x200..]).unwrap();
    }

    fn opcode_3xnn_branch_if_eq_to_val(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] == Wrapping((opcode & 0x00FF) as u8) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_4xnn_branch_if_not_eq_to_val(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] != Wrapping((opcode & 0x00FF) as u8) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_5xnn_branch_if_eq_to_reg(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] == self.registers[((opcode & 0x00F0) >> 4) as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_6xnn_set_reg(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = Wrapping((opcode & 0x00FF) as u8);
        self.pc += 2;
    }

    fn opcode_7xnn_add_reg_no_carry(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] += Wrapping((opcode & 0x00FF) as u8);
        self.pc += 2;
    }

    fn opcode_8xy0_set_reg(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = self.registers[((opcode & 0x0F0) >> 4) as usize];
        self.pc += 2;
    }

    fn opcode_8xy1_reg_or_eq(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = self.registers[((opcode & 0x0F00) >> 8) as usize] | self.registers[((opcode & 0x00F0) >> 4) as usize];
        self.pc += 2;
    }

    fn opcode_8xy2_reg_and_eq(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = self.registers[((opcode & 0x0F00) >> 8) as usize] & self.registers[((opcode & 0x00F0) >> 4) as usize];
        self.pc += 2;
    }

    fn opcode_8xy3_reg_xor_eq(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = self.registers[((opcode & 0x0F00) >> 8) as usize] ^ self.registers[((opcode & 0x00F0) >> 4) as usize];
        self.pc += 2;
    }

    fn opcode_8xy4_add_reg_carry(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x00F0) >> 4) as usize] > Wrapping(0xFFu8) - self.registers[((opcode & 0x0F00) >> 8) as usize] {
            self.registers[0xF] = Wrapping(1);
        } else {
            self.registers[0xF] = Wrapping(0);
        }

        self.registers[((opcode & 0x0F00) >> 8) as usize] += self.registers[((opcode & 0x00F0) >> 4) as usize];
        self.pc += 2;
    }
    
    fn opcode_8xy5_sub_eq_reg_carry(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x00F0) >> 4) as usize] > self.registers[((opcode & 0x0F00) >> 8) as usize] {
            self.registers[0xF] = Wrapping(0);
        } else {
            self.registers[0xF] = Wrapping(1);
        }

        self.registers[((opcode & 0x0F00) >> 8) as usize] -= self.registers[((opcode & 0x00F0) >> 4) as usize];
        self.pc += 2;
    }

    fn opcode_8xy6_reg_shift_right(&mut self, opcode: u16) {
        self.registers[0xF] = self.registers[((opcode & 0xF00) >> 8) as usize] & Wrapping(1);
        self.registers[((opcode & 0xF00) >> 8) as usize] >>= 1;
        self.pc += 2;
    }

    fn opcode_8xy7_sub_reg_carry(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x00F0) >> 4) as usize] > self.registers[((opcode & 0x0F00) >> 8) as usize] {
            self.registers[0xF] = Wrapping(0);
        } else {
            self.registers[0xF] = Wrapping(1);
        }

        self.registers[((opcode & 0x0F00) >> 8) as usize] = self.registers[((opcode & 0x00F0) >> 4) as usize] - self.registers[((opcode & 0x0F00) >> 8) as usize];
        self.pc += 2;
    }

    fn opcode_8xye_reg_shift_left(&mut self, opcode: u16) {
        self.registers[0xF] = self.registers[((opcode & 0x0F00) >> 8) as usize] >> 7;
        self.registers[((opcode & 0x0F00) >> 8) as usize] <<= 1;
        self.pc += 2;
    }

    fn opcode_9xy0_branch_if_not_eq_reg(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] != self.registers[((opcode & 0x00F0) >> 4) as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_annn_set_index(&mut self, opcode: u16) {
        self.index = opcode & 0xFFF;
        self.pc += 2;
    }

    fn opcode_bnnn_jump_to_addr(&mut self, opcode: u16) {
        let offset: u16 = self.registers[0].0.into();
        self.pc = offset + ((opcode & 0xFFF) as u16);
    }

    fn opcode_cxnn_rand(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = rand::random();
        self.pc += 2;
    }

    fn opcode_fx07_get_delay(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = Wrapping(self.delay_timer);
        self.pc += 2;
    }
}

impl Executable for Chip8 {
    fn cycle(&mut self) {
        let opcode:u16 = (self.memory[self.pc as usize] as u16) << 8u16 | (self.memory[(self.pc + 1) as usize] as u16);
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x000F {
                0x0000 => println!("TODO: clear screen"),
                0x000E => println!("TODO: subroutine return"),
                _ => println!("opcode: {} Not Implemented", opcode),
            },
            0x1000 => println!("TODO: goto"),
            0x2000 => println!("TODO: subroutine"),
            0x3000 => self.opcode_3xnn_branch_if_eq_to_val(opcode),
            0x4000 => self.opcode_4xnn_branch_if_not_eq_to_val(opcode),
            0x5000 => self.opcode_5xnn_branch_if_eq_to_reg(opcode),
            0x6000 => self.opcode_6xnn_set_reg(opcode),
            0x7000 => self.opcode_7xnn_add_reg_no_carry(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.opcode_8xy0_set_reg(opcode),
                0x0001 => self.opcode_8xy1_reg_or_eq(opcode),
                0x0002 => self.opcode_8xy2_reg_and_eq(opcode),
                0x0003 => self.opcode_8xy3_reg_xor_eq(opcode),
                0x0004 => self.opcode_8xy4_add_reg_carry(opcode),
                0x0005 => self.opcode_8xy5_sub_eq_reg_carry(opcode),
                0x0006 => self.opcode_8xy6_reg_shift_right(opcode),
                0x0007 => self.opcode_8xy7_sub_reg_carry(opcode),
                0x000E => self.opcode_8xye_reg_shift_left(opcode),
                _ => println!("opcode: {} Not Implemented", opcode),
            },
            0x9000 => self.opcode_9xy0_branch_if_not_eq_reg(opcode),
            0xA000 => self.opcode_annn_set_index(opcode),
            0xB000 => self.opcode_bnnn_jump_to_addr(opcode),
            0xC000 => self.opcode_cxnn_rand(opcode),
            0xD000 => println!("TODO: draw"),
            0xE000 => match opcode & 0x000F {
                0x000E => println!("TODO: skip if key"),
                0x0001 => println!("TODO: skip if NOT key"),
                _ => println!("opcode: {} Not Implemented", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.opcode_fx07_get_delay(opcode),
                0x000A => println!("TODO: wait for key"),
                0x0015 => println!("TODO: set delay timer"),
                0x0018 => println!("TODO: set sound timer"),
                0x001E => println!("TODO: add index"),
                0x0029 => println!("TODO: set sprite addr"),
                0x0033 => println!("TODO: bcd"),
                0x0055 => println!("TODO: regdump"),
                0x0065 => println!("TODO: reg load"),
                _ => println!("opcode: {} Not Implemented", opcode),
            },
            _ => println!("opcode: {} Not Implemented", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_set() {
        let mut cpu = Chip8::new();
        let instruction:u16 = 0x6244;
        cpu.memory[0x200] = (instruction >> 8u16) as u8;
        cpu.memory[0x201] = (instruction & 0xFF) as u8;
        cpu.cycle();
        assert_eq!(cpu.registers[2], Wrapping(0x44));
    }

    #[test]
    fn branch_if_eq_to_val() {
        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x3155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x55);
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }

        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x3155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x99);
            let expected = cpu.pc + 2;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
    }

    #[test]
    fn branch_if_not_eq_to_val() {
        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x4155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x55);
            let expected = cpu.pc + 2;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }

        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x4155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x99);
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
    }

    #[test]
    fn branch_if_eq_to_reg() {
        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x5120;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;    
            cpu.registers[1] = Wrapping(0x55);
            cpu.registers[2] = Wrapping(0x55);
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }

        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x5120;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;    
            cpu.registers[1] = Wrapping(0x99);
            cpu.registers[2] = Wrapping(0x55);
            let expected = cpu.pc + 2;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
    }

    #[test]
    fn add_reg_no_carry() {
        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x7301;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;                
            cpu.cycle();
            assert_eq!(cpu.registers[0x3], Wrapping(1));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }

        {
            let mut cpu = Chip8::new();
            let instruction2:u16 = 0x73FE;
            cpu.memory[0x200] = (instruction2 >> 8u16) as u8;
            cpu.memory[0x201] = (instruction2 & 0xFF) as u8;
            cpu.registers[3] = Wrapping(0x5);
            cpu.cycle();
            assert_eq!(cpu.registers[0x3], Wrapping(0x03));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }
    }

    #[test]
    fn set_reg() {
        let mut cpu = Chip8::new();
        let instruction2:u16 = 0x8230;
        cpu.memory[0x200] = (instruction2 >> 8u16) as u8;
        cpu.memory[0x201] = (instruction2 & 0xFF) as u8;
        cpu.registers[3] = Wrapping(0x5);
        cpu.cycle();
        assert_eq!(cpu.registers[0x2], cpu.registers[0x3]);
    }

    #[test]
    fn reg_or_eq() {
        let mut cpu = Chip8::new();
        let instruction2:u16 = 0x8231;
        cpu.memory[0x200] = (instruction2 >> 8u16) as u8;
        cpu.memory[0x201] = (instruction2 & 0xFF) as u8;
        cpu.registers[2] = Wrapping(0x9);
        cpu.registers[3] = Wrapping(0x2);
        cpu.cycle();
        assert_eq!(cpu.registers[0x2], Wrapping(0xb));
    }
    
    #[test]
    fn reg_and_eq() {
        let instruction:u16 = 0x8232;
        {
            let mut cpu = Chip8::new();
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[2] = Wrapping(0x9);
            cpu.registers[3] = Wrapping(0xF);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0x9));
        }
        {
            let mut cpu = Chip8::new();
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[2] = Wrapping(0x9);
            cpu.registers[3] = Wrapping(0x2);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0));
        }
    }
    
    #[test]
    fn reg_xor_eq() {
        let instruction:u16 = 0x8233;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[2] = Wrapping(0x9);
            cpu.registers[3] = Wrapping(0xF);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0x6));
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[2] = Wrapping(0x9);
            cpu.registers[3] = Wrapping(0x2);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0xb));
        }
    }

     #[test]
    fn add_reg_carry() {
        let instruction:u16 = 0x8124;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x4);
            cpu.registers[2] = Wrapping(0x4);
            cpu.cycle();
            assert_eq!(cpu.registers[0x1], Wrapping(0x8));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x4);
            cpu.registers[2] = Wrapping(0xFF);
            cpu.cycle();
            assert_eq!(cpu.registers[0x1], Wrapping(0x3));
            assert_eq!(cpu.registers[0xF], Wrapping(1));
        }
    }

    #[test]
    fn sub_eq_reg_carry() {
        let instruction:u16 = 0x8125;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x4);
            cpu.registers[2] = Wrapping(0x2);
            cpu.cycle();
            assert_eq!(cpu.registers[0x1], Wrapping(0x2));
            assert_eq!(cpu.registers[0xF], Wrapping(1));
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = Wrapping(0x4);
            cpu.registers[2] = Wrapping(0xFF);
            cpu.cycle();
            assert_eq!(cpu.registers[0x1], Wrapping(0x5));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }
    }

    #[test]
    fn reg_shift_right() {
        let instruction:u16 = 0x8206;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[2] = Wrapping(0x3);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0x1));
            assert_eq!(cpu.registers[0xF], Wrapping(1));
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[2] = Wrapping(0x4);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0x2));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }
    }
    
    #[test]
    fn reg_shift_left() {
        let instruction:u16 = 0x820E;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[2] = Wrapping(0xFF);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0xFE));
            assert_eq!(cpu.registers[0xF], Wrapping(1));
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[2] = Wrapping(0xB);
            cpu.cycle();
            assert_eq!(cpu.registers[0x2], Wrapping(0x16));
            assert_eq!(cpu.registers[0xF], Wrapping(0));
        }
    }

    #[test]
    fn branch_if_not_eq_to_reg() {
        let instruction:u16 = 0x9120;
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[1] = Wrapping(0x55);
            cpu.registers[2] = Wrapping(0x55);
            let expected = cpu.pc + 2;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
        {
            let mut cpu = Chip8::new();            
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;            
            cpu.registers[1] = Wrapping(0x99);
            cpu.registers[2] = Wrapping(0x55);
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
    }

    #[test]
    fn set_index() {
        let instruction:u16 = 0xA123;
        let mut cpu = Chip8::new();            
        cpu.memory[0x200] = (instruction >> 8u16) as u8;
        cpu.memory[0x201] = (instruction & 0xFF) as u8;                    
        let expected = 0x123;
        cpu.cycle();
        assert_eq!(cpu.index, expected);
    }    

    #[test]
    fn jump_to_addr() {
        let instruction:u16 = 0xB123;
        let mut cpu = Chip8::new();            
        cpu.memory[0x200] = (instruction >> 8u16) as u8;
        cpu.memory[0x201] = (instruction & 0xFF) as u8;                    
        let expected = 0x123;
        cpu.cycle();
        assert_eq!(cpu.pc, expected);
    } 

    #[test]
    fn read_delay_timer() {
        let instruction:u16 = 0xF207;
        let mut cpu = Chip8::new();            
        cpu.memory[0x200] = (instruction >> 8u16) as u8;
        cpu.memory[0x201] = (instruction & 0xFF) as u8;                    
        cpu.delay_timer = 0xF;
        cpu.cycle();
        assert_eq!(cpu.registers[0x2], Wrapping(cpu.delay_timer));
    } 
}