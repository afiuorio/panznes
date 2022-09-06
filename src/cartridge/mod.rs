pub struct Cartridge {
    pub pkg_rom: Vec<u8>,
    pub pkg_rom_size: usize,
    pub chr_rom: Vec<u8>,
    pub chr_rom_size: usize,
    pub namespace_mirroring: CartridgeMirroring,
}

#[derive(Debug)]
pub enum CartridgeMirroring {
    HORIZONTAL,
    VERTICAL,
}

impl Cartridge {
    pub fn from_ines(rom: &Vec<u8>) -> Cartridge {
        //TODO check header
        let pkg_rom_size = rom[4] as usize * 16384;
        let chr_rom_size = rom[5] as usize * 8192;

        println!("file size {}", rom.len());
        println!(
            "pkg_rom_size {}, chr_rom_size {}",
            pkg_rom_size, chr_rom_size
        );

        let pkg_rom_start_index = 16;
        let chr_rom_start_index = pkg_rom_start_index + pkg_rom_size;

        let pkg_rom = rom[pkg_rom_start_index..pkg_rom_start_index + pkg_rom_size].to_vec();
        let chr_rom = rom[chr_rom_start_index..chr_rom_start_index + chr_rom_size].to_vec();

        let namespace_mirroring = if rom[6] & 0x1 == 0 {
            CartridgeMirroring::HORIZONTAL
        } else {
            CartridgeMirroring::VERTICAL
        };

        println!("namespace_mirroring {:?}", namespace_mirroring);
        Cartridge {
            pkg_rom,
            pkg_rom_size,
            chr_rom,
            chr_rom_size,
            namespace_mirroring,
        }
    }
}
