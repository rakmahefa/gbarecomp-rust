//! Typed intermediate representation and diagnostics for static recompilation.

use super::{InstructionSet, RecompileError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProcessingOpcode {
    And,
    ExclusiveOr,
    Subtract,
    ReverseSubtract,
    Add,
    Test,
    TestExclusiveOr,
    Compare,
    CompareNegative,
    Or,
    Move,
    BitClear,
    MoveNot,
}

impl DataProcessingOpcode {
    pub fn from_arm_opcode(opcode: u8) -> Option<Self> {
        Some(match opcode {
            0 => Self::And,
            1 => Self::ExclusiveOr,
            2 => Self::Subtract,
            3 => Self::ReverseSubtract,
            4 => Self::Add,
            8 => Self::Test,
            9 => Self::TestExclusiveOr,
            10 => Self::Compare,
            11 => Self::CompareNegative,
            12 => Self::Or,
            13 => Self::Move,
            14 => Self::BitClear,
            15 => Self::MoveNot,
            _ => return None,
        })
    }

    pub const fn writes_destination(self) -> bool {
        !matches!(
            self,
            Self::Test | Self::TestExclusiveOr | Self::Compare | Self::CompareNegative
        )
    }

    pub const fn updates_flags_by_definition(self) -> bool {
        matches!(
            self,
            Self::Test | Self::TestExclusiveOr | Self::Compare | Self::CompareNegative
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbImmediateOpcode {
    Move,
    Compare,
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrOperation {
    Nop,
    ArmDataProcessingImmediate {
        condition: u8,
        opcode: DataProcessingOpcode,
        set_flags: bool,
        rn: u8,
        rd: u8,
        value: u32,
        shifter_carry: Option<bool>,
    },
    ThumbAluImmediate {
        operation: ThumbImmediateOpcode,
        rd: u8,
        immediate: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrInstruction {
    pub address: u32,
    pub raw: u32,
    pub instruction_set: InstructionSet,
    pub operation: IrOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockTerminator {
    Fallthrough {
        target: u32,
        instruction_set: InstructionSet,
    },
    Branch {
        condition: u8,
        target: u32,
        instruction_set: InstructionSet,
        link: bool,
        return_address: u32,
        fallthrough: Option<u32>,
    },
    ConditionalBranch {
        condition: u8,
        target: u32,
        fallthrough: u32,
        instruction_set: InstructionSet,
    },
    Exit,
    Unsupported,
    Indirect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub start: u32,
    pub instruction_set: InstructionSet,
    pub instructions: Vec<IrInstruction>,
    pub terminator: BlockTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisReport {
    pub blocks: Vec<BasicBlock>,
    pub diagnostics: Vec<RecompileError>,
    pub instruction_count: usize,
    pub instruction_limit: usize,
    pub reached_instruction_limit: bool,
}

impl AnalysisReport {
    pub fn is_complete_for_codegen(&self) -> bool {
        self.diagnostics.is_empty()
            && !self.reached_instruction_limit
            && !self.blocks.is_empty()
            && self.blocks.iter().all(|block| {
                !matches!(
                    block.terminator,
                    BlockTerminator::Unsupported | BlockTerminator::Indirect
                )
            })
    }

    pub fn first_error(&self) -> Option<RecompileError> {
        self.diagnostics.first().cloned().or_else(|| {
            self.reached_instruction_limit
                .then_some(RecompileError::InstructionLimitReached {
                    limit: self.instruction_limit,
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_operations_do_not_write_registers() {
        assert!(!DataProcessingOpcode::Compare.writes_destination());
        assert!(!DataProcessingOpcode::Test.writes_destination());
        assert!(!DataProcessingOpcode::CompareNegative.writes_destination());
        assert!(DataProcessingOpcode::Move.writes_destination());
        assert!(DataProcessingOpcode::ReverseSubtract.writes_destination());
        assert!(DataProcessingOpcode::CompareNegative.updates_flags_by_definition());
    }
}
