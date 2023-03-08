use crate::cartridge::{CartridgeMirroring, Mapper};

pub struct MMC1 {
    pub pkg_rom: Vec<u8>,
    pub pkg_rom_size: usize,
    pub chr_rom: Vec<u8>,
    pub chr_rom_size: usize,
    pub namespace_mirroring: CartridgeMirroring,
    pub mapper: u8,

    pub shift_register: u8,
    pub current_shift_loc: u8,
    pub control_register: u8,
    pub pkg_bank: u8,
    pub chr0_bank: u8,
    pub chr1_bank: u8,
}

impl Mapper for MMC1 {
    fn read_pkg_byte(&mut self, addr: u16) -> u8 {
        let pkg_rom_mode = (self.control_register & 0x0C) >> 2;
        let pkg_bank = u32::from(self.pkg_bank & 0xF);

        return match pkg_rom_mode {
            0..=1 => self.pkg_rom[(pkg_bank * 0x8000) + addr],
            2 => 0,
            3 => 0,
            _ => {
                panic!("Error register type")
            }
        };
    }

    fn write_pkg_byte(&mut self, addr: u16, value: u8) {
        if (value & 0x80) != 0 {
            self.shift_register = 0;
            self.current_shift_loc = 0;
            self.control_register = self.control_register | 0x0C;
            return;
        }

        let new_bit = value & 0x01;

        self.shift_register = self.shift_register >> 1;
        self.shift_register = self.shift_register | (new_bit << 4);
        self.current_shift_loc += 1;

        if self.current_shift_loc == 5 {
            match (addr & 0x6000) >> 13 {
                0 => {
                    self.control_register = self.shift_register;
                }
                1 => {
                    self.chr0_bank = self.shift_register;
                }
                2 => {
                    self.chr1_bank = self.shift_register;
                }
                3 => {
                    self.pkg_bank = self.shift_register;
                }
                _ => {
                    panic!("Error register type")
                }
            };
            self.current_shift_loc = 0;
            self.shift_register = 0x10;
        }
    }

    fn read_chr_byte(&mut self, addr: u16) -> u8 {
        return self.chr_rom[addr as usize];
    }

    fn write_chr_byte(&mut self, _addr: u16, _value: u8) {}

    fn get_namespace_mirroring(&mut self) -> CartridgeMirroring {
        return self.namespace_mirroring.clone();
    }
}
