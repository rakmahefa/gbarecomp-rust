//! Conservative Thumb-state decoder for the statically emitted subset.
//!
//! Unrecognized encodings are rejected during static recompilation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbInstruction {
    Branch {
        offset: i32,
    },
    ConditionalBranch {
        condition: u8,
        offset: i32,
    },
    AluImmediate {
        operation: ThumbImmediateOp,
        rd: u8,
        immediate: u8,
    },
    BranchExchange {
        rm: u8,
    },
    SoftwareInterrupt {
        comment: u8,
    },
    LongBranchPrefix {
        offset: i32,
    },
    LongBranchSuffix {
        offset: u16,
    },
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbImmediateOp {
    Move,
    Compare,
    Add,
    Subtract,
}

pub fn decode(word: u16) -> ThumbInstruction {
    if word & 0xFF87 == 0x4700 {
        return ThumbInstruction::BranchExchange {
            rm: ((word >> 3) & 0x0F) as u8,
        };
    }

    if word & 0xF800 == 0xF000 {
        let value = u32::from(word & 0x07FF);
        return ThumbInstruction::LongBranchPrefix {
            offset: sign_extend(value, 11) << 12,
        };
    }

    if word & 0xF800 == 0xF800 {
        return ThumbInstruction::LongBranchSuffix {
            offset: (word & 0x07FF) << 1,
        };
    }

    if word & 0xF000 == 0xD000 {
        let condition = ((word >> 8) & 0x0F) as u8;
        if condition == 0x0F {
            return ThumbInstruction::SoftwareInterrupt {
                comment: (word & 0x00FF) as u8,
            };
        }
        if condition == 0x0E {
            return ThumbInstruction::Unsupported;
        }
        return ThumbInstruction::ConditionalBranch {
            condition,
            offset: sign_extend(u32::from(word & 0x00FF), 8) << 1,
        };
    }

    if word & 0xF800 == 0xE000 {
        return ThumbInstruction::Branch {
            offset: sign_extend(u32::from(word & 0x07FF), 11) << 1,
        };
    }

    if word & 0xE000 == 0x2000 {
        let operation = match (word >> 11) & 0x03 {
            0 => ThumbImmediateOp::Move,
            1 => ThumbImmediateOp::Compare,
            2 => ThumbImmediateOp::Add,
            _ => ThumbImmediateOp::Subtract,
        };
        return ThumbInstruction::AluImmediate {
            operation,
            rd: ((word >> 8) & 0x07) as u8,
            immediate: (word & 0x00FF) as u8,
        };
    }

    ThumbInstruction::Unsupported
}

fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_thumb_branches_with_sign_extension() {
        assert_eq!(decode(0xE001), ThumbInstruction::Branch { offset: 2 });
        assert_eq!(decode(0xE7FF), ThumbInstruction::Branch { offset: -2 });
    }

    #[test]
    fn decodes_conditional_branch_and_movs_immediate() {
        assert_eq!(
            decode(0xD101),
            ThumbInstruction::ConditionalBranch {
                condition: 1,
                offset: 2,
            }
        );
        assert_eq!(
            decode(0x2107),
            ThumbInstruction::AluImmediate {
                operation: ThumbImmediateOp::Move,
                rd: 1,
                immediate: 7,
            }
        );
        assert_eq!(
            decode(0x2907),
            ThumbInstruction::AluImmediate {
                operation: ThumbImmediateOp::Compare,
                rd: 1,
                immediate: 7,
            }
        );
        assert_eq!(
            decode(0x3107),
            ThumbInstruction::AluImmediate {
                operation: ThumbImmediateOp::Add,
                rd: 1,
                immediate: 7,
            }
        );
        assert_eq!(
            decode(0x3907),
            ThumbInstruction::AluImmediate {
                operation: ThumbImmediateOp::Subtract,
                rd: 1,
                immediate: 7,
            }
        );
    }

    #[test]
    fn rejects_unknown_thumb_encodings() {
        assert_eq!(decode(0xDE00), ThumbInstruction::Unsupported);
    }
}
