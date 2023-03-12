use crate::nes::ppu::registers::{PPUMASK, PPUSTATUS};
use crate::nes::Nes;

mod background_renderer;
mod memory;
mod palette;
pub(crate) mod registers;
mod sprite_renderer;
mod utilities;

impl Nes {
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
                    self.current_scanline += 1;
                }
                240 => {
                    //set VBlank, check if NMI is active and raise
                    let mut ppustatus = self.ppustatus;
                    ppustatus.insert(PPUSTATUS::V_BLANK);
                    ppustatus.remove(PPUSTATUS::SPRITE_0_HIT);
                    self.ppustatus = ppustatus;

                    self.ppustatus.insert(PPUSTATUS::V_BLANK);
                    self.current_scanline += 1;
                }
                //VBlank = do nothing
                241..=260 => {
                    self.current_scanline += 1;
                }
                //Finished scanlines, reset
                261 => {
                    self.ppu_v = (self.ppu_v & 0x041F) | (u16::from(self.ppu_t & 0xFBE0));
                    self.current_scanline = 0;
                    self.ppustatus.remove(PPUSTATUS::V_BLANK);
                }
                _ => {
                    //TODO panic
                    panic!("Bad scanline")
                }
            }
            self.clock_current_scanline -= 341;
        }
    }
}
