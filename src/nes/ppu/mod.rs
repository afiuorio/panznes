use crate::nes::ppu::registers::{PPUCTRL, PPUMASK, PPUSTATUS};
use crate::nes::Nes;

mod background_renderer;
mod memory;
mod palette;
pub(crate) mod registers;
mod sprite_renderer;
mod utilities;

impl<'a> Nes<'a> {
    pub fn execute_ppu(&mut self, cpu_cycles: u32) {
        //A CPU tick is equal to 3 PPU ticks...
        let ppu_cycles = cpu_cycles * 3;
        let clock_current_scanline = self.clock_current_scanline.wrapping_add(ppu_cycles);

        self.clock_current_scanline = clock_current_scanline;
        if clock_current_scanline >= 341 {
            match self.current_scanline {
                0..=239 => {
                    //TODO maybe I should emulate the PPU clock by clock?
                    if self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED) {
                        self.render_background(self.current_scanline as u16);
                    }
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED) {
                        self.render_sprites(self.current_scanline as u16);
                    }
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED)
                        || self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED)
                    {
                        self.increment_y();
                        self.copy_horizontal_position();
                    }
                    self.current_scanline += 1;
                    //TODO fix for super mario bros!
                    /*if self.current_scanline == 32 {
                        self.ppustatus.insert(PPUSTATUS::SPRITE_0_HIT);
                    }*/
                }
                240 => {
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED)
                        || self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED)
                    {
                        self.increment_y();
                        self.copy_horizontal_position();
                    }
                    self.current_scanline += 1;
                }
                241 => {
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED)
                        || self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED)
                    {
                        self.increment_y();
                        self.copy_horizontal_position();
                    }
                    //set VBlank, check if NMI is active and raise
                    let mut ppustatus = self.ppustatus;
                    ppustatus.insert(PPUSTATUS::V_BLANK);
                    ppustatus.remove(PPUSTATUS::SPRITE_0_HIT);
                    self.ppustatus = ppustatus;

                    let ppuctrl = self.ppuctrl;
                    if ppuctrl.contains(PPUCTRL::NMI_ENABLED) {
                        self.raise_nmi();
                    }
                    self.current_scanline += 1;
                }
                //VBlank = do nothing
                242..=260 => {
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED)
                        || self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED)
                    {
                        self.increment_y();
                        self.copy_horizontal_position();
                    }
                    self.current_scanline += 1;
                }
                //Finished scanlines, reset
                261 => {
                    if self.ppumask.contains(PPUMASK::SPRITES_ENABLED)
                        || self.ppumask.contains(PPUMASK::BACKGROUND_ENABLED)
                    {
                        self.increment_y();
                        self.copy_horizontal_position();
                        //Must copy everything to from t to v
                        // v: GHIA.BC DEF..... <- t: GHIA.BC DEF.....

                        let fine_y = self.t_vram_addr & 0x7B00;
                        let coarse_y = self.t_vram_addr & 0x03E0;
                        self.vram_addr &= 0x041F;
                        self.vram_addr = self.vram_addr | fine_y | coarse_y;
                    }

                    self.current_scanline = 0;
                }
                _ => {
                    //TODO panic
                    panic!("Bad scanline")
                }
            }
            self.clock_current_scanline -= 341;
        }
    }

    fn copy_horizontal_position(&mut self) {
        //v: ....A.. ...BCDEF <- t: ....A.. ...BCDEF
        let coarse_x = self.t_vram_addr & 0x1F;
        let nametable = self.t_vram_addr & 0x0400;

        self.vram_addr &= !0x041F;
        self.vram_addr = self.vram_addr | coarse_x | nametable;
    }
    fn increment_y(&mut self) {
        if (self.vram_addr & 0x7000) != 0x7000
        // if fine Y < 7
        {
            self.vram_addr += 0x1000
        }
        // increment fine Y
        else {
            self.vram_addr &= !0x7000; // fine Y = 0
            let mut y = (self.vram_addr & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0; // coarse Y = 0
                self.vram_addr ^= 0x0800
            }
            // switch vertical nametable
            else if y == 31 {
                y = 0;
            }
            // coarse Y = 0, nametable not switched
            else {
                y += 1
            } // increment coarse Y
            self.vram_addr = (self.vram_addr & !0x03E0) | (y << 5); // put coarse Y back into v
        }
    }
}
