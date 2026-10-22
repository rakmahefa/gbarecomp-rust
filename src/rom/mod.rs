//! Cartridge metadata parsing and static CPU-address mapping.
//!
//! ROM bytes are treated as input data. These modules inspect headers and map
//! bus addresses to ROM offsets; they do not decode or execute guest instructions.

mod header;
mod image;

pub use header::{GBA_HEADER_LEN, GbaHeader, RomHeaderError};
pub use image::{GbaRom, MAX_GBA_ROM_SIZE, RomAddress, RomImageError, RomWindow};
