#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThumbOperation {
    BranchExchange {
        rm: u8,
        link: bool,
    },
    ConditionalBranch {
        condition: u8,
        offset: i32,
    },
    Branch {
        offset: i32,
    },
    SoftwareInterrupt {
        comment: u8,
    },
    AluImmediate {
        operation: ImmediateAluOp,
        rd: u8,
        immediate: u8,
    },
    LoadStoreImmediate {
        load: bool,
        byte: bool,
        rn: u8,
        rd: u8,
        offset: u16,
    },
    LongBranchPrefix {
        offset: i32,
    },
    LongBranchSuffix {
        offset: u16,
    },
    Unimplemented {
        raw: u16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImmediateAluOp {
    Move,
    Compare,
    Add,
    Subtract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedThumb {
    pub operation: ThumbOperation,
}

pub fn decode(instruction: u16) -> DecodedThumb {
    if instruction & 0xFF87 == 0x4700 {
        return DecodedThumb {
            operation: ThumbOperation::BranchExchange {
                rm: ((instruction >> 3) & 0x0F) as u8,
                link: instruction & 0x0080 != 0,
            },
        };
    }

    if instruction & 0xF800 == 0xF000 {
        let immediate = u32::from(instruction & 0x07FF);
        let signed_offset = sign_extend(immediate, 11) << 12;
        return DecodedThumb {
            operation: ThumbOperation::LongBranchPrefix {
                offset: signed_offset,
            },
        };
    }

    if instruction & 0xF800 == 0xF800 {
        return DecodedThumb {
            operation: ThumbOperation::LongBranchSuffix {
                offset: (instruction & 0x07FF) << 1,
            },
        };
    }

    if instruction & 0xF000 == 0xD000 {
        let condition = ((instruction >> 8) & 0x0F) as u8;
        if condition == 0x0F {
            return DecodedThumb {
                operation: ThumbOperation::SoftwareInterrupt {
                    comment: (instruction & 0x00FF) as u8,
                },
            };
        }
        if condition == 0x0E {
            return DecodedThumb {
                operation: ThumbOperation::Unimplemented { raw: instruction },
            };
        }
        return DecodedThumb {
            operation: ThumbOperation::ConditionalBranch {
                condition,
                offset: sign_extend(u32::from(instruction & 0x00FF), 8) << 1,
            },
        };
    }

    if instruction & 0xF800 == 0xE000 {
        return DecodedThumb {
            operation: ThumbOperation::Branch {
                offset: sign_extend(u32::from(instruction & 0x07FF), 11) << 1,
            },
        };
    }

    if instruction & 0xE000 == 0x2000 {
        let operation = match (instruction >> 11) & 0x03 {
            0 => ImmediateAluOp::Move,
            1 => ImmediateAluOp::Compare,
            2 => ImmediateAluOp::Add,
            _ => ImmediateAluOp::Subtract,
        };
        return DecodedThumb {
            operation: ThumbOperation::AluImmediate {
                operation,
                rd: ((instruction >> 8) & 0x07) as u8,
                immediate: (instruction & 0x00FF) as u8,
            },
        };
    }

    if instruction & 0xE000 == 0x6000 {
        let byte = instruction & 0x1000 != 0;
        let load = instruction & 0x0800 != 0;
        let immediate = u16::from((instruction >> 6) & 0x1F);
        let offset = if byte { immediate } else { immediate << 2 };
        return DecodedThumb {
            operation: ThumbOperation::LoadStoreImmediate {
                load,
                byte,
                rn: ((instruction >> 3) & 0x07) as u8,
                rd: (instruction & 0x07) as u8,
                offset,
            },
        };
    }

    DecodedThumb {
        operation: ThumbOperation::Unimplemented { raw: instruction },
    }
}

fn sign_extend(value: u32, bits: u32) -> i32 {
    let shift = 32 - bits;
    ((value << shift) as i32) >> shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_thumb_unconditional_branch() {
        assert_eq!(
            decode(0xE001).operation,
            ThumbOperation::Branch { offset: 2 }
        );
        assert_eq!(
            decode(0xE7FF).operation,
            ThumbOperation::Branch { offset: -2 }
        );
    }

    #[test]
    fn decodes_thumb_conditional_branch() {
        assert_eq!(
            decode(0xD101).operation,
            ThumbOperation::ConditionalBranch {
                condition: 1,
                offset: 2
            }
        );
    }

    #[test]
    fn decodes_thumb_immediate_move() {
        assert_eq!(
            decode(0x2107).operation,
            ThumbOperation::AluImmediate {
                operation: ImmediateAluOp::Move,
                rd: 1,
                immediate: 7
            }
        );
    }

    #[test]
    fn flags_unknown_encodings_instead_of_nopping() {
        assert_eq!(
            decode(0xDE00).operation,
            ThumbOperation::Unimplemented { raw: 0xDE00 }
        );
    }
}
