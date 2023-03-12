use crate::cartridge::Cartridge;
use crate::nes::ppu::registers::{PPUCTRL, PPUMASK, PPUSTATUS};
use bitflags::bitflags;

mod cpu;
mod ppu;
mod system_bus;

enum Interrupt {
    NMI,
    BREAK,
}

bitflags! {
    struct FlagRegister: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const IRQ_DISABLE       = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const UNUSED            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

pub struct Nes {
    // Registers

    //accumulator
    a: u8,
    //Index register x
    x: u8,
    //Index register y
    y: u8,
    //Stack pointer
    stack_ptr: u8,
    //P (status)
    flag: FlagRegister,
    //Program Counter
    prog_counter: u16,

    //Main WRAM
    cpu_memory: [u8; 0x800],
    cartridge: Box<dyn Cartridge>,

    ppuctrl: PPUCTRL,
    ppumask: PPUMASK,
    ppustatus: PPUSTATUS,
    oam_addr: u8,
    oam_ram: [u8; 0x100],

    ppu_second_write: bool,

    vram_data: u8,

    ppu_memory: [u8; 0x800],

    request_dma: bool,
    dma_src: u16,

    current_scanline: u32,
    clock_current_scanline: u32,

    palettes: [u8; 0x20],

    pub screen: [u32; 256 * 240],
    background_hit_flag: [bool; 256 * 240],

    controller_first_port: [bool; 8],
    first_port_strobing: bool,
    first_port_strobing_index: usize,

    ppu_v: u16,
    ppu_t: u16,
    ppu_x: u8,
}

pub enum NesControllerButton {
    A = 0,
    B,
    SELECT,
    START,
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Nes {
    pub fn create_nes(cartridge: Box<dyn Cartridge>) -> Nes {
        Nes {
            a: 0,
            x: 0,
            y: 0,
            stack_ptr: 0,
            flag: FlagRegister::from_bits_truncate(0),
            prog_counter: 0,

            //Main WRAM
            cpu_memory: [0x0; 0x800],
            cartridge,

            ppuctrl: PPUCTRL::from_bits_truncate(0),
            ppumask: PPUMASK::from_bits_truncate(0),
            ppustatus: PPUSTATUS::from_bits_truncate(0),
            oam_addr: 0,
            oam_ram: [0x0; 0x100],

            ppu_second_write: false,

            vram_data: 0x0,
            ppu_memory: [0x0; 0x800],

            request_dma: false,
            dma_src: 0x0,

            current_scanline: 0,
            clock_current_scanline: 0,

            palettes: [0x0; 0x20],
            screen: [0x0; 256 * 240],
            background_hit_flag: [false; 256 * 240],
            controller_first_port: [false; 8],
            first_port_strobing: false,
            first_port_strobing_index: 0,

            ppu_v: 0,
            ppu_t: 0,
            ppu_x: 0,
        }
    }
}
