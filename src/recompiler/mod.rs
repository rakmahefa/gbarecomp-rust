//! Contract and diagnostics for static recompilation.
//!
//! The intended pipeline is ROM analysis -> Rust source generation -> Cargo /
//! rustc native compilation. The runtime must never interpret GBA opcodes,
//! generate host code at runtime, or fall back to a JIT when static analysis
//! cannot resolve a path.

use std::{error::Error, fmt};

/// Instruction encoding state for an ARM7TDMI address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionSet {
    /// 32-bit ARM instruction encoding.
    Arm,
    /// 16-bit Thumb instruction encoding (with architecture-defined compound cases).
    Thumb,
}

/// A condition that prevents a cartridge from being fully statically recompiled.
///
/// Errors are surfaced during analysis or code generation. They are not
/// converted into runtime interpretation, JIT compilation, or silent no-ops.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecompileError {
    /// The decoder encountered an instruction for which no correct Rust lowering exists.
    UnsupportedInstruction {
        /// Guest ROM address of the instruction.
        address: u32,
        /// Raw instruction bits as read from the guest image.
        opcode: u32,
        /// ARM or Thumb encoding state.
        instruction_set: InstructionSet,
    },
    /// A statically known control-flow edge points outside resolvable code.
    UnresolvedStaticTarget {
        /// Guest address of the branch or call.
        instruction_address: u32,
        /// Guest destination address.
        target_address: u32,
    },
    /// An indirect branch or call has no complete, statically established target set.
    UnresolvedIndirectControlFlow {
        /// Guest address of the indirect branch or call.
        instruction_address: u32,
    },
    /// Executable code is created or modified at runtime and cannot be represented
    /// by a statically generated Rust function set.
    RuntimeGeneratedCode {
        /// Guest address of the affected code region.
        address: u32,
    },
    /// Analysis cannot safely distinguish instructions from embedded data.
    CodeDataOverlap {
        /// Guest address of the ambiguous region.
        address: u32,
    },
}

impl fmt::Display for RecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedInstruction {
                address,
                opcode,
                instruction_set,
            } => write!(
                f,
                "unsupported {instruction_set:?} instruction 0x{opcode:08X} at 0x{address:08X}"
            ),
            Self::UnresolvedStaticTarget {
                instruction_address,
                target_address,
            } => write!(
                f,
                "unresolved static target 0x{target_address:08X} from 0x{instruction_address:08X}"
            ),
            Self::UnresolvedIndirectControlFlow {
                instruction_address,
            } => write!(
                f,
                "indirect control flow at 0x{instruction_address:08X} has no static target set"
            ),
            Self::RuntimeGeneratedCode { address } => write!(
                f,
                "runtime-generated guest code at 0x{address:08X} cannot be statically recompiled"
            ),
            Self::CodeDataOverlap { address } => write!(
                f,
                "code/data classification is ambiguous at 0x{address:08X}"
            ),
        }
    }
}

impl Error for RecompileError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_indirect_control_flow_has_an_actionable_diagnostic() {
        let error = RecompileError::UnresolvedIndirectControlFlow {
            instruction_address: 0x0800_1234,
        };

        assert!(error.to_string().contains("0x08001234"));
        assert!(error.to_string().contains("no static target set"));
    }

    #[test]
    fn runtime_generated_code_is_rejected_during_recompilation() {
        let error = RecompileError::RuntimeGeneratedCode {
            address: 0x0300_1000,
        };

        assert!(error.to_string().contains("0x03001000"));
        assert!(
            error
                .to_string()
                .contains("cannot be statically recompiled")
        );
    }
}
