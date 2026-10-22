//! Cartridge metadata parsing.
//!
//! ROM bytes are treated as input data. This module inspects the fixed GBA
//! header only; it does not decode or execute guest instructions.

mod header;

pub use header::{GbaHeader, RomHeaderError, GBA_HEADER_LEN};
