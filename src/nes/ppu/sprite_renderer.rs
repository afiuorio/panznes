use crate::nes::ppu::registers::{PPUCTRL, PPUSTATUS};
use crate::Nes;
use std::ops::Mul;

impl Nes {
    pub(super) fn render_sprites(&mut self, current_scanline: u16) {
        // PPU has a 32 byte memory that works as a secondary OAM that contains
        // the 8 sprites for this line. We start by doing a linear search

        let mut secondary_oam: [u8; 8] = [0; 8];
        let mut secondary_oam_index = 0;
        //Sprites can be 8x8 or 8x16, based on PPUCTRL
        let sprite_size = self.get_sprite_size();
        for i in 0..=63 {
            // A sprite is composed by 4 bytes:
            // Byte 0 = y position - 1
            // Byte 1 = tile number
            // Byte 2 = Attributes
            // Byte 3 = x position

            let current_sprite_y_index = i * 4;
            let y_pos = (self.oam_ram[current_sprite_y_index] as u16).wrapping_add(1);

            if current_scanline >= y_pos && current_scanline < (y_pos + sprite_size) {
                if secondary_oam_index == 8 {
                    //There are more than 8 sprites on this line -> sprite overflow
                    self.ppustatus.insert(PPUSTATUS::SPRITE_OVERFLOW);
                } else {
                    secondary_oam[secondary_oam_index] = current_sprite_y_index as u8;
                    secondary_oam_index += 1;
                }
            }
        }
        let number_sprites_scanline = secondary_oam_index;
        //TODO handle sprite 0 hit
        for i in 0..number_sprites_scanline {
            let sprite_index = secondary_oam[i] as usize;
            let sprite_y_position = self.oam_ram[sprite_index];
            let sprite_tile_index = self.oam_ram[sprite_index + 1];
            let sprite_attributes = self.oam_ram[sprite_index + 2];
            let sprite_x_position = self.oam_ram[sprite_index + 3];

            //If 0, the sprite is in front at the backgroud
            let sprite_priority = (sprite_attributes & 0x20) == 0;

            let pattern_table: u16 =
                // If sprite_size is 8, the pattern table depends of PPUCTRL bit
                // If sprite_size if 16, the LSB of the index indicates the table
                if sprite_size == 8 {
                    self.get_active_pattern_table(PPUCTRL::SPRITE_PATTERN_TABLE)
                } else {
                    (u16::from(sprite_tile_index) & 0x1) * 0x1000
                };

            let tile_address = pattern_table.wrapping_add(u16::from(sprite_tile_index) << 4);

            let current_tile_row = current_scanline
                .wrapping_sub(u16::from(sprite_y_position))
                .wrapping_sub(1);

            let tile_row_address = tile_address.wrapping_add(if sprite_attributes & 0x80 == 0 {
                current_tile_row
            } else {
                (sprite_size - 1) - current_tile_row
            });
            let tile_first_plane = self.read_ppu_byte(tile_row_address);
            let tile_second_plane = self.read_ppu_byte(tile_row_address.wrapping_add(8));

            let palette_msb = sprite_attributes & 0x3;

            for current_pixel in 0..=7 {
                //To handle pixel mirroring
                let pixel_to_render = if sprite_attributes & 0x40 == 0 {
                    current_pixel
                } else {
                    7 - current_pixel
                };

                let palette_lsb = Nes::get_tile_pixel_from_planes(
                    tile_first_plane,
                    tile_second_plane,
                    pixel_to_render,
                );

                if palette_lsb == 0x0 {
                    //Transparent pixel -> nothing to do here
                    continue;
                }

                let palette_index = (palette_msb << 2) | (palette_lsb as u8);
                let x_pos = u16::from(sprite_x_position).wrapping_add(u16::from(current_pixel));

                if x_pos < 256 {
                    let index_screen = u16::from(current_scanline)
                        .mul(256)
                        .wrapping_add(u16::from(x_pos));

                    let has_background = self.background_hit_flag[index_screen as usize];

                    if sprite_index == 0 && has_background {
                        self.ppustatus.insert(PPUSTATUS::SPRITE_0_HIT);
                    }

                    //I must draw the pixel only if there isn't a background pixel or if the sprite is in front background
                    if !has_background || sprite_priority {
                        self.render_pixel(
                            0x3F10 + u16::from(palette_index),
                            x_pos as u8,
                            current_scanline as u8,
                        );
                    }
                }
            }
        }
    }

    fn get_tile_pixel_from_planes(
        tile_first_plane: u8,
        tile_second_plane: u8,
        which_pixel: u8,
    ) -> u8 {
        (tile_first_plane >> (7 - which_pixel) & 0x1)
            | ((tile_second_plane >> (7 - which_pixel) & 0x1) << 1)
    }

    fn get_sprite_size(&self) -> u16 {
        if self.ppuctrl.contains(PPUCTRL::SPRITE_SIZE_16) {
            16
        } else {
            8
        }
    }
}
