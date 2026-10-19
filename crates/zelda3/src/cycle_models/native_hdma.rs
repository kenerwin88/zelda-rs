//! HDMA events on the retained CPU's live DMA and PPU devices.
use super::{intro_clear::Access, native_work::WorkBus};
use snes::{cpu_timeline::{CpuBusEvent, CpuSynchronousTimelineEvent}, dma::HdmaBusAccess, CpuMasterTimeline, DmaState};

pub(super) struct HdmaBus<'a, B> {
    pub(super) dma: &'a mut DmaState,
    pub(super) bus: B,
}

fn hdma_event(event: CpuSynchronousTimelineEvent) -> Option<CpuBusEvent> {
    match event {
        CpuSynchronousTimelineEvent::Bus(event @ (CpuBusEvent::HdmaInit | CpuBusEvent::HdmaStart)) => Some(event),
        _ => None,
    }
}

impl<B: WorkBus> WorkBus for HdmaBus<'_, B> {
    fn inactive_sprite_body_mask(&mut self) -> u16 { self.bus.inactive_sprite_body_mask() }
    fn observe_clock(&mut self, clock: &CpuMasterTimeline) { self.bus.observe_clock(clock); }
    fn access(&mut self, at: u64, access: Access) -> u16 {
        if let Access::Write(address, value, 1) = access {
            match address {
                0x4300..=0x437f => self.dma.write_register(address as u16, value as u8),
                0x420c => {
                    for (i, channel) in self.dma.channel.iter_mut().enumerate() {
                        channel.hdma_active = value & (1 << i) != 0;
                    }
                }
                _ => {}
            }
        }
        self.bus.access(at, access)
    }
    fn retire_event(&mut self, event: CpuSynchronousTimelineEvent, clock: &CpuMasterTimeline) -> u32 {
        let cycles = self.bus.retire_event(event, clock);
        let Some(event) = hdma_event(event) else { return cycles; };
        self.bus.observe_clock(clock);
        cycles + self.dma.run_synchronous_hdma(event, |access| {
            self.bus.access(clock.clock_master_cycles(), match access {
                HdmaBusAccess::Read(address) => Access::Read(address, 1),
                HdmaBusAccess::Write(address, value) => Access::Write(address, u16::from(value), 1),
            }) as u8
        })
    }
}

/// The main loop waits on WRAM while the same DMA channels continue scanning.
pub(crate) fn wait_event(event: CpuSynchronousTimelineEvent, clock: &CpuMasterTimeline,
    ram: &[u8], native_ppu: &mut super::native_ppu::NativePpuState,
    dma: &mut DmaState, ppu: &mut snes::PpuState) -> u32 {
    let Some(event) = hdma_event(event) else { return 0; };
    native_ppu.synchronize(clock);
    dma.run_synchronous_hdma(event, |access| match access {
        HdmaBusAccess::Read(address) => {
            if address & 0x400000 == 0 && address & 65535 < 0x2000 {
                ram[(address & 65535) as usize]
            } else if (0x7e0000..0x800000).contains(&address) {
                ram[(address - 0x7e0000) as usize]
            } else {
                crate::ZeldaState::native_spotlight_table_byte(address)
                    .or_else(|| crate::ZeldaState::native_hdma_descriptor_byte(address))
                    .unwrap_or_else(|| panic!("unowned native wait HDMA read {address:06x}"))
            }
        }
        HdmaBusAccess::Write(address, value) => {
            assert!((0x2100..0x2140).contains(&address), "unowned native wait HDMA write {address:06x}");
            native_ppu.write_clocked(clock, address as u16, value);
            ppu.write(address as u8, value);
            0
        }
    })
}
