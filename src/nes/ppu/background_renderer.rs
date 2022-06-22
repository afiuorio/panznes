use crate::nes::ppu::palette::NES_PALETTE;
use crate::Nes;
use std::ops::Mul;

impl<'a> Nes<'a> {
    pub(super) fn render_background(&mut self, current_scanline: u16) {
        // First, let's retrieve the tile row we need to render
        // Since we have scrolling, it's possible that the current line
        // is on the bottom nametable (mod 0x2000)

        //Now we can have the address (in the nametable) for the top-left tile of our screen

        //Which row of each tile we need to draw?
        let tile_offset_y = (self.vram_addr & 0x7000) >> 12;

        //How much we are x-shifted during tile drawing?
        let tile_x_offset = self.ppu_fine_x_scroll;

        let mut current_pixel: u16 = 0;

        // I need to retrieve 33 tiles for this row starting from the top-left one.
        // A row is 32 tiles, however it's possible to see 33 tiles because of scrolling
        while current_pixel <= 0xFF {
            //Let's take the tile address in the nametable
            let nametable_tile_address = 0x2000 | (self.vram_addr & 0x0FFF);

            //And using that, we can retrive the tile index in the tile pattern table
            let tile_index = self.read_ppu_byte(nametable_tile_address);

            let (tile_first_plane, tile_second_plane) =
                self.retrieve_tile_row(tile_index, tile_offset_y as u8);

            let palette_msb = self.retrieve_attribute_table_value(nametable_tile_address);

            let pixels = Nes::get_tile_row_pixels(palette_msb, tile_first_plane, tile_second_plane);

            let pixels_to_draw = if current_pixel == 0 {
                (tile_x_offset..=7)
            } else if current_pixel + 8 > 0xFF {
                (0..=0xFF - (current_pixel as u8))
            } else {
                (0..=7)
            };

            for i in pixels_to_draw {
                //If using palette 0 the pixel is transparent -> use default color
                //TODO maybe this part should be done by
                let palette_address = if pixels[i as usize] & 0x3 == 0 {
                    0x3F00
                } else {
                    0x3F00 + u16::from(pixels[i as usize])
                };

                
                self.background_collision[ ((current_scanline * 256) + current_pixel)as usize] 
                    = pixels[i as usize] & 0x3 != 0;

                self.render_pixel(palette_address, current_pixel as u8, current_scanline as u8);
                current_pixel = current_pixel.wrapping_add(1);
            }

            if current_pixel <= 0xFF {
                if (self.vram_addr & 0x001F) == 31 {
                    // if coarse X == 31
                    self.vram_addr &= !0x001F; // coarse X = 0
                    self.vram_addr ^= 0x0400;
                }
                // switch horizontal nametable
                else {
                    self.vram_addr += 1
                } // increment coarse X
            }
        }
    }

    fn retrieve_attribute_table_value(&mut self, nametable_tile_address: u16) -> u8 {
        let nametable_index = nametable_tile_address & 0x3FF;

        let attribute_table_address = (nametable_tile_address & 0xFC00) + 0x3C0;

        let attribute_table_index = (nametable_index >> 7) << 3 | ((nametable_index & 0x1F) >> 2);

        let attribute_table_entry =
            self.read_ppu_byte(attribute_table_address.wrapping_add(attribute_table_index));
        let internal_group_index = ((nametable_index & 0x40) >> 5) | ((nametable_index & 0x2) >> 1);

        return (attribute_table_entry >> (internal_group_index * 2)) & 0x3;
    }

    fn get_main_nametable(&mut self) -> u16 {
        return match self.ppuctrl.bits() & 0x3 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("Error nametable"),
        };
    }

    fn get_tile_row_pixels(
        palette_msb: u8,
        tile_first_plane: u8,
        tile_second_plane: u8,
    ) -> [u8; 8] {
        let mut pixels = [0x0 as u8; 8];

        for i in 0..=7 {
            let palette_lsb =
                (tile_first_plane >> (7 - i) & 0x1) | ((tile_second_plane >> (7 - i) & 0x1) << 1);
            pixels[i] = (palette_msb << 2) | (palette_lsb as u8);
        }
        return pixels;
    }
}
