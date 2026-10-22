//! Conservative reachable-code discovery for the statically emitted subset.
//!
//! Discovery starts at the cartridge entry branch, follows only statically
//! known edges, and records every unsupported or indirect edge. It never
//! attempts to interpret an instruction at runtime.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::rom::GbaRom;

use super::ir::{
    AnalysisReport, BasicBlock, BlockTerminator, DataProcessingOpcode, IrInstruction, IrOperation,
    ThumbImmediateOpcode,
};
use super::{InstructionSet, RecompileError};
use super::{arm, thumb};

const ENTRY_ADDRESS: u32 = 0x0800_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalysisOptions {
    /// Maximum number of guest instruction encodings to decode.
    pub max_instructions: usize,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_instructions: 250_000,
        }
    }
}

pub fn analyze(rom: &GbaRom, options: AnalysisOptions) -> AnalysisReport {
    let mut pending = VecDeque::from([(ENTRY_ADDRESS, InstructionSet::Arm)]);
    let mut block_starts = BTreeSet::new();
    let mut decoded_addresses = BTreeMap::<u32, InstructionSet>::new();
    let mut blocks = Vec::new();
    let mut diagnostics = Vec::new();
    let mut instruction_count = 0_usize;
    let mut reached_instruction_limit = false;

    while let Some((start, instruction_set)) = pending.pop_front() {
        if !block_starts.insert((start, instruction_set)) {
            continue;
        }

        if !address_has_bytes(rom, start, instruction_width(instruction_set)) {
            diagnostics.push(RecompileError::UnresolvedStaticTarget {
                instruction_address: start,
                target_address: start,
            });
            continue;
        }

        let mut pc = start;
        let mut instructions = Vec::new();
        let terminator;

        loop {
            if instruction_count >= options.max_instructions {
                reached_instruction_limit = true;
                diagnostics.push(RecompileError::InstructionLimitReached {
                    limit: options.max_instructions,
                });
                terminator = BlockTerminator::Unsupported;
                break;
            }

            if decoded_addresses.contains_key(&pc) {
                // This is a new block entry into bytes already consumed by a prior
                // block. Until block splitting is implemented, fail closed rather
                // than emitting overlapping functions with ambiguous ownership.
                diagnostics.push(RecompileError::CodeDataOverlap { address: pc });
                terminator = BlockTerminator::Unsupported;
                break;
            }

            match instruction_set {
                InstructionSet::Arm => {
                    let Some(raw) = read_u32(rom, pc) else {
                        diagnostics.push(unresolved_edge(pc, pc));
                        terminator = BlockTerminator::Unsupported;
                        break;
                    };
                    let instruction = arm::decode(raw);
                    decoded_addresses.insert(pc, instruction_set);
                    instruction_count += 1;

                    match instruction {
                        arm::ArmInstruction::Nop => {
                            instructions.push(IrInstruction {
                                address: pc,
                                raw,
                                instruction_set,
                                operation: IrOperation::Nop,
                            });
                            pc = pc.wrapping_add(4);
                        }
                        arm::ArmInstruction::DataProcessingImmediate {
                            condition,
                            opcode,
                            set_flags,
                            rn,
                            rd,
                            value,
                            shifter_carry,
                        } => {
                            let Some(opcode) = DataProcessingOpcode::from_arm_opcode(opcode) else {
                                diagnostics.push(RecompileError::UnsupportedInstruction {
                                    address: pc,
                                    opcode: raw,
                                    instruction_set,
                                });
                                terminator = BlockTerminator::Unsupported;
                                break;
                            };
                            if opcode.writes_destination() && rd == 15 {
                                diagnostics.push(RecompileError::UnresolvedIndirectControlFlow {
                                    instruction_address: pc,
                                });
                                terminator = BlockTerminator::Indirect;
                                break;
                            }
                            instructions.push(IrInstruction {
                                address: pc,
                                raw,
                                instruction_set,
                                operation: IrOperation::ArmDataProcessingImmediate {
                                    condition,
                                    opcode,
                                    set_flags,
                                    rn,
                                    rd,
                                    value,
                                    shifter_carry,
                                },
                            });
                            pc = pc.wrapping_add(4);
                        }
                        arm::ArmInstruction::Branch {
                            condition,
                            link,
                            offset,
                        } => {
                            let target = pc.wrapping_add(8).wrapping_add(offset as u32);
                            let fallthrough = pc.wrapping_add(4);
                            let needs_fallthrough = link || condition != 0xE;

                            enqueue_static_target(
                                rom,
                                &mut pending,
                                &mut diagnostics,
                                pc,
                                target,
                                InstructionSet::Arm,
                            );
                            if needs_fallthrough {
                                enqueue_static_target(
                                    rom,
                                    &mut pending,
                                    &mut diagnostics,
                                    pc,
                                    fallthrough,
                                    InstructionSet::Arm,
                                );
                            }

                            terminator = BlockTerminator::Branch {
                                condition,
                                target,
                                instruction_set: InstructionSet::Arm,
                                link,
                                return_address: fallthrough,
                                fallthrough: needs_fallthrough.then_some(fallthrough),
                            };
                            break;
                        }
                        arm::ArmInstruction::BranchExchange { .. } => {
                            diagnostics.push(RecompileError::UnresolvedIndirectControlFlow {
                                instruction_address: pc,
                            });
                            terminator = BlockTerminator::Indirect;
                            break;
                        }
                        arm::ArmInstruction::SoftwareInterrupt { .. }
                        | arm::ArmInstruction::Unsupported => {
                            diagnostics.push(RecompileError::UnsupportedInstruction {
                                address: pc,
                                opcode: raw,
                                instruction_set,
                            });
                            terminator = BlockTerminator::Unsupported;
                            break;
                        }
                    }
                    continue;
                }
                InstructionSet::Thumb => {
                    let Some(raw16) = read_u16(rom, pc) else {
                        diagnostics.push(unresolved_edge(pc, pc));
                        terminator = BlockTerminator::Unsupported;
                        break;
                    };
                    let instruction = thumb::decode(raw16);
                    decoded_addresses.insert(pc, instruction_set);
                    instruction_count += 1;

                    match instruction {
                        thumb::ThumbInstruction::AluImmediate {
                            operation,
                            rd,
                            immediate,
                        } => {
                            let operation = match operation {
                                thumb::ThumbImmediateOp::Move => ThumbImmediateOpcode::Move,
                                thumb::ThumbImmediateOp::Compare => ThumbImmediateOpcode::Compare,
                                thumb::ThumbImmediateOp::Add => ThumbImmediateOpcode::Add,
                                thumb::ThumbImmediateOp::Subtract => ThumbImmediateOpcode::Subtract,
                            };
                            instructions.push(IrInstruction {
                                address: pc,
                                raw: u32::from(raw16),
                                instruction_set,
                                operation: IrOperation::ThumbAluImmediate {
                                    operation,
                                    rd,
                                    immediate,
                                },
                            });
                            pc = pc.wrapping_add(2);
                        }
                        thumb::ThumbInstruction::Branch { offset } => {
                            let target = pc.wrapping_add(4).wrapping_add(offset as u32);
                            enqueue_static_target(
                                rom,
                                &mut pending,
                                &mut diagnostics,
                                pc,
                                target,
                                InstructionSet::Thumb,
                            );
                            terminator = BlockTerminator::Branch {
                                condition: 0xE,
                                target,
                                instruction_set: InstructionSet::Thumb,
                                link: false,
                                return_address: pc.wrapping_add(2),
                                fallthrough: None,
                            };
                            break;
                        }
                        thumb::ThumbInstruction::ConditionalBranch { condition, offset } => {
                            let target = pc.wrapping_add(4).wrapping_add(offset as u32);
                            let fallthrough = pc.wrapping_add(2);
                            enqueue_static_target(
                                rom,
                                &mut pending,
                                &mut diagnostics,
                                pc,
                                target,
                                InstructionSet::Thumb,
                            );
                            enqueue_static_target(
                                rom,
                                &mut pending,
                                &mut diagnostics,
                                pc,
                                fallthrough,
                                InstructionSet::Thumb,
                            );
                            terminator = BlockTerminator::ConditionalBranch {
                                condition,
                                target,
                                fallthrough,
                                instruction_set: InstructionSet::Thumb,
                            };
                            break;
                        }
                        thumb::ThumbInstruction::BranchExchange { .. } => {
                            diagnostics.push(RecompileError::UnresolvedIndirectControlFlow {
                                instruction_address: pc,
                            });
                            terminator = BlockTerminator::Indirect;
                            break;
                        }
                        thumb::ThumbInstruction::SoftwareInterrupt { .. }
                        | thumb::ThumbInstruction::LongBranchPrefix { .. }
                        | thumb::ThumbInstruction::LongBranchSuffix { .. }
                        | thumb::ThumbInstruction::Unsupported => {
                            diagnostics.push(RecompileError::UnsupportedInstruction {
                                address: pc,
                                opcode: u32::from(raw16),
                                instruction_set,
                            });
                            terminator = BlockTerminator::Unsupported;
                            break;
                        }
                    }
                    continue;
                }
            }
        }

        blocks.push(BasicBlock {
            start,
            instruction_set,
            instructions,
            terminator,
        });
    }

    AnalysisReport {
        blocks,
        diagnostics,
        instruction_count,
        instruction_limit: options.max_instructions,
        reached_instruction_limit,
    }
}

fn instruction_width(instruction_set: InstructionSet) -> usize {
    match instruction_set {
        InstructionSet::Arm => 4,
        InstructionSet::Thumb => 2,
    }
}

fn address_has_bytes(rom: &GbaRom, address: u32, width: usize) -> bool {
    (0..width).all(|offset| {
        address
            .checked_add(offset as u32)
            .is_some_and(|byte_address| rom.read_byte(byte_address).is_ok())
    })
}

fn read_u16(rom: &GbaRom, address: u32) -> Option<u16> {
    let first = rom.read_byte(address).ok()?;
    let second = rom.read_byte(address.checked_add(1)?).ok()?;
    Some(u16::from_le_bytes([first, second]))
}

fn read_u32(rom: &GbaRom, address: u32) -> Option<u32> {
    let first = rom.read_byte(address).ok()?;
    let second = rom.read_byte(address.checked_add(1)?).ok()?;
    let third = rom.read_byte(address.checked_add(2)?).ok()?;
    let fourth = rom.read_byte(address.checked_add(3)?).ok()?;
    Some(u32::from_le_bytes([first, second, third, fourth]))
}

fn enqueue_static_target(
    rom: &GbaRom,
    pending: &mut VecDeque<(u32, InstructionSet)>,
    diagnostics: &mut Vec<RecompileError>,
    instruction_address: u32,
    target_address: u32,
    instruction_set: InstructionSet,
) {
    if address_has_bytes(rom, target_address, instruction_width(instruction_set)) {
        pending.push_back((target_address, instruction_set));
    } else {
        diagnostics.push(unresolved_edge(instruction_address, target_address));
    }
}

fn unresolved_edge(instruction_address: u32, target_address: u32) -> RecompileError {
    RecompileError::UnresolvedStaticTarget {
        instruction_address,
        target_address,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::{GBA_HEADER_LEN, GbaRom};

    fn synthetic_rom(code: &[(usize, u32)]) -> GbaRom {
        let mut bytes = vec![0_u8; GBA_HEADER_LEN + 0x100];
        for (offset, word) in code {
            bytes[*offset..*offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes[0xB2] = 0x96;
        bytes[0xBD] = 0_u8.wrapping_sub(
            bytes[0xA0..=0xBC]
                .iter()
                .fold(0x19_u8, |sum, byte| sum.wrapping_add(*byte)),
        );
        GbaRom::from_bytes(bytes).expect("synthetic fixture parses")
    }

    #[test]
    fn follows_entry_branch_to_a_known_instruction() {
        let rom = synthetic_rom(&[
            (0, 0xEA00_002E),                  // 0x08000000 -> 0x080000C0
            (GBA_HEADER_LEN, 0xE3A0_0001),     // MOV r0, #1
            (GBA_HEADER_LEN + 4, 0xEAFF_FFFD), // B back to block start
        ]);
        let report = analyze(
            &rom,
            AnalysisOptions {
                max_instructions: 8,
            },
        );

        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        assert_eq!(report.instruction_count, 3);
        assert!(report.blocks.iter().any(|block| block.start == 0x0800_00C0));
    }

    #[test]
    fn rejects_indirect_control_flow_instead_of_inventing_a_target() {
        let rom = synthetic_rom(&[
            (0, 0xEA00_002E),
            (GBA_HEADER_LEN, 0xE12F_FF10), // BX r0
        ]);
        let report = analyze(
            &rom,
            AnalysisOptions {
                max_instructions: 8,
            },
        );

        assert!(report.diagnostics.iter().any(|error| matches!(
            error,
            RecompileError::UnresolvedIndirectControlFlow {
                instruction_address: 0x0800_00C0
            }
        )));
    }

    #[test]
    fn refuses_to_claim_complete_discovery_when_budget_is_exhausted() {
        let rom = synthetic_rom(&[(0, 0xEA00_002E), (GBA_HEADER_LEN, 0xE3A0_0001)]);
        let report = analyze(
            &rom,
            AnalysisOptions {
                max_instructions: 1,
            },
        );
        assert!(report.reached_instruction_limit);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|error| matches!(error, RecompileError::InstructionLimitReached { limit: 1 }))
        );
    }
}
