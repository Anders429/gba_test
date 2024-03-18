use core::{fmt, fmt::Write};

pub(super) struct Cursor {
    cursor: *mut u16,
    palette: u8,
}

impl Cursor {
    pub(super) unsafe fn new(cursor: *mut u16) -> Self {
        Self { cursor, palette: 0 }
    }

    pub(super) fn set_palette(&mut self, palette: u8) {
        self.palette = palette;
    }
}

impl Write for Cursor {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for character in s.chars() {
            let ascii: u32 = character.into();
            // Only account for basic characters.
            if ascii == u32::from('\n') {
                // Move to the next line.
                self.cursor = (((self.cursor as usize / 0x40) + 1) * 0x40) as *mut u16;
            } else if ascii < 128 {
                unsafe {
                    self.cursor
                        .write_volatile((ascii | ((self.palette as u32) << 12)) as u16);
                    self.cursor = self.cursor.add(1);
                    // Don't write past the view of the screen.
                    if (self.cursor as usize) % 0x40 > 0x3a {
                        self.cursor = (((self.cursor as usize / 0x40) + 1) * 0x40) as *mut u16;
                    }
                }
            }
        }

        Ok(())
    }
}
