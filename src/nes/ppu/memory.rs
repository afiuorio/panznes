use crate::cartridge::CartridgeMirroring;
use crate::Nes;

impl Nes {
    pub(crate) fn read_ppu_byte(&mut self, addr: u16) -> u8 {
        let read_addr = addr & 0x3FFF;

        return match read_addr {
            //CHR_ROM
            //TODO This depends on cart mapping
            0..=0x1FFF => self.cartridge.read_chr_byte(read_addr),
            //Nametables
            0x2000..=0x2FFF => {
                let ppu_addr = match self.cartridge.get_namespace_mirroring() {
                    CartridgeMirroring::HORIZONTAL => read_addr & 0xFBFF,
                    CartridgeMirroring::VERTICAL => read_addr & 0xF7FF,
                };

                self.ppu_memory[ppu_addr.wrapping_sub(0x2000) as usize]
            }
            //Mirror of 0x2000 .. 0x2EFF
            0x3000..=0x3EFF => {
                let ppu_addr = read_addr - 0x1000;
                self.read_ppu_byte(ppu_addr)
            }
            //Palettes area
            0x3F00..=0x3F1F => {
                let palette_addr = read_addr & 0x1F;
                match palette_addr {
                    0x10 | 0x14 | 0x18 | 0x1C => {
                        self.palettes[palette_addr.wrapping_sub(0x10) as usize]
                    }
                    _ => self.palettes[palette_addr as usize],
                }
            }
            //Mirror of 3F00 .. 0x3F1F
            0x3F20..=0x3FFF => {
                let ppu_addr = 0x3F00 + (read_addr & 0x1F);
                self.read_ppu_byte(ppu_addr)
            }
            _ => panic!("PPU bus is 14 bit long"),
        };
    }

    pub(crate) fn write_ppu_byte(&mut self, addr: u16, value: u8) {
        //PPU bus is 14 bit long, so every address in 0x4000..0xFFFF is mapped to 0x0000..0x3FFF
        let write_addr = addr & 0x3FFF;
        match write_addr {
            0..=0x1FFF => {
                self.cartridge.write_chr_byte(write_addr, value);
            }
            //Nametables
            //TODO this depends on cart mirroring!
            0x2000..=0x2FFF => {
                let ppu_addr = match self.cartridge.get_namespace_mirroring() {
                    CartridgeMirroring::HORIZONTAL => write_addr & 0xFBFF,
                    CartridgeMirroring::VERTICAL => write_addr & 0xF7FF,
                };

                self.ppu_memory[ppu_addr.wrapping_sub(0x2000) as usize] = value;
            }
            //Mirror of 0x2000 .. 0x2EFF
            0x3000..=0x3EFF => {
                let ppu_addr = write_addr - 0x1000;
                self.write_ppu_byte(ppu_addr, value);
            }
            //Palettes area
            0x3F00..=0x3F1F => {
                let palette_addr = write_addr & 0x1F;
                match palette_addr {
                    0x10 | 0x14 | 0x18 | 0x1C => {
                        self.palettes[palette_addr as usize] = value;
                        self.palettes[palette_addr.wrapping_sub(0x10) as usize] = value;
                    }
                    _ => {
                        self.palettes[palette_addr as usize] = value;
                    }
                }

                self.palettes[palette_addr as usize] = value;
            }
            //Mirror of 3F00 .. 0x3F1F
            0x3F20..=0x3FFF => {
                let ppu_addr = 0x3F00 + (write_addr & 0x1F);
                self.write_ppu_byte(ppu_addr, value);
            }
            //This should never happens...
            _ => panic!("PPU bus is 14 bit long"),
        };
    }
}
