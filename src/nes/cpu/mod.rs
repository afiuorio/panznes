use crate::memory::Memory;
use crate::nes::cpu::opcodes::OPCODES;
use crate::nes::ppu::registers::{PPUCTRL, PPUSTATUS};
use crate::nes::{FlagRegister, Interrupt, Nes};

mod opcodes;

impl Memory for Nes {
    fn read_byte(&mut self, addr: u16) -> u8 {
        return self.read_cpu_byte(addr);
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        self.write_cpu_byte(addr, value);
    }
}

impl Nes {
    pub fn execute_instruction(&mut self) -> u32 {
        if self.ppuctrl.contains(PPUCTRL::NMI_ENABLED)
            && self.ppustatus.contains(PPUSTATUS::V_BLANK)
        {
            self.ppuctrl.remove(PPUCTRL::NMI_ENABLED);
            self.ppustatus.remove(PPUSTATUS::V_BLANK);
            return self.raise_interrupt(Interrupt::NMI);
        }

        if self.request_dma == true {
            self.request_dma = false;
            self.dma_transfert();
            return 512;
        }

        let opcode = self.read_byte(self.prog_counter);
        let instruction = &OPCODES[opcode as usize];
        /*println!(
            "Executing {:x}: {:x} (a:{:x}, x:{:x}, y:{:x}, S:{:x}, P:{:x}  )",
            self.prog_counter, opcode, self.a, self.x, self.y, self.stack_ptr, self.flag
        );*/
        self.prog_counter = self.prog_counter.wrapping_add(1);

        let cycles = match opcode {
            //Register/Immediate to Register move
            0xA8 => {
                self.y = self.a;
                self.update_zero_and_negative_flags(self.y);
                2
            }
            0xAA => {
                self.x = self.a;
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0xBA => {
                self.x = self.stack_ptr;
                self.update_zero_and_negative_flags(self.x);
                2
            }
            0x98 => {
                self.a = self.y;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x8A => {
                self.a = self.x;
                self.update_zero_and_negative_flags(self.a);
                2
            }
            0x9A => {
                self.stack_ptr = self.x;
                2
            }

            //LDA
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(instruction),
            //LDX
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(instruction),
            //LDY
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(instruction),

            //STA
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.st(self.a, instruction),
            //STX
            0x86 | 0x96 | 0x8E => self.st(self.x, instruction),
            //STY
            0x84 | 0x94 | 0x8C => self.st(self.y, instruction),

            //PHA
            0x48 => self.push(self.a),

            //PHP
            0x08 => self.push(self.flag.bits | 0x30),

            //PLA
            0x68 => {
                let (value, cycles) = self.pop();
                self.a = value;
                self.update_zero_and_negative_flags(self.a);
                cycles
            }

            //PLP
            0x28 => {
                let (value, cycles) = self.pop();
                self.flag =
                    FlagRegister::from_bits_truncate((value & 0xCF) | (self.flag.bits & 0x30));
                cycles
            }

            //ADC
            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.adc(instruction),
            //SBC
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 | 0xEB => self.sbc(instruction),
            //AND
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                self.logical_register_a(Nes::and, instruction)
            }
            //XOR
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                self.logical_register_a(Nes::xor, instruction)
            }
            //OR
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                self.logical_register_a(Nes::or, instruction)
            }
            //CMP
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                self.compare(instruction, self.a)
            }
            //CPX
            0xE0 | 0xE4 | 0xEC => self.compare(instruction, self.x),
            //CPY
            0xC0 | 0xC4 | 0xCC => self.compare(instruction, self.y),
            //BIT
            0x24 | 0x2C => self.bit(instruction),
            //INC
            0xE6 | 0xF6 | 0xEE | 0xFE => self.increase_in_memory(instruction),
            //DEC
            0xC6 | 0xD6 | 0xCE | 0xDE => self.decrease_in_memory(instruction),
            //INX
            0xE8 => {
                self.x = self.x.wrapping_add(1);
                self.update_zero_and_negative_flags(self.x);
                2
            }
            //INY
            0xC8 => {
                self.y = self.y.wrapping_add(1);
                self.update_zero_and_negative_flags(self.y);
                2
            }
            //DEX
            0xCA => {
                self.x = self.x.wrapping_sub(1);
                self.update_zero_and_negative_flags(self.x);
                2
            }
            //DEY
            0x88 => {
                self.y = self.y.wrapping_sub(1);
                self.update_zero_and_negative_flags(self.y);
                2
            }
            //ASL
            0x0A => self.shl_acc(),
            0x06 | 0x16 | 0x0E | 0x1E => self.shl(instruction),
            //LSR
            0x4A => self.shr_acc(),
            0x46 | 0x56 | 0x4E | 0x5E => self.shr(instruction),
            //ROL
            0x2A => self.rol_acc(),
            0x26 | 0x36 | 0x2E | 0x3E => self.rol(instruction),
            //ROR
            0x6A => self.ror_acc(),
            0x66 | 0x76 | 0x6E | 0x7E => self.ror(instruction),
            //JMP
            0x4C => {
                let jump_addr = self.read_instruction_operand_16bit();
                self.prog_counter = jump_addr;
                3
            }
            0x6C => {
                let jump_addr_ptr = self.read_instruction_operand_16bit();
                let jump_addr = if jump_addr_ptr & 0xFF == 0xFF {
                    //This is an interesting glitch in NES CPU. If the operand is between two pages, the second byte is taken from
                    //the head of the current page. In other words, we cannot do any page cross here.
                    let be = self.read_cpu_byte(jump_addr_ptr);
                    let msb = self.read_cpu_byte(jump_addr_ptr & 0xF0);
                    (u16::from(msb) << 8) | u16::from(be)
                } else {
                    self.read_word(jump_addr_ptr)
                };
                self.prog_counter = jump_addr;
                5
            }
            //JSR
            0x20 => {
                let jump_addr = self.read_instruction_operand_16bit();
                let ret_pc = self.prog_counter - 1;
                self.push((ret_pc >> 8) as u8);
                self.push((ret_pc & 0xFF) as u8);
                self.prog_counter = jump_addr;
                6
            }
            //RTI
            0x40 => {
                let (flag, _) = self.pop();
                let current_brk = if self.flag.contains(FlagRegister::BREAK) {
                    1
                } else {
                    0
                };

                self.flag = FlagRegister::from_bits_truncate(flag);
                if current_brk != 0 {
                    self.flag.insert(FlagRegister::BREAK);
                }
                self.flag.insert(FlagRegister::UNUSED);

                let (low_byte, _) = self.pop();
                let (hi_byte, _) = self.pop();
                let prog_counter = ((hi_byte as u16) << 8) | low_byte as u16;
                self.prog_counter = prog_counter;
                6
            }
            //RTS
            0x60 => {
                let (low_byte, _) = self.pop();
                let (hi_byte, _) = self.pop();
                let prog_counter = ((hi_byte as u16) << 8) | low_byte as u16;
                self.prog_counter = prog_counter.wrapping_add(1);
                6
            }
            //BPL
            0x10 => self.conditional_jump(instruction, !self.flag.contains(FlagRegister::NEGATIV)),
            //BMI
            0x30 => self.conditional_jump(instruction, self.flag.contains(FlagRegister::NEGATIV)),
            //BVC
            0x50 => self.conditional_jump(instruction, !self.flag.contains(FlagRegister::OVERFLOW)),
            //BVS
            0x70 => self.conditional_jump(instruction, self.flag.contains(FlagRegister::OVERFLOW)),
            //BCC
            0x90 => self.conditional_jump(instruction, !self.flag.contains(FlagRegister::CARRY)),
            //BCS
            0xB0 => self.conditional_jump(instruction, self.flag.contains(FlagRegister::CARRY)),
            //BNE
            0xD0 => self.conditional_jump(instruction, !self.flag.contains(FlagRegister::ZERO)),
            //BEQ
            0xF0 => self.conditional_jump(instruction, self.flag.contains(FlagRegister::ZERO)),
            //CLC
            0x18 => {
                self.flag.remove(FlagRegister::CARRY);
                instruction.cycles
            }
            //CLI
            0x58 => {
                self.flag.remove(FlagRegister::IRQ_DISABLE);
                instruction.cycles
            }
            //CLD
            0xD8 => {
                self.flag.remove(FlagRegister::DECIMAL_MODE);
                instruction.cycles
            }
            //CLV
            0xB8 => {
                self.flag.remove(FlagRegister::OVERFLOW);
                instruction.cycles
            }
            //SEC
            0x38 => {
                self.flag.insert(FlagRegister::CARRY);
                instruction.cycles
            }
            //SEI
            0x78 => {
                self.flag.insert(FlagRegister::IRQ_DISABLE);
                instruction.cycles
            }
            //SED
            0xF8 => {
                self.flag.insert(FlagRegister::DECIMAL_MODE);
                instruction.cycles
            }
            //BRK
            0x00 => self.raise_interrupt(Interrupt::BREAK),
            //NOP
            0xEA | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => instruction.cycles,
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 | 0x04 | 0x44 | 0x64 => {
                self.read_instruction_operand_8bit();
                3
            }
            //Illegal opcodes
            0x0b | 0x2B | 0x4B | 0x6B | 0xCB | 0xAB => {
                self.read_instruction_operand_8bit();
                2
            }
            _ => todo!(),
        };
        return cycles;
    }
}
