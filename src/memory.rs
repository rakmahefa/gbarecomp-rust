use std::error::Error;
use std::fmt;

const BIOS_SIZE: usize = 0x4000;
const EWRAM_SIZE: usize = 0x40000;
const IWRAM_SIZE: usize = 0x8000;
const IO_SIZE: usize = 0x400;
const PALETTE_SIZE: usize = 0x400;
const VRAM_SIZE: usize = 0x18000;
const OAM_SIZE: usize = 0x400;
const SAVE_SIZE: usize = 0x10000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Region {
    Bios,
    Ewram,
    Iwram,
    Io,
    Palette,
    Vram,
    Oam,
    Rom,
    Save,
}

/// A deliberately small, side-effect-free bus foundation.
///
/// This is not yet a cycle-accurate GBA bus: MMIO registers are backed by bytes
/// and do not trigger IRQ, DMA, timer, PPU, or audio behavior. Those effects
/// must be added explicitly before the bus is used to claim game compatibility.
#[derive(Clone, Debug)]
pub struct Bus {
    bios: Option<Vec<u8>>,
    ewram: Vec<u8>,
    iwram: Vec<u8>,
    io: Vec<u8>,
    palette: Vec<u8>,
    vram: Vec<u8>,
    oam: Vec<u8>,
    rom: Vec<u8>,
    save: Vec<u8>,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            bios: None,
            ewram: vec![0; EWRAM_SIZE],
            iwram: vec![0; IWRAM_SIZE],
            io: vec![0; IO_SIZE],
            palette: vec![0; PALETTE_SIZE],
            vram: vec![0; VRAM_SIZE],
            oam: vec![0; OAM_SIZE],
            rom,
            save: vec![0xFF; SAVE_SIZE],
        }
    }

    pub fn load_bios(&mut self, bytes: &[u8]) -> Result<(), BusError> {
        if bytes.len() != BIOS_SIZE {
            return Err(BusError::InvalidBiosSize {
                actual: bytes.len(),
                expected: BIOS_SIZE,
            });
        }
        self.bios = Some(bytes.to_vec());
        Ok(())
    }

    pub fn read8(&self, address: u32) -> Result<u8, BusError> {
        let (region, offset) = self.locate(address)?;
        let value = match region {
            Region::Bios => self.bios.as_ref().ok_or(BusError::BiosUnavailable)?[offset],
            Region::Ewram => self.ewram[offset],
            Region::Iwram => self.iwram[offset],
            Region::Io => self.io[offset],
            Region::Palette => self.palette[offset],
            Region::Vram => self.vram[offset],
            Region::Oam => self.oam[offset],
            Region::Rom => self.rom[offset],
            Region::Save => self.save[offset],
        };
        Ok(value)
    }

    pub fn read16(&self, address: u32) -> Result<u16, BusError> {
        let lo = self.read8(address)?;
        let hi = self.read8(address.wrapping_add(1))?;
        Ok(u16::from_le_bytes([lo, hi]))
    }

    pub fn read32(&self, address: u32) -> Result<u32, BusError> {
        let b0 = self.read8(address)?;
        let b1 = self.read8(address.wrapping_add(1))?;
        let b2 = self.read8(address.wrapping_add(2))?;
        let b3 = self.read8(address.wrapping_add(3))?;
        Ok(u32::from_le_bytes([b0, b1, b2, b3]))
    }

    pub fn write8(&mut self, address: u32, value: u8) -> Result<(), BusError> {
        let (region, offset) = self.locate(address)?;
        match region {
            Region::Bios | Region::Rom => Err(BusError::ReadOnly { address }),
            Region::Ewram => {
                self.ewram[offset] = value;
                Ok(())
            }
            Region::Iwram => {
                self.iwram[offset] = value;
                Ok(())
            }
            Region::Io => {
                self.io[offset] = value;
                Ok(())
            }
            Region::Palette => {
                self.palette[offset] = value;
                Ok(())
            }
            Region::Vram => {
                self.vram[offset] = value;
                Ok(())
            }
            Region::Oam => {
                self.oam[offset] = value;
                Ok(())
            }
            Region::Save => {
                self.save[offset] = value;
                Ok(())
            }
        }
    }

    pub fn write16(&mut self, address: u32, value: u16) -> Result<(), BusError> {
        let bytes = value.to_le_bytes();
        self.write8(address, bytes[0])?;
        self.write8(address.wrapping_add(1), bytes[1])
    }

    pub fn write32(&mut self, address: u32, value: u32) -> Result<(), BusError> {
        for (index, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.write8(address.wrapping_add(index as u32), byte)?;
        }
        Ok(())
    }

    fn locate(&self, address: u32) -> Result<(Region, usize), BusError> {
        let found = if address < BIOS_SIZE as u32 {
            Some((Region::Bios, address as usize))
        } else if (0x0200_0000..0x0300_0000).contains(&address) {
            Some((
                Region::Ewram,
                ((address - 0x0200_0000) as usize) & (EWRAM_SIZE - 1),
            ))
        } else if (0x0300_0000..0x0400_0000).contains(&address) {
            Some((
                Region::Iwram,
                ((address - 0x0300_0000) as usize) & (IWRAM_SIZE - 1),
            ))
        } else if (0x0400_0000..0x0400_0000 + IO_SIZE as u32).contains(&address) {
            Some((Region::Io, (address - 0x0400_0000) as usize))
        } else if (0x0500_0000..0x0500_0000 + PALETTE_SIZE as u32).contains(&address) {
            Some((Region::Palette, (address - 0x0500_0000) as usize))
        } else if (0x0600_0000..0x0602_0000).contains(&address) {
            let window_offset = (address - 0x0600_0000) as usize;
            let offset = if window_offset >= VRAM_SIZE {
                window_offset - 0x8000
            } else {
                window_offset
            };
            Some((Region::Vram, offset))
        } else if (0x0700_0000..0x0700_0000 + OAM_SIZE as u32).contains(&address) {
            Some((Region::Oam, (address - 0x0700_0000) as usize))
        } else if (0x0800_0000..0x0E00_0000).contains(&address) && !self.rom.is_empty() {
            let window_offset = (address - 0x0800_0000) as usize;
            Some((Region::Rom, (window_offset & 0x01FF_FFFF) % self.rom.len()))
        } else if (0x0E00_0000..0x0E01_0000).contains(&address) {
            Some((Region::Save, (address - 0x0E00_0000) as usize))
        } else {
            None
        };

        found.ok_or(BusError::UnmappedAddress { address })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BusError {
    UnmappedAddress { address: u32 },
    ReadOnly { address: u32 },
    BiosUnavailable,
    InvalidBiosSize { actual: usize, expected: usize },
}

impl fmt::Display for BusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnmappedAddress { address } => {
                write!(f, "unmapped GBA bus address 0x{address:08X}")
            }
            Self::ReadOnly { address } => {
                write!(f, "write to read-only memory at 0x{address:08X}")
            }
            Self::BiosUnavailable => write!(f, "BIOS access requested, but no BIOS was loaded"),
            Self::InvalidBiosSize { actual, expected } => {
                write!(f, "invalid BIOS size: {actual} bytes, expected {expected}")
            }
        }
    }
}

impl Error for BusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_working_ram_and_vram() {
        let mut bus = Bus::new(vec![0xAA; 0x100]);
        bus.write8(0x0200_0003, 0x42).expect("write EWRAM");
        assert_eq!(bus.read8(0x0204_0003), Ok(0x42));
        bus.write8(0x0601_0005, 0x19).expect("write VRAM");
        assert_eq!(bus.read8(0x0601_8005), Ok(0x19));
    }

    #[test]
    fn cartridge_rom_is_read_only() {
        let mut bus = Bus::new(vec![0x7F; 0x200]);
        assert_eq!(bus.read8(0x0800_0000), Ok(0x7F));
        assert_eq!(
            bus.write8(0x0800_0000, 0),
            Err(BusError::ReadOnly {
                address: 0x0800_0000
            })
        );
    }

    #[test]
    fn bios_reads_fail_until_supplied() {
        let bus = Bus::new(vec![0; 0x100]);
        assert_eq!(bus.read8(0), Err(BusError::BiosUnavailable));
    }

    #[test]
    fn unmapped_addresses_are_not_silently_zeroed() {
        let bus = Bus::new(vec![0; 0x100]);
        assert!(matches!(
            bus.read8(0x0100_0000),
            Err(BusError::UnmappedAddress { .. })
        ));
    }
}
