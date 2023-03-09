use crate::cartridge::{CartridgeMirroring, Mapper};
use std::ops::{Add, Mul};

pub struct MMC1 {
    pub pkg_rom: Vec<u8>,
    pub pkg_rom_size: usize,
    pub chr_rom: Vec<u8>,
    pub chr_rom_size: usize,
    pub namespace_mirroring: CartridgeMirroring,
    pub mapper: u8,
    pub ram: Vec<u8>,

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
            0..=1 => {
                //Use 32k banks
                let pkg_32k_bank_addr = (pkg_bank & 0xFE).mul(0x8000).add(addr as u32);
                self.pkg_rom[pkg_32k_bank_addr as usize]
            }
            2 => {
                //First bank fixed, second bank variable
                if addr < 0x4000 {
                    self.pkg_rom[addr as usize]
                } else {
                    let bank_addr = addr.wrapping_sub(0x4000) as u32;
                    let rom_addr = (pkg_bank * 0x4000).add(bank_addr);
                    self.pkg_rom[rom_addr as usize]
                }
            }
            3 => {
                //First bank fixed, second bank variable
                if addr < 0x4000 {
                    let rom_addr = (pkg_bank * 0x4000).add(addr as u32);
                    self.pkg_rom[rom_addr as usize]
                } else {
                    let last_bank = (self.pkg_rom_size / 0x4000) - 1;
                    let bank_addr = addr.wrapping_sub(0x4000) as usize;
                    let rom_addr = (last_bank * 0x4000).add(bank_addr);
                    self.pkg_rom[rom_addr]
                }
            }
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
        let chr_mode = (self.control_register & 0x10) >> 4;

        // 8k mode
        if chr_mode == 0 {
            let chr_bank = (self.chr0_bank & 0x1E) as usize;
            let chr_addr = chr_bank.mul(0x2000).add(addr as usize);
            return self.chr_rom[chr_addr];
        }

        let current_bank_reg = if addr < 0x1000 {
            self.chr0_bank
        } else {
            self.chr1_bank
        };

        let chr_bank = current_bank_reg as usize;
        let chr_addr = chr_bank.mul(0x1000).add(addr as usize);
        return self.chr_rom[chr_addr];
    }

    fn write_chr_byte(&mut self, addr: u16, value: u8) {
        let chr_mode = (self.control_register & 0x10) >> 4;

        // 8k mode
        if chr_mode == 0 {
            let chr_bank = (self.chr0_bank & 0x1E) as usize;
            let chr_addr = chr_bank.mul(0x2000).add(addr as usize);
            self.chr_rom[chr_addr] = value;
        }

        let current_bank_reg = if addr < 0x1000 {
            self.chr0_bank
        } else {
            self.chr1_bank
        };

        let chr_bank = current_bank_reg as usize;
        let chr_addr = chr_bank.mul(0x1000).add(addr as usize);
        self.chr_rom[chr_addr] = value;
    }

    fn read_ram_byte(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write_ram_byte(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    fn get_namespace_mirroring(&mut self) -> CartridgeMirroring {
        return self.namespace_mirroring.clone();
    }
}

pub fn create_mmc1_from_rom(rom: &Vec<u8>) -> Box<impl Mapper> {
    let pkg_rom_size = rom[4] as usize * 16384;
    let chr_rom_size = rom[5] as usize * 8192;
    let flag6 = rom[6];

    let pkg_rom_start_index = 16;
    let chr_rom_start_index = pkg_rom_start_index + pkg_rom_size;

    let pkg_rom = rom[pkg_rom_start_index..pkg_rom_start_index + pkg_rom_size].to_vec();
    //The cartridge could use chr_ram...
    let chr_rom = if chr_rom_size == 0 {
        vec![0; 0xFFFFF]
    } else {
        rom[chr_rom_start_index..chr_rom_start_index + chr_rom_size].to_vec()
    };

    let namespace_mirroring = if flag6 & 0x1 == 0 {
        CartridgeMirroring::HORIZONTAL
    } else {
        CartridgeMirroring::VERTICAL
    };

    return Box::new(MMC1 {
        pkg_rom,
        pkg_rom_size,
        chr_rom,
        chr_rom_size,
        namespace_mirroring,
        mapper: 1,
        ram: vec![0; 0x2000],
        shift_register: 0x10,
        current_shift_loc: 0,
        control_register: 0,
        pkg_bank: 0,
        chr0_bank: 0,
        chr1_bank: 0,
    });
}
