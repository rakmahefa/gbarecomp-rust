//! Core library for the Rust-output, static-only GBA recompiler.
//!
//! The build-time recompiler will emit Rust source. Cargo/rustc compile that
//! source before distribution; the shipped runtime never translates guest
//! instructions. There is intentionally no interpreter or JIT path.

pub mod recompiler;
pub mod rom;
pub mod runtime;
