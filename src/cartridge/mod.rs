mod mappers;

use crate::cartridge::mappers::mmc1::create_mmc1_from_rom;
use crate::cartridge::mappers::nrom::create_nrom_from_rom;

pub trait Mapper {
    fn read_pkg_byte(&mut self, addr: u16) -> u8;
    fn write_pkg_byte(&mut self, addr: u16, value: u8);

    fn read_chr_byte(&mut self, addr: u16) -> u8;
    fn write_chr_byte(&mut self, addr: u16, value: u8);

    fn read_ram_byte(&mut self, addr: u16) -> u8;
    fn write_ram_byte(&mut self, addr: u16, value: u8);

    fn get_namespace_mirroring(&mut self) -> CartridgeMirroring;
}

#[derive(Copy, Clone)]
pub enum CartridgeMirroring {
    HORIZONTAL,
    VERTICAL,
}

pub fn from_ines(rom: &Vec<u8>) -> Box<dyn Mapper> {
    //TODO check header
    let pkg_rom_size = rom[4] as usize * 16384;
    let chr_rom_size = rom[5] as usize * 8192;
    let flag6 = rom[6];
    let flag7 = rom[7];

    let mapper = (flag7 & 0xF0) | ((flag6 & 0xF0) >> 4);

    println!("file size {}", rom.len());
    println!(
        "pkg_rom_size {}, chr_rom_size {}, mapper {}",
        pkg_rom_size, chr_rom_size, mapper
    );

    return match mapper {
        0 => create_nrom_from_rom(rom),
        1 => create_mmc1_from_rom(rom),
        _ => panic!("Unsupported mapper"),
    };
}
