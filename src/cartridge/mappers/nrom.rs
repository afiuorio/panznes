use crate::cartridge::{CartridgeMirroring, Mapper};

pub struct NROM {
    pub pkg_rom: Vec<u8>,
    pub pkg_rom_size: usize,
    pub chr_rom: Vec<u8>,
    pub chr_rom_size: usize,
    pub namespace_mirroring: CartridgeMirroring,
}

impl Mapper for NROM {
    fn read_pkg_byte(&mut self, addr: u16) -> u8 {
        let pkg_rom_size = self.pkg_rom_size;

        //If ROM only has 1 page, it's mirrored into 0xC000 - 0xFFFF
        let rom_addr = addr
            % if pkg_rom_size == 0x4000 {
                0x4000
            } else {
                0x8000
            };
        return *self.pkg_rom.get(rom_addr as usize).unwrap();
    }

    fn write_pkg_byte(&mut self, _addr: u16, _value: u8) {}

    fn read_chr_byte(&mut self, addr: u16) -> u8 {
        return self.chr_rom[addr as usize];
    }

    fn write_chr_byte(&mut self, _addr: u16, _value: u8) {}

    fn read_ram_byte(&mut self, _addr: u16) -> u8 {
        0
    }

    fn write_ram_byte(&mut self, _addr: u16, _value: u8) {}

    fn get_namespace_mirroring(&mut self) -> CartridgeMirroring {
        return self.namespace_mirroring.clone();
    }
}

pub fn create_nrom_from_rom(rom: &Vec<u8>) -> Box<impl Mapper> {
    let pkg_rom_size = rom[4] as usize * 16384;
    let chr_rom_size = rom[5] as usize * 8192;
    let flag6 = rom[6];

    let pkg_rom_start_index = 16;
    let chr_rom_start_index = pkg_rom_start_index + pkg_rom_size;

    let pkg_rom = rom[pkg_rom_start_index..pkg_rom_start_index + pkg_rom_size].to_vec();
    //The cartridge could use chr_ram...
    let chr_rom = rom[chr_rom_start_index..chr_rom_start_index + chr_rom_size].to_vec();

    let namespace_mirroring = if flag6 & 0x1 == 0 {
        CartridgeMirroring::HORIZONTAL
    } else {
        CartridgeMirroring::VERTICAL
    };

    Box::new(NROM {
        pkg_rom,
        pkg_rom_size,
        chr_rom,
        chr_rom_size,
        namespace_mirroring,
    })
}
