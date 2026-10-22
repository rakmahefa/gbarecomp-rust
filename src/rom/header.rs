//! Parsing of the fixed 0xC0-byte Game Boy Advance cartridge header.

use std::{error::Error, fmt};

/// Number of bytes needed to parse the standard GBA cartridge header.
pub const GBA_HEADER_LEN: usize = 0xC0;

/// Byte offset of the title field in the GBA header.
const TITLE_OFFSET: usize = 0xA0;
/// Byte offset of the four-byte game code in the GBA header.
const GAME_CODE_OFFSET: usize = 0xAC;
/// Byte offset of the two-byte maker code in the GBA header.
const MAKER_CODE_OFFSET: usize = 0xB0;
/// Byte offset of the fixed value field in the GBA header.
const FIXED_VALUE_OFFSET: usize = 0xB2;
/// Byte offset of the unit code field in the GBA header.
const UNIT_CODE_OFFSET: usize = 0xB3;
/// Byte offset of the device type field in the GBA header.
const DEVICE_TYPE_OFFSET: usize = 0xB4;
/// Byte offset of the software version field in the GBA header.
const SOFTWARE_VERSION_OFFSET: usize = 0xBC;
/// Byte offset of the complement checksum in the GBA header.
const CHECKSUM_OFFSET: usize = 0xBD;
/// Constant included in the GBA header complement-checksum formula.
const CHECKSUM_BIAS: u8 = 0x19;

/// A structural error encountered while reading a GBA cartridge header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomHeaderError {
    /// The input is shorter than the standard header.
    HeaderTooShort {
        /// Number of input bytes provided.
        actual: usize,
        /// Minimum number of bytes required.
        minimum: usize,
    },
}

impl fmt::Display for RomHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderTooShort { actual, minimum } => write!(
                f,
                "GBA ROM data is too short for its header: got {actual} bytes, need at least {minimum}"
            ),
        }
    }
}

impl Error for RomHeaderError {}

/// Parsed metadata from the fixed GBA cartridge header.
///
/// checksum_valid is diagnostic metadata, not a parse failure. Some homebrew
/// images have a malformed complement checksum; callers can decide whether to
/// reject such an image at a later validation stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GbaHeader {
    /// ARM branch instruction at cartridge offset zero, decoded as little-endian.
    pub entry_point: u32,
    /// Twelve raw bytes from the cartridge title field.
    pub title: [u8; 12],
    /// Four raw bytes identifying the game.
    pub game_code: [u8; 4],
    /// Two raw bytes identifying the maker.
    pub maker_code: [u8; 2],
    /// Fixed header byte; valid commercial images normally contain 0x96.
    pub fixed_value: u8,
    /// GBA unit code.
    pub unit_code: u8,
    /// Optional device type byte.
    pub device_type: u8,
    /// Software revision byte.
    pub software_version: u8,
    /// Complement checksum stored in the ROM header.
    pub complement_checksum: u8,
    /// Checksum calculated from header bytes 0xA0..=0xBC and bias 0x19.
    pub calculated_checksum: u8,
    /// Whether the stored complement checksum matches the calculated checksum.
    pub checksum_valid: bool,
}

impl GbaHeader {
    /// Parse the fixed metadata fields from a ROM image.
    ///
    /// The input can contain the entire ROM or any buffer at least
    /// GBA_HEADER_LEN bytes long. Checksum mismatch is reported in the
    /// returned structure rather than treated as a structural parsing error.
    pub fn parse(image: &[u8]) -> Result<Self, RomHeaderError> {
        if image.len() < GBA_HEADER_LEN {
            return Err(RomHeaderError::HeaderTooShort {
                actual: image.len(),
                minimum: GBA_HEADER_LEN,
            });
        }

        let title = copy_array::<12>(image, TITLE_OFFSET);
        let game_code = copy_array::<4>(image, GAME_CODE_OFFSET);
        let maker_code = copy_array::<2>(image, MAKER_CODE_OFFSET);
        let complement_checksum = image[CHECKSUM_OFFSET];
        let calculated_checksum = calculate_complement_checksum(image);

        Ok(Self {
            entry_point: u32::from_le_bytes(copy_array::<4>(image, 0)),
            title,
            game_code,
            maker_code,
            fixed_value: image[FIXED_VALUE_OFFSET],
            unit_code: image[UNIT_CODE_OFFSET],
            device_type: image[DEVICE_TYPE_OFFSET],
            software_version: image[SOFTWARE_VERSION_OFFSET],
            complement_checksum,
            calculated_checksum,
            checksum_valid: complement_checksum == calculated_checksum,
        })
    }

    /// Return the title as lossy UTF-8, removing trailing NULs and spaces.
    ///
    /// The raw bytes remain available in the title field for exact binary
    /// inspection.
    pub fn title_lossy(&self) -> String {
        String::from_utf8_lossy(&self.title)
            .trim_end_matches('\0')
            .trim_end()
            .to_owned()
    }
}

fn copy_array<const N: usize>(image: &[u8], offset: usize) -> [u8; N] {
    let mut output = [0_u8; N];
    output.copy_from_slice(&image[offset..offset + N]);
    output
}

fn calculate_complement_checksum(image: &[u8]) -> u8 {
    let sum = image[TITLE_OFFSET..=SOFTWARE_VERSION_OFFSET]
        .iter()
        .fold(CHECKSUM_BIAS, |sum, byte| sum.wrapping_add(*byte));
    0_u8.wrapping_sub(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_header() -> [u8; GBA_HEADER_LEN] {
        let mut image = [0_u8; GBA_HEADER_LEN];
        image[0..4].copy_from_slice(&0xEA00_002Eu32.to_le_bytes());
        image[TITLE_OFFSET..TITLE_OFFSET + 12].copy_from_slice(b"TEST GAME   ");
        image[GAME_CODE_OFFSET..GAME_CODE_OFFSET + 4].copy_from_slice(b"TSTR");
        image[MAKER_CODE_OFFSET..MAKER_CODE_OFFSET + 2].copy_from_slice(b"01");
        image[FIXED_VALUE_OFFSET] = 0x96;
        image[UNIT_CODE_OFFSET] = 0;
        image[DEVICE_TYPE_OFFSET] = 0;
        image[SOFTWARE_VERSION_OFFSET] = 0;

        image[CHECKSUM_OFFSET] = calculate_complement_checksum(&image);
        image
    }

    #[test]
    fn parses_fields_and_validates_a_synthetic_header() {
        let image = synthetic_header();
        let header = GbaHeader::parse(&image).expect("synthetic header should parse");

        assert_eq!(header.entry_point, 0xEA00_002E);
        assert_eq!(header.title, *b"TEST GAME   ");
        assert_eq!(header.title_lossy(), "TEST GAME");
        assert_eq!(header.game_code, *b"TSTR");
        assert_eq!(header.maker_code, *b"01");
        assert_eq!(header.fixed_value, 0x96);
        assert!(header.checksum_valid);
        assert_eq!(header.complement_checksum, header.calculated_checksum);
    }

    #[test]
    fn rejects_an_input_shorter_than_the_header() {
        let error = GbaHeader::parse(&[0_u8; GBA_HEADER_LEN - 1])
            .expect_err("short input must not be accepted");

        assert_eq!(
            error,
            RomHeaderError::HeaderTooShort {
                actual: GBA_HEADER_LEN - 1,
                minimum: GBA_HEADER_LEN,
            }
        );
    }

    #[test]
    fn reports_a_checksum_mismatch_without_hiding_the_header() {
        let mut image = synthetic_header();
        image[TITLE_OFFSET] ^= 1;

        let header = GbaHeader::parse(&image).expect("structurally valid header should parse");

        assert!(!header.checksum_valid);
        assert_ne!(header.complement_checksum, header.calculated_checksum);
    }
}
