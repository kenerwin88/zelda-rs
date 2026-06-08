//! 65816 CPU state. Behavior (opcode table, addressing modes) lives in
//! `cpu_step` etc. and is filled in as part of task #3 — this file
//! currently provides only the state and reset, which the bus needs.

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuState {
    // registers
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub sp: u16,
    pub pc: u16,
    pub dp: u16, // direct page (D)
    pub k: u8,   // program bank (PB)
    pub db: u8,  // data bank (B)
    // flags
    pub c: bool,
    pub z: bool,
    pub v: bool,
    pub n: bool,
    pub i: bool,
    pub d: bool,
    pub xf: bool,
    pub mf: bool,
    pub e: bool,
    // interrupts
    pub irq_wanted: bool,
    pub nmi_wanted: bool,
    // power state (WAI/STP)
    pub waiting: bool,
    pub stopped: bool,
    // internal use
    pub cycles_used: u8,
    pub sp_breakpoint: u16,
    pub in_emu: bool,
}

impl CpuState {
    pub const C_SAVELOAD_SIZE: usize = 27;

    pub fn new() -> Self {
        Self::default()
    }

    /// Reset state per `snes/cpu.c:cpu_reset`. The reset vector at $FFFC
    /// is read separately by the bus once the cart is wired up.
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0x100;
        self.pc = 0;
        self.dp = 0;
        self.k = 0;
        self.db = 0;
        self.c = false;
        self.z = false;
        self.v = false;
        self.n = false;
        self.i = true;
        self.d = false;
        self.xf = true;
        self.mf = true;
        self.e = true;
        self.irq_wanted = false;
        self.nmi_wanted = false;
        self.waiting = false;
        self.stopped = false;
        self.cycles_used = 0;
        self.sp_breakpoint = 0;
        self.in_emu = false;
    }

    /// Pack the P flag register the way `cpu_getFlags` does.
    pub fn pack_flags(&self) -> u8 {
        (self.n as u8) << 7
            | (self.v as u8) << 6
            | (self.mf as u8) << 5
            | (self.xf as u8) << 4
            | (self.d as u8) << 3
            | (self.i as u8) << 2
            | (self.z as u8) << 1
            | self.c as u8
    }

    pub fn unpack_flags(&mut self, val: u8) {
        self.n = val & 0x80 != 0;
        self.v = val & 0x40 != 0;
        self.mf = val & 0x20 != 0;
        self.xf = val & 0x10 != 0;
        self.d = val & 0x08 != 0;
        self.i = val & 0x04 != 0;
        self.z = val & 0x02 != 0;
        self.c = val & 0x01 != 0;
    }

    /// Byte layout used by C `cpu_saveload`.
    ///
    /// C serializes from `Cpu.a` up to, but not including, `cyclesUsed`, then
    /// resets `spBreakpoint` to zero after load/save.
    pub fn save_c_saveload(&mut self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::C_SAVELOAD_SIZE);
        out.extend_from_slice(&self.a.to_le_bytes());
        out.extend_from_slice(&self.x.to_le_bytes());
        out.extend_from_slice(&self.y.to_le_bytes());
        out.extend_from_slice(&self.sp.to_le_bytes());
        out.extend_from_slice(&self.pc.to_le_bytes());
        out.extend_from_slice(&self.dp.to_le_bytes());
        out.push(self.k);
        out.push(self.db);
        out.push(self.c as u8);
        out.push(self.z as u8);
        out.push(self.v as u8);
        out.push(self.n as u8);
        out.push(self.i as u8);
        out.push(self.d as u8);
        out.push(self.xf as u8);
        out.push(self.mf as u8);
        out.push(self.e as u8);
        out.push(self.irq_wanted as u8);
        out.push(self.nmi_wanted as u8);
        out.push(self.waiting as u8);
        out.push(self.stopped as u8);
        self.sp_breakpoint = 0;
        debug_assert_eq!(out.len(), Self::C_SAVELOAD_SIZE);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != Self::C_SAVELOAD_SIZE {
            return Err(format!(
                "invalid CPU saveload size {}, expected {}",
                data.len(),
                Self::C_SAVELOAD_SIZE
            ));
        }
        self.a = u16::from_le_bytes([data[0], data[1]]);
        self.x = u16::from_le_bytes([data[2], data[3]]);
        self.y = u16::from_le_bytes([data[4], data[5]]);
        self.sp = u16::from_le_bytes([data[6], data[7]]);
        self.pc = u16::from_le_bytes([data[8], data[9]]);
        self.dp = u16::from_le_bytes([data[10], data[11]]);
        self.k = data[12];
        self.db = data[13];
        self.c = data[14] != 0;
        self.z = data[15] != 0;
        self.v = data[16] != 0;
        self.n = data[17] != 0;
        self.i = data[18] != 0;
        self.d = data[19] != 0;
        self.xf = data[20] != 0;
        self.mf = data[21] != 0;
        self.e = data[22] != 0;
        self.irq_wanted = data[23] != 0;
        self.nmi_wanted = data[24] != 0;
        self.waiting = data[25] != 0;
        self.stopped = data[26] != 0;
        self.sp_breakpoint = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_saveload_layout_matches_cpu_c_range() {
        let mut cpu = CpuState {
            a: 0x1234,
            x: 0x2345,
            y: 0x3456,
            sp: 0x4567,
            pc: 0x5678,
            dp: 0x6789,
            k: 0x12,
            db: 0x34,
            c: true,
            z: false,
            v: true,
            n: false,
            i: true,
            d: false,
            xf: true,
            mf: false,
            e: true,
            irq_wanted: true,
            nmi_wanted: false,
            waiting: true,
            stopped: false,
            cycles_used: 99,
            sp_breakpoint: 0xabcd,
            in_emu: true,
        };

        let bytes = cpu.save_c_saveload();
        assert_eq!(bytes.len(), CpuState::C_SAVELOAD_SIZE);
        assert_eq!(&bytes[0..2], &0x1234u16.to_le_bytes());
        assert_eq!(&bytes[8..10], &0x5678u16.to_le_bytes());
        assert_eq!(bytes[12], 0x12);
        assert_eq!(bytes[22], 1);
        assert_eq!(bytes[26], 0);
        assert_eq!(cpu.sp_breakpoint, 0);

        let mut loaded = CpuState::new();
        loaded.cycles_used = 7;
        loaded.in_emu = true;
        loaded.sp_breakpoint = 0xffff;
        loaded.load_c_saveload(&bytes).unwrap();
        assert_eq!(loaded.a, 0x1234);
        assert_eq!(loaded.pc, 0x5678);
        assert_eq!(loaded.k, 0x12);
        assert!(loaded.c);
        assert!(loaded.e);
        assert!(loaded.irq_wanted);
        assert!(!loaded.stopped);
        assert_eq!(loaded.cycles_used, 7);
        assert!(loaded.in_emu);
        assert_eq!(loaded.sp_breakpoint, 0);
    }
}
