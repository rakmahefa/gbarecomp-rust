//! Conservative ARM-state decoder for the statically emitted subset.
//!
//! This decoder is deliberately incomplete. Every unrecognized encoding is
//! returned as Unsupported; callers must not reinterpret it as a no-op.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmInstruction {
    Branch {
        condition: u8,
        link: bool,
        offset: i32,
    },
    DataProcessingImmediate {
        condition: u8,
        opcode: u8,
        set_flags: bool,
        rn: u8,
        rd: u8,
        value: u32,
        shifter_carry: Option<bool>,
    },
    Nop,
    BranchExchange {
        condition: u8,
        rm: u8,
    },
    SoftwareInterrupt {
        condition: u8,
        comment: u32,
    },
    Unsupported,
}

pub fn decode(word: u32) -> ArmInstruction {
    let condition = (word >> 28) as u8;

    if word == 0xE1A0_0000 {
        return ArmInstruction::Nop;
    }

    if condition == 0x0F {
        // ARM7TDMI does not define the later unconditional instruction space.
        // Reject it before masks for ordinary BX/SWI encodings can match.
        return ArmInstruction::Unsupported;
    }

    if word & 0x0FFF_FFF0 == 0x012F_FF10 {
        return ArmInstruction::BranchExchange {
            condition,
            rm: (word & 0x0F) as u8,
        };
    }

    if word & 0x0F00_0000 == 0x0F00_0000 {
        return ArmInstruction::SoftwareInterrupt {
            condition,
            comment: word & 0x00FF_FFFF,
        };
    }

    if word & 0x0E00_0000 == 0x0A00_0000 {
        let encoded = word & 0x00FF_FFFF;
        return ArmInstruction::Branch {
            condition,
            link: word & (1 << 24) != 0,
            offset: ((encoded << 8) as i32) >> 6,
        };
    }

    // Data-processing class with a rotated 8-bit immediate operand only.
    if word & 0x0C00_0000 == 0 && word & (1 << 25) != 0 {
        let opcode = ((word >> 21) & 0x0F) as u8;
        if !matches!(opcode, 0 | 1 | 2 | 4 | 8 | 9 | 10 | 12 | 13 | 14 | 15) {
            return ArmInstruction::Unsupported;
        }

        let immediate = word & 0xFF;
        let rotation = ((word >> 8) & 0x0F) as u32 * 2;
        let value = immediate.rotate_right(rotation);
        return ArmInstruction::DataProcessingImmediate {
            condition,
            opcode,
            set_flags: word & (1 << 20) != 0 || matches!(opcode, 8 | 9 | 10),
            rn: ((word >> 16) & 0x0F) as u8,
            rd: ((word >> 12) & 0x0F) as u8,
            value,
            shifter_carry: (rotation != 0).then_some(value & 0x8000_0000 != 0),
        };
    }

    ArmInstruction::Unsupported
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_entry_style_branch_to_pc_relative_displacement() {
        assert_eq!(
            decode(0xEA00_002E),
            ArmInstruction::Branch {
                condition: 0xE,
                link: false,
                offset: 0xB8,
            }
        );
    }

    #[test]
    fn decodes_rotated_immediate_mov_and_carry() {
        assert_eq!(
            decode(0xE3B0_0101),
            ArmInstruction::DataProcessingImmediate {
                condition: 0xE,
                opcode: 13,
                set_flags: true,
                rn: 0,
                rd: 0,
                value: 0x4000_0000,
                shifter_carry: Some(false),
            }
        );
    }

    #[test]
    fn decodes_software_interrupt_with_a_valid_condition() {
        assert_eq!(
            decode(0xEF12_3456),
            ArmInstruction::SoftwareInterrupt {
                condition: 0xE,
                comment: 0x0012_3456,
            }
        );
    }

    #[test]
    fn rejects_condition_f_encodings_before_bx_or_swi_masks() {
        assert_eq!(decode(0xFF00_0001), ArmInstruction::Unsupported);
        assert_eq!(decode(0xF12F_FF10), ArmInstruction::Unsupported);
    }

    #[test]
    fn leaves_register_shift_and_unknown_instruction_unsupported() {
        assert_eq!(decode(0xE1A0_0061), ArmInstruction::Unsupported);
        assert_eq!(decode(0xE7F0_00F0), ArmInstruction::Unsupported);
    }
}
