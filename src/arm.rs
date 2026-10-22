#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedArm {
    pub condition: u8,
    pub operation: ArmOperation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArmOperation {
    Branch {
        link: bool,
        offset: i32,
    },
    BranchExchange {
        rm: u8,
    },
    SoftwareInterrupt {
        comment: u32,
    },
    DataProcessing {
        opcode: u8,
        set_flags: bool,
        rn: u8,
        rd: u8,
        operand2: Operand2,
    },
    SingleDataTransfer {
        load: bool,
        byte: bool,
        pre_index: bool,
        add_offset: bool,
        write_back: bool,
        rn: u8,
        rd: u8,
        offset: TransferOffset,
    },
    BlockDataTransfer {
        load: bool,
        pre_index: bool,
        add_offset: bool,
        write_back: bool,
        rn: u8,
        register_list: u16,
    },
    Multiply {
        accumulate: bool,
        set_flags: bool,
        rd: u8,
        rn: u8,
        rs: u8,
        rm: u8,
    },
    Unimplemented {
        raw: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operand2 {
    Immediate {
        value: u8,
        rotate: u8,
    },
    Register {
        rm: u8,
        shift_type: u8,
        shift_amount: u8,
        shift_by_register: bool,
        rs: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferOffset {
    Immediate(u16),
    Register {
        rm: u8,
        shift_type: u8,
        shift_amount: u8,
    },
}

pub fn decode(word: u32) -> DecodedArm {
    let condition = (word >> 28) as u8;

    if word & 0x0FFF_FFF0 == 0x012F_FF10 {
        return DecodedArm {
            condition,
            operation: ArmOperation::BranchExchange {
                rm: (word & 0x0F) as u8,
            },
        };
    }

    if word & 0x0F00_0000 == 0x0F00_0000 {
        return DecodedArm {
            condition,
            operation: ArmOperation::SoftwareInterrupt {
                comment: word & 0x00FF_FFFF,
            },
        };
    }

    if condition != 0x0F && word & 0x0E00_0000 == 0x0A00_0000 {
        let immediate = word & 0x00FF_FFFF;
        return DecodedArm {
            condition,
            operation: ArmOperation::Branch {
                link: word & (1 << 24) != 0,
                offset: sign_extend(immediate, 24) << 2,
            },
        };
    }

    if word & 0x0FC0_00F0 == 0x0000_0090 {
        return DecodedArm {
            condition,
            operation: ArmOperation::Multiply {
                accumulate: word & (1 << 21) != 0,
                set_flags: word & (1 << 20) != 0,
                rd: ((word >> 16) & 0x0F) as u8,
                rn: ((word >> 12) & 0x0F) as u8,
                rs: ((word >> 8) & 0x0F) as u8,
                rm: (word & 0x0F) as u8,
            },
        };
    }

    if word & 0x0C00_0000 == 0x0800_0000 {
        return DecodedArm {
            condition,
            operation: ArmOperation::BlockDataTransfer {
                pre_index: word & (1 << 24) != 0,
                add_offset: word & (1 << 23) != 0,
                write_back: word & (1 << 21) != 0,
                load: word & (1 << 20) != 0,
                rn: ((word >> 16) & 0x0F) as u8,
                register_list: (word & 0xFFFF) as u16,
            },
        };
    }

    if word & 0x0C00_0000 == 0x0400_0000 {
        let register_offset = word & (1 << 25) != 0;
        let offset = if register_offset {
            if word & (1 << 4) != 0 {
                return DecodedArm {
                    condition,
                    operation: ArmOperation::Unimplemented { raw: word },
                };
            }
            TransferOffset::Register {
                rm: (word & 0x0F) as u8,
                shift_type: ((word >> 5) & 0x03) as u8,
                shift_amount: ((word >> 7) & 0x1F) as u8,
            }
        } else {
            TransferOffset::Immediate((word & 0x0FFF) as u16)
        };

        return DecodedArm {
            condition,
            operation: ArmOperation::SingleDataTransfer {
                load: word & (1 << 20) != 0,
                byte: word & (1 << 22) != 0,
                pre_index: word & (1 << 24) != 0,
                add_offset: word & (1 << 23) != 0,
                write_back: word & (1 << 21) != 0,
                rn: ((word >> 16) & 0x0F) as u8,
                rd: ((word >> 12) & 0x0F) as u8,
                offset,
            },
        };
    }

    if word & 0x0C00_0000 == 0 && word & 0x0000_0090 == 0x0000_0090 {
        return DecodedArm {
            condition,
            operation: ArmOperation::Unimplemented { raw: word },
        };
    }

    if word & 0x0C00_0000 == 0 {
        let operand2 = if word & (1 << 25) != 0 {
            Operand2::Immediate {
                value: (word & 0xFF) as u8,
                rotate: ((word >> 8) & 0x0F) as u8,
            }
        } else {
            Operand2::Register {
                rm: (word & 0x0F) as u8,
                shift_type: ((word >> 5) & 0x03) as u8,
                shift_amount: ((word >> 7) & 0x1F) as u8,
                shift_by_register: word & (1 << 4) != 0,
                rs: ((word >> 8) & 0x0F) as u8,
            }
        };

        return DecodedArm {
            condition,
            operation: ArmOperation::DataProcessing {
                opcode: ((word >> 21) & 0x0F) as u8,
                set_flags: word & (1 << 20) != 0,
                rn: ((word >> 16) & 0x0F) as u8,
                rd: ((word >> 12) & 0x0F) as u8,
                operand2,
            },
        };
    }

    DecodedArm {
        condition,
        operation: ArmOperation::Unimplemented { raw: word },
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
    fn decodes_arm_branch_and_link() {
        let decoded = decode(0xEB00_0002);
        assert_eq!(decoded.condition, 0xE);
        assert_eq!(
            decoded.operation,
            ArmOperation::Branch {
                link: true,
                offset: 8
            }
        );
    }

    #[test]
    fn decodes_bx_register() {
        let decoded = decode(0xE12F_FF13);
        assert_eq!(decoded.operation, ArmOperation::BranchExchange { rm: 3 });
    }

    #[test]
    fn decodes_data_processing_immediate() {
        let decoded = decode(0xE3A0_0007);
        assert_eq!(
            decoded.operation,
            ArmOperation::DataProcessing {
                opcode: 0xD,
                set_flags: false,
                rn: 0,
                rd: 0,
                operand2: Operand2::Immediate {
                    value: 7,
                    rotate: 0
                }
            }
        );
    }

    #[test]
    fn flags_unknown_encodings_instead_of_nopping() {
        assert!(matches!(
            decode(0xE7F0_00F0).operation,
            ArmOperation::Unimplemented { .. }
        ));
    }
}
