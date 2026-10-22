//! Owned cartridge image and static CPU-address-to-ROM-offset mapping.
//!
//! These operations only inspect bytes and address windows. They do not
//! decode, interpret, execute, or compile guest instructions.

use std::{error::Error, fmt};

use super::{GbaHeader, RomHeaderError};

/// Maximum Game Pak ROM image size modeled by this initial implementation: 32 MiB.
pub const MAX_GBA_ROM_SIZE: usize = 0x0200_0000;

/// One of the three Game Pak ROM windows visible on the GBA CPU bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomWindow {
    /// Window beginning at 0x08000000, wait-state setting zero.
    WaitState0,
    /// Window beginning at 0x0A000000, wait-state setting one.
    WaitState1,
    /// Window beginning at 0x0C000000, wait-state setting two.
    WaitState2,
}

impl RomWindow {
    /// Return the inclusive start address of this 32 MiB Game Pak window.
    pub const fn base_address(self) -> u32 {
        match self {
            Self::WaitState0 => 0x0800_0000,
            Self::WaitState1 => 0x0A00_0000,
            Self::WaitState2 => 0x0C00_0000,
        }
    }

    /// Return the wait-state window number (0, 1, or 2).
    pub const fn wait_state(self) -> u8 {
        match self {
            Self::WaitState0 => 0,
            Self::WaitState1 => 1,
            Self::WaitState2 => 2,
        }
    }
}

/// A successful mapping from a guest CPU address to a byte in the ROM file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomAddress {
    /// Game Pak window through which the guest address was accessed.
    pub window: RomWindow,
    /// Backing byte offset in the owned ROM image.
    pub offset: usize,
}

/// An error while creating a cartridge image or mapping a guest address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomImageError {
    /// The supplied cartridge image is larger than the modeled 32 MiB maximum.
    ImageTooLarge {
        /// Actual input size in bytes.
        actual: usize,
        /// Maximum accepted size in bytes.
        maximum: usize,
    },
    /// The input does not contain a complete fixed GBA header.
    InvalidHeader(RomHeaderError),
    /// The CPU address is not in one of the three Game Pak ROM windows.
    AddressOutsideRomWindows {
        /// CPU address that could not be mapped.
        address: u32,
    },
    /// The address maps beyond the supplied image and no conservative mirror
    /// mapping can be established for this non-power-of-two image.
    OffsetOutsideImage {
        /// CPU address that could not be backed by the file.
        address: u32,
        /// Offset within the selected 32 MiB Game Pak window before mirroring.
        raw_offset: usize,
        /// Number of bytes in the supplied image.
        image_size: usize,
    },
}

impl fmt::Display for RomImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImageTooLarge { actual, maximum } => write!(
                f,
                "GBA ROM image is too large: got {actual} bytes, maximum is {maximum}"
            ),
            Self::InvalidHeader(source) => write!(f, "invalid GBA ROM header: {source}"),
            Self::AddressOutsideRomWindows { address } => write!(
                f,
                "CPU address 0x{address:08X} is outside the Game Pak ROM windows"
            ),
            Self::OffsetOutsideImage {
                address,
                raw_offset,
                image_size,
            } => write!(
                f,
                "CPU address 0x{address:08X} maps to ROM offset 0x{raw_offset:X}, outside image size 0x{image_size:X}"
            ),
        }
    }
}

impl Error for RomImageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidHeader(source) => Some(source),
            _ => None,
        }
    }
}

/// An owned GBA image with parsed header metadata and conservative address mapping.
///
/// All bytes are caller-supplied local input. This type does not load files from
/// disk, access a BIOS, execute the image, or perform instruction translation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GbaRom {
    bytes: Vec<u8>,
    header: GbaHeader,
}

impl GbaRom {
    /// Construct a cartridge image from bytes.
    ///
    /// The fixed header must be present and the image must be at most 32 MiB.
    /// Header checksum mismatches remain diagnostic metadata on the header;
    /// they do not cause construction to fail.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RomImageError> {
        if bytes.len() > MAX_GBA_ROM_SIZE {
            return Err(RomImageError::ImageTooLarge {
                actual: bytes.len(),
                maximum: MAX_GBA_ROM_SIZE,
            });
        }

        let header = GbaHeader::parse(&bytes).map_err(RomImageError::InvalidHeader)?;
        Ok(Self { bytes, header })
    }

    /// Return parsed fixed-header metadata.
    pub const fn header(&self) -> &GbaHeader {
        &self.header
    }

    /// Return the byte length of the supplied image.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Return whether the owned image contains no bytes.
    ///
    /// A successfully constructed GbaRom is never empty, but the method is
    /// provided for conventional collection ergonomics.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Resolve a CPU address in a Game Pak window to a backing ROM offset.
    ///
    /// The three 32 MiB Game Pak windows expose the same cartridge image, with
    /// distinct wait-state selections. For power-of-two image sizes, unused
    /// high address bits are conservatively modeled as a mirror using a mask.
    /// For a non-power-of-two image, only offsets physically present in the
    /// supplied bytes are accepted; out-of-image accesses return an error
    /// rather than guessing the cartridge's address-line behavior.
    pub fn resolve_cpu_address(&self, address: u32) -> Result<RomAddress, RomImageError> {
        let (window, base) = match address {
            0x0800_0000..=0x09FF_FFFF => (RomWindow::WaitState0, 0x0800_0000),
            0x0A00_0000..=0x0BFF_FFFF => (RomWindow::WaitState1, 0x0A00_0000),
            0x0C00_0000..=0x0DFF_FFFF => (RomWindow::WaitState2, 0x0C00_0000),
            _ => return Err(RomImageError::AddressOutsideRomWindows { address }),
        };

        let raw_offset = (address - base) as usize;
        let offset = if raw_offset < self.bytes.len() {
            raw_offset
        } else if self.bytes.len().is_power_of_two() {
            raw_offset & (self.bytes.len() - 1)
        } else {
            return Err(RomImageError::OffsetOutsideImage {
                address,
                raw_offset,
                image_size: self.bytes.len(),
            });
        };

        Ok(RomAddress { window, offset })
    }

    /// Read a byte at a guest CPU address in a Game Pak ROM window.
    pub fn read_byte(&self, address: u32) -> Result<u8, RomImageError> {
        let mapped = self.resolve_cpu_address(address)?;
        Ok(self.bytes[mapped.offset])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::GBA_HEADER_LEN;

    fn synthetic_rom(size: usize) -> Vec<u8> {
        let mut bytes = vec![0_u8; size];
        bytes[0..4].copy_from_slice(&0xEA00_002Eu32.to_le_bytes());
        for (index, byte) in bytes.iter_mut().enumerate().skip(GBA_HEADER_LEN) {
            *byte = (index & 0xFF) as u8;
        }
        bytes
    }

    #[test]
    fn all_three_game_pak_windows_resolve_to_the_same_backing_offset() {
        let rom = GbaRom::from_bytes(synthetic_rom(0x4000)).expect("fixture should parse");
        let mapped = [
            rom.resolve_cpu_address(0x0800_0123).expect("WS0 maps"),
            rom.resolve_cpu_address(0x0A00_0123).expect("WS1 maps"),
            rom.resolve_cpu_address(0x0C00_0123).expect("WS2 maps"),
        ];

        assert_eq!(mapped[0].offset, 0x123);
        assert_eq!(mapped[1].offset, 0x123);
        assert_eq!(mapped[2].offset, 0x123);
        assert_eq!(mapped[0].window, RomWindow::WaitState0);
        assert_eq!(mapped[1].window, RomWindow::WaitState1);
        assert_eq!(mapped[2].window, RomWindow::WaitState2);
        assert_eq!(mapped[0].window.wait_state(), 0);
        assert_eq!(mapped[1].window.wait_state(), 1);
        assert_eq!(mapped[2].window.wait_state(), 2);
    }

    #[test]
    fn power_of_two_image_size_mirrors_inside_each_game_pak_window() {
        let rom = GbaRom::from_bytes(synthetic_rom(0x4000)).expect("fixture should parse");

        let mapped = rom
            .resolve_cpu_address(0x0800_4020)
            .expect("power-of-two ROM mapping should mirror");
        assert_eq!(mapped.window, RomWindow::WaitState0);
        assert_eq!(mapped.offset, 0x20);
        assert_eq!(rom.read_byte(0x0800_4020), rom.read_byte(0x0800_0020));
    }

    #[test]
    fn rejects_addresses_outside_the_game_pak_rom_windows() {
        let rom = GbaRom::from_bytes(synthetic_rom(0x4000)).expect("fixture should parse");

        assert_eq!(
            rom.resolve_cpu_address(0x0200_0000),
            Err(RomImageError::AddressOutsideRomWindows {
                address: 0x0200_0000,
            })
        );
    }

    #[test]
    fn non_power_of_two_image_does_not_guess_out_of_file_mirroring() {
        let rom = GbaRom::from_bytes(synthetic_rom(GBA_HEADER_LEN + 1))
            .expect("header-sized fixture should parse");

        assert_eq!(
            rom.resolve_cpu_address(0x0800_00C1),
            Err(RomImageError::OffsetOutsideImage {
                address: 0x0800_00C1,
                raw_offset: GBA_HEADER_LEN + 1,
                image_size: GBA_HEADER_LEN + 1,
            })
        );
    }
}
