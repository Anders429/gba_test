#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(crate) struct RegisterRamReset(u8);

impl RegisterRamReset {
    pub(crate) const fn new() -> Self {
        Self(0)
    }

    pub(crate) const fn with_palette(self) -> Self {
        Self(self.0 | 0b0000_0100)
    }

    pub(crate) const fn with_vram(self) -> Self {
        Self(self.0 | 0b0000_1000)
    }

    pub(crate) const fn with_oam(self) -> Self {
        Self(self.0 | 0b0001_0000)
    }

    pub(crate) const fn with_sio_registers(self) -> Self {
        Self(self.0 | 0b0010_0000)
    }

    pub(crate) const fn with_sound_registers(self) -> Self {
        Self(self.0 | 0b0100_0000)
    }

    pub(crate) const fn with_other_registers(self) -> Self {
        Self(self.0 | 0b1000_0000)
    }

    pub(crate) const fn to_u8(self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum HaltControl {
    Halt = 0x00,
}

#[cfg(test)]
mod tests {
    use super::RegisterRamReset;
    use gba_test::test;

    #[test]
    fn register_ram_reset_none() {
        assert_eq!(RegisterRamReset::new().to_u8(), 0);
    }

    #[test]
    fn register_ram_reset_palette() {
        assert_eq!(RegisterRamReset::new().with_palette().to_u8(), 4);
    }

    #[test]
    fn register_ram_reset_vram() {
        assert_eq!(RegisterRamReset::new().with_vram().to_u8(), 8);
    }

    #[test]
    fn register_ram_reset_oam() {
        assert_eq!(RegisterRamReset::new().with_oam().to_u8(), 16);
    }

    #[test]
    fn register_ram_reset_sio_registers() {
        assert_eq!(RegisterRamReset::new().with_sio_registers().to_u8(), 32);
    }

    #[test]
    fn register_ram_reset_sound_registers() {
        assert_eq!(RegisterRamReset::new().with_sound_registers().to_u8(), 64);
    }

    #[test]
    fn register_ram_reset_other_registers() {
        assert_eq!(RegisterRamReset::new().with_other_registers().to_u8(), 128);
    }

    #[test]
    fn register_ram_reset_all() {
        assert_eq!(
            RegisterRamReset::new()
                .with_palette()
                .with_vram()
                .with_oam()
                .with_sio_registers()
                .with_sound_registers()
                .with_other_registers()
                .to_u8(),
            0xFC
        );
    }
}
