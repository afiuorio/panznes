use crate::nes::ppu::registers::{PPUCTRL, PPUMASK};
use crate::nes::Nes;

mod controller;

impl<'a> Nes<'a> {
    pub(crate) fn read_cpu_byte(&mut self, addr: u16) -> u8 {
        return match addr {
            0x0000..=0x1FFF => {
                //Ram is mirrored in this space
                let ram_addr = (addr % 0x800) as usize;
                self.cpu_memory[ram_addr]
            }
            0x2000..=0x3FFF => {
                //This area is a mirror for 0x2000 - 0x2007
                let ppu_io_addr = (addr - 0x2000) % 0x8;
                return match ppu_io_addr {
                    //todo should panic?
                    0 | 1 | 3 | 5 | 6 => 0,
                    2 => {
                        self.ppu_second_write = false;
                        self.ppustatus.bits()
                    }
                    4 => {
                        let oam_addr = self.oam_addr;
                        self.oam_ram[oam_addr as usize]
                    }
                    7 => {
                        let old_data = self.vram_data;
                        let vram_addr = self.vram_addr;
                        let value = self.read_ppu_byte(vram_addr);
                        //Increase vram_addr based on VRAM_INCREMENT bit
                        let horizontal_increment = self.ppuctrl.contains(PPUCTRL::VRAM_INCREMENT);
                        self.vram_addr = vram_addr + if horizontal_increment { 32 } else { 1 };
                        self.vram_data = value;
                        if vram_addr <= 0x3EFF {
                            old_data
                        } else {
                            value
                        }
                    }
                    _ => 0,
                };
            }
            //APU I/O
            0x4000..=0x4013 => 0,
            //DMA request, only write
            0x4014 => 0,
            //APU status
            0x4015 => 0,
            //Joypad 1 and strobing
            0x4016 => {
                return match self.first_port_strobing_index {
                    0..=7 => {
                        let is_pressed = self.controller_first_port[self.first_port_strobing_index];
                        self.first_port_strobing_index += 1;
                        return if is_pressed { 0x1 } else { 0x0 };
                    }
                    _ => 0,
                };
            }
            //Joypad 2
            0x4017 => 0,
            //Used only on debug, disabled on commercial NES
            0x4018..=0x401F => 0,
            //Expansion ROM (only certain mappers)
            0x4020..=0x5FFF => 0,
            //Cart RAM
            0x6000..=0x7FFF => 0,
            0x8000..=0xFFFF => {
                //TODO handle cart mapper
                let cartridge = self.cartridge.unwrap();
                let pkg_rom = &cartridge.pkg_rom;
                let pkg_rom_size = cartridge.pkg_rom_size;

                //If ROM only has 1 page, it's mirrored into 0xC000 - 0xFFFF
                let rom_addr = (addr - 0x8000)
                    % if pkg_rom_size == 0x4000 {
                        0x4000
                    } else {
                        0x8000
                    };
                *pkg_rom.get(rom_addr as usize).unwrap()
            }
        };
    }

    pub(crate) fn write_cpu_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                //Ram is mirrored in this space
                let ram_addr = (addr % 0x800) as usize;

                self.cpu_memory[ram_addr] = value;
            }
            0x2000..=0x3FFF => {
                //This area is a mirror for 0x2000 - 0x2007
                let ppu_io_addr = (addr - 0x2000) % 0x8;
                return match ppu_io_addr {
                    0 => {
                        self.ppuctrl = PPUCTRL::from_bits_truncate(value);
                        //TODO use bitfield!!
                        self.t_vram_addr =
                            (self.t_vram_addr & 0xF3FF) | ((u16::from(value) & 0x3) << 10);
                    }
                    1 => {
                        self.ppumask = PPUMASK::from_bits_truncate(value);
                    }
                    //OAM ADDR
                    3 => {
                        self.oam_addr = value;
                    }
                    //OAM DATA
                    4 => {
                        let addr = self.oam_addr;
                        self.oam_ram[addr as usize] = value;
                        self.oam_addr = addr.wrapping_add(1);
                    }
                    5 => {
                        let second_write = self.ppu_second_write;
                        if second_write {
                            // t: FGH..AB CDE..... <- d: ABCDEFGH
                            let y_part = (u16::from(value) & 0xF8) << 2;
                            let head = (u16::from(value) & 0x7) << 12;

                            self.t_vram_addr &= 0x0C1F;
                            self.t_vram_addr = self.t_vram_addr | head | y_part;

                            self.vertical_scroll_origin =
                                if value > 0xf0 { value - 0xf0 } else { value }
                        } else {
                            self.ppu_fine_x_scroll = value & 0x7;
                            self.t_vram_addr &= !0x7;
                            self.t_vram_addr = self.t_vram_addr | (u16::from(value) >> 3);
                            self.horizontal_scroll_origin = value
                        }
                        self.ppu_second_write = !second_write
                    }
                    6 => {
                        let second_write = self.ppu_second_write;
                        if second_write {
                            self.t_vram_addr &= 0xFF00;
                            self.t_vram_addr = self.t_vram_addr | u16::from(value);
                            self.vram_addr = self.t_vram_addr;
                        } else {
                            /* t: .CDEFGH ........ <- d: ..CDEFGH
                                      <unused>     <- d: AB......
                               t: Z...... ........ <- 0 (bit Z is cleared)
                               w:                  <- 1
                            */

                            let c = (u16::from(value) & 0x3F) << 8;
                            self.t_vram_addr &= 0x00FF;
                            self.t_vram_addr = self.t_vram_addr | c;
                        };
                        self.ppu_second_write = !second_write;
                    }
                    7 => {
                        let vram_addr = self.vram_addr & 0x3FFF;
                        self.write_ppu_byte(vram_addr, value);
                        //Increase vram_addr based on VRAM_INCREMENT bit
                        let horizontal_increment = self.ppuctrl.contains(PPUCTRL::VRAM_INCREMENT);
                        self.vram_addr =
                            vram_addr.wrapping_add(if horizontal_increment { 32 } else { 1 });
                    }
                    2 => {
                        //NOP
                    }
                    _ => {}
                };
            }
            //APU I/O
            0x4000..=0x4013 => {}
            //DMA request
            0x4014 => {
                self.request_dma = true;
                self.dma_src = u16::from(value) << 8;
            }
            //APU status
            0x4015 => {}
            //Joypad 1 and strobing
            0x4016 => {
                self.first_port_strobing = value & 0x1 != 0;
                self.first_port_strobing_index = 0;
            }
            //Joypad 2
            0x4017 => {}
            //Used only on debug, disabled on commercial NES
            0x4018..=0x401F => {}
            //Expansion ROM (only certain mappers
            0x4020..=0x5FFF => {}
            //Cart RAM
            0x6000..=0x7FFF => unsafe {
                self.serial[addr as usize - 0x6000] = value;
                //println!("{}", String::from_utf8_unchecked(self.serial[0..0x40].to_vec()));
            },
            //PGR_ROM
            0x8000..=0xFFFF => {
                //TODO this should be different for each cart
            }
        }
    }

    pub(crate) fn raise_nmi(&mut self) {
        //TODO handle better for clock accuracy
        self.raised_nmi = true;
    }

    pub(crate) fn dma_transfert(&mut self) {
        let dma_src = self.dma_src;
        for i in 0..=0xFF {
            self.oam_ram[i] = self.read_cpu_byte(dma_src.wrapping_add(i as u16));
        }
    }
}
