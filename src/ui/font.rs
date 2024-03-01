const BG_PALETTE: *mut [u8; 512] = 0x0500_0000 as *mut [u8; 512];
const CHARBLOCK0: *mut [u32; 8] = 0x0600_0000 as *mut [u32; 8];

/// Loads the font into VRAM.
pub(super) fn load() {
    // Load palettes.
    unsafe {BG_PALETTE.write_volatile(include_aligned_bytes!("../../data/font.pal").0);}

    // Load font tiles.
    let mut charblock = CHARBLOCK0;
    for character in include_aligned_bytes!("../../data/font8x8_basic.fnt").0.chunks(8) {
        let mut converted = [0u32; 8];
        for (index, byte) in character.iter().enumerate() {
            for bit in (0..8).rev() {
                match (byte >> bit) & 1 {
                    0 => {},
                _ => {converted[index] |= 1}
                }
                converted[index] <<= 4;
            }
        }
        unsafe {charblock.write_volatile(converted); charblock = charblock.add(1);}
    }
}
