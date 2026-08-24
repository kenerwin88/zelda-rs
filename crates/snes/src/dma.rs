//! DMA + HDMA. Port of `zelda3/snes/dma.c`.

use crate::snes::Snes;

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct DmaChannel {
    pub b_adr: u8,
    pub a_bank: u8,
    pub ind_bank: u8,
    pub rep_count: u8,
    pub a_adr: u16,
    pub size: u16, // also indirect hdma adr
    pub table_adr: u16,
    pub unused_byte: u8,
    pub dma_active: bool,
    pub hdma_active: bool,
    pub mode: u8,
    pub fixed: bool,
    pub decrement: bool,
    pub indirect: bool,
    pub from_b: bool,
    pub unused_bit: bool,
    pub do_transfer: bool,
    pub terminated: bool,
    pub off_index: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DmaState {
    pub channel: [DmaChannel; 8],
    pub hdma_timer: u16,
    pub dma_timer: u32,
    pub dma_busy: bool,
}

impl Default for DmaState {
    fn default() -> Self {
        let mut s = Self {
            channel: [DmaChannel::default(); 8],
            hdma_timer: 0,
            dma_timer: 0,
            dma_busy: false,
        };
        s.reset();
        s
    }
}

impl DmaState {
    pub const C_SAVELOAD_SIZE: usize = 192;

    pub fn new() -> Self {
        Self::default()
    }

    /// Matches `dma_reset`. Channel registers power up with all-ones in
    /// most fields and `mode = 7`.
    pub fn reset(&mut self) {
        for c in &mut self.channel {
            c.b_adr = 0xff;
            c.a_adr = 0xffff;
            c.a_bank = 0xff;
            c.size = 0xffff;
            c.ind_bank = 0xff;
            c.table_adr = 0xffff;
            c.rep_count = 0xff;
            c.unused_byte = 0xff;
            c.dma_active = false;
            c.hdma_active = false;
            c.mode = 7;
            c.fixed = true;
            c.decrement = true;
            c.indirect = true;
            c.from_b = true;
            c.unused_bit = true;
            c.do_transfer = false;
            c.terminated = false;
            c.off_index = 0;
        }
        self.hdma_timer = 0;
        self.dma_timer = 0;
        self.dma_busy = false;
    }

    /// Byte layout used by C `dma_saveload`.
    ///
    /// The C routine serializes from `Dma.channel` through the end of `Dma`,
    /// including ABI padding after `hdmaTimer` and after `dmaBusy`.
    pub fn save_c_saveload(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::C_SAVELOAD_SIZE);
        for ch in &self.channel {
            out.push(ch.b_adr);
            out.push(ch.a_bank);
            out.push(ch.ind_bank);
            out.push(ch.rep_count);
            out.extend_from_slice(&ch.a_adr.to_le_bytes());
            out.extend_from_slice(&ch.size.to_le_bytes());
            out.extend_from_slice(&ch.table_adr.to_le_bytes());
            out.push(ch.unused_byte);
            out.push(ch.dma_active as u8);
            out.push(ch.hdma_active as u8);
            out.push(ch.mode);
            out.push(ch.fixed as u8);
            out.push(ch.decrement as u8);
            out.push(ch.indirect as u8);
            out.push(ch.from_b as u8);
            out.push(ch.unused_bit as u8);
            out.push(ch.do_transfer as u8);
            out.push(ch.terminated as u8);
            out.push(ch.off_index);
        }
        out.extend_from_slice(&self.hdma_timer.to_le_bytes());
        out.extend_from_slice(&[0; 2]);
        out.extend_from_slice(&self.dma_timer.to_le_bytes());
        out.push(self.dma_busy as u8);
        out.extend_from_slice(&[0; 7]);
        debug_assert_eq!(out.len(), Self::C_SAVELOAD_SIZE);
        out
    }

    pub fn load_c_saveload(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != Self::C_SAVELOAD_SIZE {
            return Err(format!(
                "invalid DMA saveload size {}, expected {}",
                data.len(),
                Self::C_SAVELOAD_SIZE
            ));
        }
        let mut pos = 0usize;
        for ch in &mut self.channel {
            ch.b_adr = data[pos];
            ch.a_bank = data[pos + 1];
            ch.ind_bank = data[pos + 2];
            ch.rep_count = data[pos + 3];
            ch.a_adr = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);
            ch.size = u16::from_le_bytes([data[pos + 6], data[pos + 7]]);
            ch.table_adr = u16::from_le_bytes([data[pos + 8], data[pos + 9]]);
            ch.unused_byte = data[pos + 10];
            ch.dma_active = data[pos + 11] != 0;
            ch.hdma_active = data[pos + 12] != 0;
            ch.mode = data[pos + 13];
            ch.fixed = data[pos + 14] != 0;
            ch.decrement = data[pos + 15] != 0;
            ch.indirect = data[pos + 16] != 0;
            ch.from_b = data[pos + 17] != 0;
            ch.unused_bit = data[pos + 18] != 0;
            ch.do_transfer = data[pos + 19] != 0;
            ch.terminated = data[pos + 20] != 0;
            ch.off_index = data[pos + 21];
            pos += 22;
        }
        self.hdma_timer = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 4;
        self.dma_timer =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;
        self.dma_busy = data[pos] != 0;
        Ok(())
    }
}

/// $43x0..$43xf register table.
const B_ADR_OFFSETS: [[u8; 4]; 8] = [
    [0, 0, 0, 0],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
    [0, 1, 2, 3],
    [0, 1, 0, 1],
    [0, 0, 0, 0],
    [0, 0, 1, 1],
];

const TRANSFER_LENGTH: [u8; 8] = [1, 2, 2, 4, 4, 4, 2, 4];

/// One source-ordered general-DMA bus semantic. Timing is deliberately owned
/// by the synchronous CPU timeline rather than by the legacy DMA timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SynchronousGeneralDmaByte {
    pub channel: u8,
    pub a_address: u32,
    pub b_address: u8,
    pub from_b: bool,
    /// The one pinned fast-path assignment to `OpenBus`: mode 1/5, B=$18,
    /// high-byte phase. The synchronous owner additionally verifies that the
    /// A-bus address uses a direct base-pointer map.
    pub publishes_fast_vram_high_open_bus: bool,
    pub a_bus_is_wram: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum SynchronousGeneralDmaUnsupported {
    #[error("general DMA channel {channel} collides with active HDMA")]
    ActiveHdma { channel: u8 },
    #[error("general DMA channel {channel} touches the fallible APUI synchronization path")]
    ApuBus { channel: u8 },
    #[error(
        "general DMA channel {channel} maps its A-bus through an unimplemented low-bank I/O range"
    )]
    CpuOrPpuRegisterBus { channel: u8 },
}

impl Snes {
    pub(crate) fn validate_synchronous_general_dma(
        &self,
        mask: u8,
    ) -> Result<(), SynchronousGeneralDmaUnsupported> {
        for channel in 0..8u8 {
            if mask & (1 << channel) == 0 {
                continue;
            }
            let dma = &self.dma.channel[usize::from(channel)];
            if dma.hdma_active {
                return Err(SynchronousGeneralDmaUnsupported::ActiveHdma { channel });
            }
            let phases = if dma.size == 0 {
                u32::from(u16::MAX) + 1
            } else {
                u32::from(dma.size)
            };
            let mut a_address = dma.a_adr;
            for transfer_index in 0..phases {
                let b_phase = transfer_index as usize & 3;
                let b_address = dma
                    .b_adr
                    .wrapping_add(B_ADR_OFFSETS[usize::from(dma.mode)][b_phase]);
                if (0x40..0x80).contains(&b_address) {
                    return Err(SynchronousGeneralDmaUnsupported::ApuBus { channel });
                }
                let full_address = (u32::from(dma.a_bank) << 16) | u32::from(a_address);
                if Snes::synchronous_cpu_apu_port(full_address).is_some() {
                    return Err(SynchronousGeneralDmaUnsupported::ApuBus { channel });
                }
                let bank = dma.a_bank & 0x7f;
                if bank < 0x40 && matches!(a_address, 0x2000..=0x5fff) {
                    return Err(SynchronousGeneralDmaUnsupported::CpuOrPpuRegisterBus { channel });
                }
                if !dma.fixed {
                    a_address = if dma.decrement {
                        a_address.wrapping_sub(1)
                    } else {
                        a_address.wrapping_add(1)
                    };
                }
            }
        }
        Ok(())
    }

    /// Latch `$420b`'s selected general-DMA channels without charging timing.
    /// Pinned Snes9x charges the CPU-sync and channel/byte clocks around these
    /// register semantics in `S9xSetCPU`/`S9xDoDMA`.
    pub(crate) fn synchronous_general_dma_begin(&mut self, _mask: u8) {
        // The selected mask is continuation ownership, not a source CPU flag:
        // `CPU.InDMA` remains false during the initial DMACPUSync charge.
        self.dma.dma_busy = false;
    }

    pub(crate) fn synchronous_general_dma_begin_channel(&mut self, channel: u8) {
        self.dma.channel[usize::from(channel)].dma_active = true;
        self.dma.dma_busy = true;
    }

    /// Begin one selected channel after its source eight-clock setup charge.
    /// Describe the next A/B-bus semantic from the live generic DMA registers.
    pub(crate) fn synchronous_general_dma_next_byte(
        &self,
        channel: u8,
        b_phase: u8,
    ) -> SynchronousGeneralDmaByte {
        let channel_index = usize::from(channel);
        let dma = &self.dma.channel[channel_index];
        let bias = B_ADR_OFFSETS[usize::from(dma.mode)][usize::from(b_phase)];
        SynchronousGeneralDmaByte {
            channel,
            a_address: (u32::from(dma.a_bank) << 16) | u32::from(dma.a_adr),
            b_address: dma.b_adr.wrapping_add(bias),
            from_b: dma.from_b,
            publishes_fast_vram_high_open_bus: !dma.from_b
                && matches!(dma.mode, 1 | 5)
                && dma.b_adr == 0x18
                && b_phase & 1 != 0,
            a_bus_is_wram: matches!(dma.a_bank, 0x7e | 0x7f)
                || (dma.a_bank & 0x40 == 0 && dma.a_adr < 0x2000),
        }
    }

    /// Commit the counter updates which precede `addCyclesInDMA` in the pinned
    /// `UPDATE_COUNTERS` macro. A zero initial size naturally wraps and means
    /// 65,536 bytes, exactly as the source does.
    pub(crate) fn synchronous_general_dma_commit_byte(&mut self, channel: u8) -> bool {
        let dma = &mut self.dma.channel[usize::from(channel)];
        if !dma.from_b && matches!(dma.a_bank, 0x7e | 0x7f) && dma.b_adr == 0x80 {
            // dma.cpp's invalid WRAM->$2180 branch advances A unconditionally.
            dma.a_adr = dma.a_adr.wrapping_add(1);
        } else if !dma.fixed {
            dma.a_adr = if dma.decrement {
                dma.a_adr.wrapping_sub(1)
            } else {
                dma.a_adr.wrapping_add(1)
            };
        }
        dma.size = dma.size.wrapping_sub(1);
        dma.size == 0
    }

    pub(crate) fn synchronous_general_dma_finish_channel(&mut self, channel: u8) {
        let dma = &mut self.dma.channel[usize::from(channel)];
        dma.dma_active = false;
        self.dma.dma_busy = false;
    }

    pub(crate) fn synchronous_general_dma_skips_nmi_retime(&self, channel: u8) -> bool {
        let dma = &self.dma.channel[usize::from(channel)];
        !dma.from_b && matches!(dma.a_bank, 0x7e | 0x7f) && dma.b_adr == 0x80
    }

    pub(crate) fn synchronous_general_dma_finish_all(&mut self) {
        self.dma.dma_busy = false;
    }

    pub(crate) fn dma_read_reg(&self, adr: u16) -> u8 {
        let c = ((adr & 0x70) >> 4) as usize;
        let ch = &self.dma.channel[c];
        match adr & 0xf {
            0x0 => {
                ch.mode
                    | (ch.fixed as u8) << 3
                    | (ch.decrement as u8) << 4
                    | (ch.unused_bit as u8) << 5
                    | (ch.indirect as u8) << 6
                    | (ch.from_b as u8) << 7
            }
            0x1 => ch.b_adr,
            0x2 => ch.a_adr as u8,
            0x3 => (ch.a_adr >> 8) as u8,
            0x4 => ch.a_bank,
            0x5 => ch.size as u8,
            0x6 => (ch.size >> 8) as u8,
            0x7 => ch.ind_bank,
            0x8 => ch.table_adr as u8,
            0x9 => (ch.table_adr >> 8) as u8,
            0xa => ch.rep_count,
            0xb | 0xf => ch.unused_byte,
            _ => self.open_bus,
        }
    }

    pub(crate) fn dma_write_reg(&mut self, adr: u16, val: u8) {
        let c = ((adr & 0x70) >> 4) as usize;
        let ch = &mut self.dma.channel[c];
        match adr & 0xf {
            0x0 => {
                ch.mode = val & 0x7;
                ch.fixed = val & 0x8 != 0;
                ch.decrement = val & 0x10 != 0;
                ch.unused_bit = val & 0x20 != 0;
                ch.indirect = val & 0x40 != 0;
                ch.from_b = val & 0x80 != 0;
            }
            0x1 => ch.b_adr = val,
            0x2 => ch.a_adr = (ch.a_adr & 0xff00) | val as u16,
            0x3 => ch.a_adr = (ch.a_adr & 0x00ff) | ((val as u16) << 8),
            0x4 => ch.a_bank = val,
            0x5 => ch.size = (ch.size & 0xff00) | val as u16,
            0x6 => ch.size = (ch.size & 0x00ff) | ((val as u16) << 8),
            0x7 => ch.ind_bank = val,
            0x8 => ch.table_adr = (ch.table_adr & 0xff00) | val as u16,
            0x9 => ch.table_adr = (ch.table_adr & 0x00ff) | ((val as u16) << 8),
            0xa => ch.rep_count = val,
            0xb | 0xf => ch.unused_byte = val,
            _ => {}
        }
    }

    /// `dma_startDma` — for non-HDMA this latches `dma_busy` and adds
    /// the 16-cycle setup overhead.
    pub(crate) fn dma_start_real(&mut self, val: u8, hdma: bool) {
        for i in 0..8 {
            if hdma {
                self.dma.channel[i].hdma_active = val & (1 << i) != 0;
            } else {
                self.dma.channel[i].dma_active = val & (1 << i) != 0;
            }
        }
        if !hdma {
            self.dma.dma_busy = val != 0;
            if self.dma.dma_busy {
                self.dma.dma_timer = self.dma.dma_timer.wrapping_add(16);
            }
        }
    }

    fn dma_transfer_byte(&mut self, a_adr: u16, a_bank: u8, b_adr: u8, from_b: bool) {
        let full = ((a_bank as u32) << 16) | a_adr as u32;
        if from_b {
            let v = self.read_b_bus(b_adr);
            self.write(full, v);
        } else {
            let v = self.read(full);
            self.write_b_bus(b_adr, v);
        }
    }

    pub fn dma_do(&mut self) {
        let mut i = 0;
        while i < 8 {
            if self.dma.channel[i].dma_active {
                break;
            }
            i += 1;
        }
        if i == 8 {
            self.dma.dma_busy = false;
            return;
        }

        let mode = self.dma.channel[i].mode as usize;
        let off = self.dma.channel[i].off_index as usize;
        let bias = B_ADR_OFFSETS[mode][off];
        let a_adr = self.dma.channel[i].a_adr;
        let a_bank = self.dma.channel[i].a_bank;
        let b_adr = self.dma.channel[i].b_adr.wrapping_add(bias);
        let from_b = self.dma.channel[i].from_b;

        self.dma.channel[i].off_index = (self.dma.channel[i].off_index + 1) & 3;
        self.dma_transfer_byte(a_adr, a_bank, b_adr, from_b);

        self.dma.dma_timer = self.dma.dma_timer.wrapping_add(6);
        if !self.dma.channel[i].fixed {
            self.dma.channel[i].a_adr = if self.dma.channel[i].decrement {
                self.dma.channel[i].a_adr.wrapping_sub(1)
            } else {
                self.dma.channel[i].a_adr.wrapping_add(1)
            };
        }
        self.dma.channel[i].size = self.dma.channel[i].size.wrapping_sub(1);
        if self.dma.channel[i].size == 0 {
            self.dma.channel[i].off_index = 0;
            self.dma.channel[i].dma_active = false;
            self.dma.dma_timer = self.dma.dma_timer.wrapping_add(8);
        }
    }

    pub fn dma_init_hdma(&mut self) {
        self.dma.hdma_timer = 0;
        let mut any = false;
        for i in 0..8 {
            if self.dma.channel[i].hdma_active {
                any = true;
                self.dma.channel[i].dma_active = false;
                self.dma.channel[i].off_index = 0;
                self.dma.channel[i].table_adr = self.dma.channel[i].a_adr;

                let bank = self.dma.channel[i].a_bank;
                let addr = self.dma.channel[i].table_adr;
                let rep = self.read(((bank as u32) << 16) | addr as u32);
                self.dma.channel[i].rep_count = rep;
                self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(8);

                if self.dma.channel[i].indirect {
                    let lo_addr = self.dma.channel[i].table_adr;
                    let lo = self.read(((bank as u32) << 16) | lo_addr as u32);
                    self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                    let hi_addr = self.dma.channel[i].table_adr;
                    let hi = self.read(((bank as u32) << 16) | hi_addr as u32);
                    self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                    self.dma.channel[i].size = lo as u16 | ((hi as u16) << 8);
                    self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(16);
                }
                self.dma.channel[i].do_transfer = true;
            } else {
                self.dma.channel[i].do_transfer = false;
            }
            self.dma.channel[i].terminated = false;
        }
        if any {
            self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(16);
        }
    }

    pub fn dma_do_hdma(&mut self) {
        self.dma.hdma_timer = 0;
        let mut any = false;
        for i in 0..8 {
            if !(self.dma.channel[i].hdma_active && !self.dma.channel[i].terminated) {
                continue;
            }
            any = true;
            self.dma.channel[i].dma_active = false;
            self.dma.channel[i].off_index = 0;
            self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(8);

            if self.dma.channel[i].do_transfer {
                let len = TRANSFER_LENGTH[self.dma.channel[i].mode as usize];
                for j in 0..len {
                    self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(8);
                    let bias = B_ADR_OFFSETS[self.dma.channel[i].mode as usize][j as usize];
                    let b_adr = self.dma.channel[i].b_adr.wrapping_add(bias);
                    let from_b = self.dma.channel[i].from_b;
                    if self.dma.channel[i].indirect {
                        let size_now = self.dma.channel[i].size;
                        self.dma.channel[i].size = size_now.wrapping_add(1);
                        self.dma_transfer_byte(
                            size_now,
                            self.dma.channel[i].ind_bank,
                            b_adr,
                            from_b,
                        );
                    } else {
                        let table = self.dma.channel[i].table_adr;
                        self.dma.channel[i].table_adr = table.wrapping_add(1);
                        self.dma_transfer_byte(table, self.dma.channel[i].a_bank, b_adr, from_b);
                    }
                }
            }
            self.dma.channel[i].rep_count = self.dma.channel[i].rep_count.wrapping_sub(1);
            self.dma.channel[i].do_transfer = self.dma.channel[i].rep_count & 0x80 != 0;

            if self.dma.channel[i].rep_count & 0x7f == 0 {
                let bank = self.dma.channel[i].a_bank;
                let addr = self.dma.channel[i].table_adr;
                let next = self.read(((bank as u32) << 16) | addr as u32);
                self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                self.dma.channel[i].rep_count = next;
                if self.dma.channel[i].indirect {
                    let lo_addr = self.dma.channel[i].table_adr;
                    let lo = self.read(((bank as u32) << 16) | lo_addr as u32);
                    self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                    let hi_addr = self.dma.channel[i].table_adr;
                    let hi = self.read(((bank as u32) << 16) | hi_addr as u32);
                    self.dma.channel[i].table_adr = self.dma.channel[i].table_adr.wrapping_add(1);
                    self.dma.channel[i].size = lo as u16 | ((hi as u16) << 8);
                    self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(16);
                }
                if self.dma.channel[i].rep_count == 0 {
                    self.dma.channel[i].terminated = true;
                }
                self.dma.channel[i].do_transfer = true;
            }
        }
        if any {
            self.dma.hdma_timer = self.dma.hdma_timer.wrapping_add(16);
        }
    }

    pub fn dma_cycle(&mut self) -> bool {
        if self.dma.hdma_timer > 0 {
            self.dma.hdma_timer = self.dma.hdma_timer.wrapping_sub(2);
            true
        } else if self.dma.dma_busy {
            self.dma_do();
            true
        } else {
            false
        }
    }

    /// Complete a latched general DMA and return the CPU stall in master
    /// cycles.
    ///
    /// The pinned Snes9x 1.63 scheduler charges its 18-cycle CPU/DMA sync,
    /// then 8 cycles for each byte and 8 for each active channel. `dma_do`
    /// already performs the transfer byte-for-byte; this helper supplies the
    /// scheduler accounting needed by read/write-isolated CPU timing runs.
    pub fn dma_run_to_completion_master_cycles(&mut self) -> u32 {
        if !self.dma.dma_busy {
            return 0;
        }

        let mut master_cycles = 18u32;
        loop {
            let Some(channel) = self
                .dma
                .channel
                .iter()
                .position(|channel| channel.dma_active)
            else {
                self.dma.dma_busy = false;
                self.dma.dma_timer = 0;
                return master_cycles;
            };
            let completes_channel = self.dma.channel[channel].size == 1;
            self.dma_do();
            master_cycles = master_cycles.checked_add(8).expect("DMA timing overflowed");
            if completes_channel {
                master_cycles = master_cycles.checked_add(8).expect("DMA timing overflowed");
            }
        }
    }
}

#[cfg(test)]
mod saveload_tests {
    use super::*;

    #[test]
    fn c_saveload_layout_matches_dma_c_abi_size() {
        let mut dma = DmaState::new();
        dma.channel[0].b_adr = 0x21;
        dma.channel[0].a_adr = 0x3456;
        dma.channel[0].dma_active = true;
        dma.channel[7].off_index = 3;
        dma.hdma_timer = 0x1234;
        dma.dma_timer = 0x89abcdef;
        dma.dma_busy = true;

        let bytes = dma.save_c_saveload();
        assert_eq!(bytes.len(), DmaState::C_SAVELOAD_SIZE);
        assert_eq!(bytes[0], 0x21);
        assert_eq!(&bytes[4..6], &0x3456u16.to_le_bytes());
        assert_eq!(bytes[11], 1);
        assert_eq!(bytes[7 * 22 + 21], 3);
        assert_eq!(&bytes[176..178], &0x1234u16.to_le_bytes());
        assert_eq!(&bytes[180..184], &0x89abcdefu32.to_le_bytes());
        assert_eq!(bytes[184], 1);

        let mut loaded = DmaState::new();
        loaded.load_c_saveload(&bytes).unwrap();
        assert_eq!(loaded.channel[0].b_adr, 0x21);
        assert_eq!(loaded.channel[0].a_adr, 0x3456);
        assert!(loaded.channel[0].dma_active);
        assert_eq!(loaded.channel[7].off_index, 3);
        assert_eq!(loaded.hdma_timer, 0x1234);
        assert_eq!(loaded.dma_timer, 0x89abcdef);
        assert!(loaded.dma_busy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_initializes_channels_to_ones() {
        let mut s = DmaState::new();
        s.reset();
        assert_eq!(s.channel[0].b_adr, 0xff);
        assert_eq!(s.channel[7].size, 0xffff);
        assert_eq!(s.channel[3].mode, 7);
        assert!(s.channel[2].fixed);
        assert!(!s.dma_busy);
    }

    #[test]
    fn dma_register_round_trip() {
        let mut snes = Snes::new();
        // write mode/flags to $4300
        snes.write(0x0000_4300, 0b1010_0011);
        let v = snes.read(0x0000_4300);
        assert_eq!(v & 0x07, 3); // mode
        assert!(v & 0x20 != 0); // unused_bit was set
        assert!(v & 0x80 != 0); // from_b
                                // bAdr at $4301
        snes.write(0x0000_4301, 0x18);
        assert_eq!(snes.read(0x0000_4301), 0x18);
    }

    #[test]
    fn general_dma_completion_reports_scheduler_master_cycles() {
        let mut snes = Snes::new();
        snes.dma.channel[0].a_bank = 0x7e;
        snes.dma.channel[0].a_adr = 0x0100;
        snes.dma.channel[0].b_adr = 0;
        snes.dma.channel[0].size = 2;
        snes.dma.channel[0].fixed = false;
        snes.dma.channel[0].decrement = false;
        snes.dma.channel[1].a_bank = 0x7e;
        snes.dma.channel[1].a_adr = 0x0200;
        snes.dma.channel[1].b_adr = 0;
        snes.dma.channel[1].size = 1;
        snes.dma.channel[1].fixed = false;
        snes.dma.channel[1].decrement = false;
        snes.dma_start_real(0b0000_0011, false);

        assert_eq!(snes.dma_run_to_completion_master_cycles(), 58);
        assert!(!snes.dma.dma_busy);
        assert_eq!(snes.dma.channel[0].a_adr, 0x0102);
        assert_eq!(snes.dma.channel[1].a_adr, 0x0201);
        assert_eq!(snes.dma_run_to_completion_master_cycles(), 0);
    }

    #[test]
    fn synchronous_general_dma_uses_source_mode_patterns_and_local_phase() {
        let expected = [
            [0, 0, 0, 0],
            [0, 1, 0, 1],
            [0, 0, 0, 0],
            [0, 0, 1, 1],
            [0, 1, 2, 3],
            [0, 1, 0, 1],
            [0, 0, 0, 0],
            [0, 0, 1, 1],
        ];
        for mode in 0..8u8 {
            let mut snes = Snes::new();
            let dma = &mut snes.dma.channel[0];
            dma.mode = mode;
            dma.b_adr = 0x20;
            for phase in 0..4u8 {
                assert_eq!(
                    snes.synchronous_general_dma_next_byte(0, phase).b_address,
                    0x20 + expected[usize::from(mode)][usize::from(phase)]
                );
            }
        }
    }

    #[test]
    fn synchronous_zero_size_wraps_to_exactly_65536_byte_counter_domain() {
        let mut snes = Snes::new();
        snes.dma.channel[0].size = 0;
        snes.dma.channel[0].a_adr = 0xffff;
        snes.dma.channel[0].fixed = false;
        snes.dma.channel[0].decrement = false;

        assert!(!snes.synchronous_general_dma_commit_byte(0));
        assert_eq!(snes.dma.channel[0].size, 0xffff);
        assert_eq!(snes.dma.channel[0].a_adr, 0);
        snes.dma.channel[0].size = 1;
        assert!(snes.synchronous_general_dma_commit_byte(0));
        assert_eq!(snes.dma.channel[0].size, 0);
    }
}
