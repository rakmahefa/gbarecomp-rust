use std::collections::{BTreeSet, VecDeque};

use crate::arm::{self, ArmOperation};
use crate::cpu::InstructionSet;
use crate::rom::{RomImage, GBA_ROM_BASE};
use crate::thumb::{self, ThumbOperation};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstructionRecord {
    pub address: u32,
    pub width: u8,
    pub raw: u32,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlockExit {
    Sequential,
    StaticBranch { target: u32 },
    ExternalBranch { target: u32 },
    Call { target: u32, return_address: u32 },
    ConditionalBranch { target: u32, fallthrough: u32 },
    IndirectBranch,
    Unsupported { raw: u32 },
    OutsideRom,
    PreviouslyDecoded,
    InstructionLimit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    pub start: u32,
    pub instruction_set: InstructionSet,
    pub instructions: Vec<InstructionRecord>,
    pub exit: BlockExit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisReport {
    pub blocks: Vec<BasicBlock>,
    pub instruction_count: usize,
    pub truncated: bool,
}

impl AnalysisReport {
    pub fn unsupported_exit_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| matches!(&block.exit, BlockExit::Unsupported { .. }))
            .count()
    }

    pub fn indirect_exit_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| matches!(&block.exit, BlockExit::IndirectBranch))
            .count()
    }
}

/// Performs conservative reachable-code discovery over bytes mapped directly
/// into the cartridge ROM window. This is not a complete function finder:
/// computed branches, code in RAM, BIOS code and data/code ambiguities remain
/// unresolved and are reported rather than guessed.
pub fn analyze_rom(rom: &RomImage, max_instructions: usize) -> AnalysisReport {
    let mut pending = VecDeque::from([(GBA_ROM_BASE, InstructionSet::Arm)]);
    let mut visited_blocks = BTreeSet::new();
    let mut decoded_addresses = BTreeSet::new();
    let mut blocks = Vec::new();
    let mut instruction_count = 0_usize;
    let mut truncated = false;

    while let Some((start, instruction_set)) = pending.pop_front() {
        if !rom.contains_guest_range(start, instruction_width(instruction_set)) {
            continue;
        }
        if !visited_blocks.insert((start, instruction_set)) {
            continue;
        }

        let mut pc = start;
        let mut instructions = Vec::new();
        let mut exit: BlockExit;

        loop {
            if instruction_count >= max_instructions {
                truncated = true;
                exit = BlockExit::InstructionLimit;
                break;
            }

            let width = instruction_width(instruction_set);
            if !rom.contains_guest_range(pc, width) {
                exit = BlockExit::OutsideRom;
                break;
            }
            if !decoded_addresses.insert((pc, instruction_set)) {
                exit = BlockExit::PreviouslyDecoded;
                break;
            }

            match instruction_set {
                InstructionSet::Arm => {
                    let raw = rom.read_guest_u32(pc).expect("range checked above");
                    let decoded = arm::decode(raw);
                    let operation = decoded.operation;
                    instructions.push(InstructionRecord {
                        address: pc,
                        width: 4,
                        raw,
                        description: format!("{operation:?}"),
                    });
                    instruction_count += 1;
                    let next = pc.wrapping_add(4);

                    match operation {
                        ArmOperation::Branch { link, offset } => {
                            let target = pc.wrapping_add(8).wrapping_add(offset as u32);
                            if rom.contains_guest_range(target, 4) {
                                pending.push_back((target, InstructionSet::Arm));
                            }

                            if link {
                                if rom.contains_guest_range(next, 4) {
                                    pending.push_back((next, InstructionSet::Arm));
                                }
                                exit = BlockExit::Call {
                                    target,
                                    return_address: next,
                                };
                            } else if decoded.condition != 0x0E {
                                if rom.contains_guest_range(next, 4) {
                                    pending.push_back((next, InstructionSet::Arm));
                                }
                                exit = BlockExit::ConditionalBranch {
                                    target,
                                    fallthrough: next,
                                };
                            } else if rom.contains_guest_range(target, 4) {
                                exit = BlockExit::StaticBranch { target };
                            } else {
                                exit = BlockExit::ExternalBranch { target };
                            }
                            break;
                        }
                        ArmOperation::BranchExchange { .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ArmOperation::DataProcessing { opcode, rd, .. }
                            if rd == 15 && !(8..=11).contains(&opcode) =>
                        {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ArmOperation::SingleDataTransfer { load: true, rd: 15, .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ArmOperation::BlockDataTransfer {
                            load: true,
                            register_list,
                            ..
                        } if register_list & (1 << 15) != 0 => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ArmOperation::Unimplemented { raw } => {
                            exit = BlockExit::Unsupported { raw };
                            break;
                        }
                        ArmOperation::SoftwareInterrupt { .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        _ => pc = next,
                    }
                }
                InstructionSet::Thumb => {
                    let raw = rom.read_guest_u16(pc).expect("range checked above");
                    let decoded = thumb::decode(raw);
                    let operation = decoded.operation;
                    instructions.push(InstructionRecord {
                        address: pc,
                        width: 2,
                        raw: u32::from(raw),
                        description: format!("{operation:?}"),
                    });
                    instruction_count += 1;
                    let next = pc.wrapping_add(2);

                    match operation {
                        ThumbOperation::Branch { offset } => {
                            let target = pc.wrapping_add(4).wrapping_add(offset as u32);
                            if rom.contains_guest_range(target, 2) {
                                pending.push_back((target, InstructionSet::Thumb));
                                exit = BlockExit::StaticBranch { target };
                            } else {
                                exit = BlockExit::ExternalBranch { target };
                            }
                            break;
                        }
                        ThumbOperation::ConditionalBranch { condition, offset } => {
                            let target = pc.wrapping_add(4).wrapping_add(offset as u32);
                            if rom.contains_guest_range(target, 2) {
                                pending.push_back((target, InstructionSet::Thumb));
                            }
                            if rom.contains_guest_range(next, 2) {
                                pending.push_back((next, InstructionSet::Thumb));
                            }
                            if condition < 0x0E {
                                exit = BlockExit::ConditionalBranch {
                                    target,
                                    fallthrough: next,
                                };
                            } else {
                                exit = BlockExit::Unsupported {
                                    raw: u32::from(raw),
                                };
                            }
                            break;
                        }
                        ThumbOperation::BranchExchange { .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ThumbOperation::LongBranchSuffix { .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        ThumbOperation::Unimplemented { raw } => {
                            exit = BlockExit::Unsupported {
                                raw: u32::from(raw),
                            };
                            break;
                        }
                        ThumbOperation::SoftwareInterrupt { .. } => {
                            exit = BlockExit::IndirectBranch;
                            break;
                        }
                        _ => pc = next,
                    }
                }
            }
        }

        if !instructions.is_empty() {
            blocks.push(BasicBlock {
                start,
                instruction_set,
                instructions,
                exit,
            });
        }
    }

    AnalysisReport {
        blocks,
        instruction_count,
        truncated,
    }
}

fn instruction_width(instruction_set: InstructionSet) -> usize {
    match instruction_set {
        InstructionSet::Arm => 4,
        InstructionSet::Thumb => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::GBA_HEADER_END;

    fn synthetic_rom(words: &[(usize, u32)]) -> RomImage {
        let mut bytes = vec![0_u8; GBA_HEADER_END + 0x40];
        for (offset, word) in words {
            bytes[*offset..*offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        let checksum = (0xA0..=0xBC)
            .map(|index| bytes[index])
            .fold(0_u8, u8::wrapping_add)
            .wrapping_neg()
            .wrapping_sub(0x19);
        bytes[0xBD] = checksum;
        RomImage::from_bytes(bytes).expect("synthetic ROM has a full header")
    }

    #[test]
    fn follows_a_direct_arm_branch() {
        let rom = synthetic_rom(&[(0, 0xEA00_0001), (12, 0xE1A0_0000)]);
        let report = analyze_rom(&rom, 16);
        assert!(!report.blocks.is_empty());
        assert_eq!(
            report.blocks[0].exit,
            BlockExit::StaticBranch {
                target: GBA_ROM_BASE + 12
            }
        );
    }

    #[test]
    fn respects_instruction_limit() {
        let rom = synthetic_rom(&[(0, 0xE1A0_0000), (4, 0xE1A0_0000)]);
        let report = analyze_rom(&rom, 1);
        assert!(report.truncated);
        assert_eq!(report.instruction_count, 1);
    }
}
