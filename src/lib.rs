use std::fs::File;
use std::io::Read;

pub struct Chip8 {
    pub opcode: u16,
    pub memory: Vec<u8>,
    pub registers: Vec<u8>,
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
            registers: vec![0; 16],
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

    pub fn cycle(&mut self) {
        let opcode:u16 = (self.memory[self.pc as usize] as u16) << 8u16 | (self.memory[(self.pc + 1) as usize] as u16);
        match opcode & 0xF000 {
            0x3000 => self.opcode_3xnn_branch_if_eq_to_val(opcode),
            0x4000 => self.opcode_3xnn_branch_if_not_eq_to_val(opcode),
            0x6000 => self.opcode_6xnn_set_reg(opcode),
            _ => println!("Not implemented!")
        }
    }

    fn opcode_3xnn_branch_if_eq_to_val(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] == (opcode & 0x00FF) as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_3xnn_branch_if_not_eq_to_val(&mut self, opcode: u16) {
        if self.registers[((opcode & 0x0F00) >> 8) as usize] != (opcode & 0x00FF) as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn opcode_6xnn_set_reg(&mut self, opcode: u16) {
        self.registers[((opcode & 0x0F00) >> 8) as usize] = (opcode & 0x00FF) as u8;
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
        assert_eq!(cpu.registers[2], 0x44);
    }

    #[test]
    fn branch_if_eq_to_val() {
        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x3155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = 0x55;
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }

        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x3155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = 0x99;        
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
            cpu.registers[1] = 0x55;
            let expected = cpu.pc + 2;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }

        {
            let mut cpu = Chip8::new();
            let instruction:u16 = 0x4155;
            cpu.memory[0x200] = (instruction >> 8u16) as u8;
            cpu.memory[0x201] = (instruction & 0xFF) as u8;
            cpu.registers[1] = 0x99;
            let expected = cpu.pc + 4;
            cpu.cycle();
            assert_eq!(cpu.pc, expected);
        }
    }
}