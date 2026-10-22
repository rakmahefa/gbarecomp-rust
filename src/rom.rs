use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

pub const GBA_HEADER_END: usize = 0xC0;
const TITLE_RANGE: std::ops::Range<usize> = 0xA0..0xAC;
const GAME_CODE_RANGE: std::ops::Range<usize> = 0xAC..0xB0;
const MAKER_CODE_RANGE: std::ops::Range<usize> = 0xB0..0xB2;
const CHECKSUM_INPUT_RANGE: std::ops::RangeInclusive<usize> = 0xA0..=0xBC;
pub const GBA_ROM_BASE: u32 = 0x0800_0000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RomHeader {
    pub title: String,
    pub game_code: String,
    pub maker_code: String,
    pub unit_code: u8,
    pub device_type: u8,
    pub software_version: u8,
    pub declared_checksum: u8,
    pub computed_checksum: u8,
}

impl RomHeader {
    pub fn parse(bytes: &[u8]) -> Result<Self, RomError> {
        if bytes.len() < GBA_HEADER_END {
            return Err(RomError::TooSmall {
                actual: bytes.len(),
                minimum: GBA_HEADER_END,
            });
        }

        let computed_checksum = CHECKSUM_INPUT_RANGE
            .map(|index| bytes[index])
            .fold(0_u8, u8::wrapping_add)
            .wrapping_neg()
            .wrapping_sub(0x19);

        Ok(Self {
            title: printable_ascii(&bytes[TITLE_RANGE]),
            game_code: printable_ascii(&bytes[GAME_CODE_RANGE]),
            maker_code: printable_ascii(&bytes[MAKER_CODE_RANGE]),
            unit_code: bytes[0xB3],
            device_type: bytes[0xB4],
            software_version: bytes[0xBC],
            declared_checksum: bytes[0xBD],
            computed_checksum,
        })
    }

    pub fn checksum_matches(&self) -> bool {
        self.declared_checksum == self.computed_checksum
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RomImage {
    bytes: Vec<u8>,
    pub header: RomHeader,
}

impl RomImage {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RomError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| RomError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_bytes(bytes)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, RomError> {
        let header = RomHeader::parse(&bytes)?;
        Ok(Self { bytes, header })
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn contains_guest_range(&self, address: u32, width: usize) -> bool {
        if address < GBA_ROM_BASE || width == 0 {
            return false;
        }
        let offset = (address - GBA_ROM_BASE) as usize;
        offset
            .checked_add(width)
            .is_some_and(|end| end <= self.bytes.len())
    }

    pub fn read_guest_u16(&self, address: u32) -> Option<u16> {
        if !self.contains_guest_range(address, 2) {
            return None;
        }
        let offset = (address - GBA_ROM_BASE) as usize;
        Some(u16::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
        ]))
    }

    pub fn read_guest_u32(&self, address: u32) -> Option<u32> {
        if !self.contains_guest_range(address, 4) {
            return None;
        }
        let offset = (address - GBA_ROM_BASE) as usize;
        Some(u32::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
            self.bytes[offset + 2],
            self.bytes[offset + 3],
        ]))
    }
}

fn printable_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() {
                char::from(*byte)
            } else {
                ' '
            }
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

#[derive(Debug)]
pub enum RomError {
    TooSmall {
        actual: usize,
        minimum: usize,
    },
    Io {
        path: String,
        source: std::io::Error,
    },
}

impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall { actual, minimum } => {
                write!(
                    f,
                    "ROM is too small: {actual} bytes, at least {minimum} required"
                )
            }
            Self::Io { path, source } => write!(f, "cannot read ROM {path}: {source}"),
        }
    }
}

impl Error for RomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::TooSmall { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rom() -> Vec<u8> {
        let mut bytes = vec![0_u8; GBA_HEADER_END + 4];
        bytes[TITLE_RANGE].copy_from_slice(b"TEST GAME   ");
        bytes[GAME_CODE_RANGE].copy_from_slice(b"ABCD");
        bytes[MAKER_CODE_RANGE].copy_from_slice(b"01");
        bytes[0xB3] = 0;
        bytes[0xB4] = 0;
        bytes[0xBC] = 1;
        let checksum = CHECKSUM_INPUT_RANGE
            .map(|index| bytes[index])
            .fold(0_u8, u8::wrapping_add)
            .wrapping_neg()
            .wrapping_sub(0x19);
        bytes[0xBD] = checksum;
        bytes[0xC0..0xC4].copy_from_slice(&0xEA00_0000_u32.to_le_bytes());
        bytes
    }

    #[test]
    fn parses_header_and_checksum() {
        let rom = RomImage::from_bytes(test_rom()).expect("valid synthetic ROM");
        assert_eq!(rom.header.title, "TEST GAME");
        assert_eq!(rom.header.game_code, "ABCD");
        assert!(rom.header.checksum_matches());
    }

    #[test]
    fn rejects_truncated_header() {
        let error = RomHeader::parse(&[0; 12]).expect_err("truncated header");
        assert!(matches!(error, RomError::TooSmall { .. }));
    }

    #[test]
    fn reads_little_endian_guest_words() {
        let rom = RomImage::from_bytes(test_rom()).expect("valid synthetic ROM");
        assert_eq!(rom.read_guest_u32(GBA_ROM_BASE), Some(0xEA00_0000));
        assert_eq!(rom.read_guest_u16(GBA_ROM_BASE + 1), Some(0));
        assert_eq!(rom.read_guest_u32(GBA_ROM_BASE + 4), None);
    }
}
