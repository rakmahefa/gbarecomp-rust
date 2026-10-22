#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum InstructionSet {
    Arm,
    Thumb,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CpuState {
    pub regs: [u32; 16],
    pub cpsr: u32,
    pub instruction_set: InstructionSet,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            regs: [0; 16],
            cpsr: 0x1F,
            instruction_set: InstructionSet::Arm,
        }
    }
}

impl CpuState {
    pub const FLAG_N: u32 = 1 << 31;
    pub const FLAG_Z: u32 = 1 << 30;
    pub const FLAG_C: u32 = 1 << 29;
    pub const FLAG_V: u32 = 1 << 28;
    pub const FLAG_T: u32 = 1 << 5;

    pub fn pc_operand(&self, instruction_address: u32) -> u32 {
        let pipeline_offset = match self.instruction_set {
            InstructionSet::Arm => 8,
            InstructionSet::Thumb => 4,
        };
        instruction_address.wrapping_add(pipeline_offset)
    }

    pub fn set_instruction_set(&mut self, instruction_set: InstructionSet) {
        self.instruction_set = instruction_set;
        match instruction_set {
            InstructionSet::Arm => self.cpsr &= !Self::FLAG_T,
            InstructionSet::Thumb => self.cpsr |= Self::FLAG_T,
        }
    }

    pub fn condition_passed(&self, condition: u8) -> bool {
        let n = self.cpsr & Self::FLAG_N != 0;
        let z = self.cpsr & Self::FLAG_Z != 0;
        let c = self.cpsr & Self::FLAG_C != 0;
        let v = self.cpsr & Self::FLAG_V != 0;

        match condition & 0x0F {
            0x0 => z,
            0x1 => !z,
            0x2 => c,
            0x3 => !c,
            0x4 => n,
            0x5 => !n,
            0x6 => v,
            0x7 => !v,
            0x8 => c && !z,
            0x9 => !c || z,
            0xA => n == v,
            0xB => n != v,
            0xC => !z && n == v,
            0xD => z || n != v,
            0xE => true,
            _ => false,
        }
    }

    pub fn set_nz(&mut self, value: u32) {
        self.cpsr &= !(Self::FLAG_N | Self::FLAG_Z);
        if value & 0x8000_0000 != 0 {
            self.cpsr |= Self::FLAG_N;
        }
        if value == 0 {
            self.cpsr |= Self::FLAG_Z;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_pipeline_depends_on_instruction_set() {
        let mut cpu = CpuState::default();
        assert_eq!(cpu.pc_operand(0x0800_0010), 0x0800_0018);
        cpu.set_instruction_set(InstructionSet::Thumb);
        assert_eq!(cpu.pc_operand(0x0800_0010), 0x0800_0014);
        assert_ne!(cpu.cpsr & CpuState::FLAG_T, 0);
    }

    #[test]
    fn evaluates_common_condition_codes() {
        let mut cpu = CpuState::default();
        cpu.cpsr |= CpuState::FLAG_Z | CpuState::FLAG_C;
        assert!(cpu.condition_passed(0x0));
        assert!(cpu.condition_passed(0x2));
        assert!(cpu.condition_passed(0x9));
        assert!(!cpu.condition_passed(0x1));
        assert!(!cpu.condition_passed(0xF));
    }

    #[test]
    fn updates_nz_without_clobbering_carry_or_overflow() {
        let mut cpu = CpuState::default();
        cpu.cpsr |= CpuState::FLAG_C | CpuState::FLAG_V;
        cpu.set_nz(0);
        assert_ne!(cpu.cpsr & CpuState::FLAG_Z, 0);
        assert_ne!(cpu.cpsr & CpuState::FLAG_C, 0);
        assert_ne!(cpu.cpsr & CpuState::FLAG_V, 0);
        cpu.set_nz(0x8000_0000);
        assert_ne!(cpu.cpsr & CpuState::FLAG_N, 0);
        assert_eq!(cpu.cpsr & CpuState::FLAG_Z, 0);
    }
}
