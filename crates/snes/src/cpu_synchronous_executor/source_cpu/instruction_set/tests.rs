use super::*;
use crate::cpu_timeline::{
    CpuBusEvent, CpuBusWorkload, CpuFieldTiming, CpuMasterTimeline, CpuRasterPosition,
    CpuSynchronousTimelineEvent,
};

// Executable contract from target/native-56390-rng-source: original
// $0d:ba71..ba7e, source comparison56389. This bus deliberately models only
// the recorded low-counter/RAM accesses. It is not a runtime shadow seed.
const PROGRAM: [u8; 14] = [
    0xad, 0x37, 0x21, 0xad, 0x3c, 0x21, 0x65, 0x1a, 0x6d, 0xa1, 0x0f, 0x8d, 0xa1, 0x0f,
];
const ENTRY: u16 = 0xba71;

struct CounterTraceBus {
    cpu: CpuState,
    open_bus: u8,
    timeline: CpuMasterTimeline,
    ram: [u8; 0x2000],
    latched: Option<u16>,
    counter_read: bool,
    reads: Vec<(u32, u16, u8)>,
    writes: Vec<(u32, u16, u8)>,
    events: Vec<(CpuSynchronousTimelineEvent, u16)>,
}

impl CounterTraceBus {
    fn new(h: u16) -> Self {
        let mut cpu = CpuState::new();
        cpu.k = 0x0d;
        cpu.pc = ENTRY;
        cpu.db = 6;
        cpu.a = 0x0401;
        cpu.c = true;
        cpu.mf = true;
        cpu.xf = true;
        cpu.e = false;
        let mut ram = [0; 0x2000];
        ram[0x1a] = 173;
        ram[0xfa1] = 90;
        let mut timeline = CpuMasterTimeline::at_raster(
            0,
            CpuRasterPosition::new(103, h),
            CpuBusWorkload::default(),
            CpuFieldTiming::NON_INTERLACE_EVEN,
        );
        timeline.begin_synchronous_timeline().unwrap();
        Self {
            cpu,
            open_bus: 1,
            timeline,
            ram,
            latched: None,
            counter_read: false,
            reads: Vec::new(),
            writes: Vec::new(),
            events: Vec::new(),
        }
    }

    fn h(&self) -> u16 {
        self.timeline.raster_position().coordinates().1
    }

    fn run_instruction(&mut self) {
        let pc = self.cpu.pc;
        let opcode = PROGRAM[usize::from(pc - ENTRY)];
        self.timeline.advance_synchronous_pcbase_opcode_fetch(8);
        self.cpu.pc += 1;
        self.execute(opcode, 0x0d_0000 | u32::from(pc), &mut Vec::new())
            .unwrap();
    }

    fn immediate(&mut self, width: u16, publish: bool) -> Result<u16, SourceCpuError> {
        let offset = usize::from(self.cpu.pc - ENTRY);
        let value = if width == 1 {
            u16::from(PROGRAM[offset])
        } else {
            u16::from_le_bytes([PROGRAM[offset], PROGRAM[offset + 1]])
        };
        self.add_cycles(u32::from(width) * 8)?;
        self.cpu.pc += width;
        if publish {
            self.open_bus = (value >> (8 * (width - 1))) as u8;
        }
        Ok(value)
    }
}

impl SourceCpuInstructionBus for CounterTraceBus {
    fn cpu(&self) -> &CpuState {
        &self.cpu
    }
    fn cpu_mut(&mut self) -> &mut CpuState {
        &mut self.cpu
    }
    fn set_open_bus(&mut self, value: u8) {
        self.open_bus = value;
    }
    fn immediate8(
        &mut self,
        publish: bool,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        Ok(self.immediate(1, publish)? as u8)
    }
    fn immediate16(
        &mut self,
        publish: bool,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        self.immediate(2, publish)
    }
    fn immediate16_slow(
        &mut self,
        _: bool,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        panic!("trace contains no slow operand fetch")
    }
    fn absolute_long_address(
        &mut self,
        _: bool,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u32, SourceCpuError> {
        panic!("trace contains no long operand")
    }
    fn read_byte(
        &mut self,
        address: u32,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u8, SourceCpuError> {
        assert!((address >> 16) & 0x7f < 0x40);
        let adr = address as u16;
        let (value, cycles) = match adr {
            0x2137 => {
                // The source witness has WRIO bit7 enabled and no long dot.
                assert!(self.h() < 1292);
                self.latched = Some(self.h() / 4);
                (1, 6) // captured PPU.OpenBus1, separate from CPU OpenBus
            }
            0x213c => {
                assert!(!self.counter_read, "trace has one low-byte counter read");
                self.counter_read = true;
                (self.latched.unwrap() as u8, 6)
            }
            0..=0x1fff => (self.ram[usize::from(adr)], 8),
            _ => return Err(SourceCpuError::UnsupportedBusMap { address }),
        };
        self.reads.push((address, self.h(), value));
        self.add_cycles(cycles)?;
        Ok(value)
    }
    fn write_byte(
        &mut self,
        address: u32,
        value: u8,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        assert!((address >> 16) & 0x7f < 0x40 && address as u16 <= 0x1fff);
        self.ram[usize::from(address as u16)] = value;
        self.writes.push((address, self.h(), value));
        self.add_cycles(8)
    }
    fn read_word(
        &mut self,
        _: u32,
        _: WordWrap,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<u16, SourceCpuError> {
        panic!("trace contains no data word read")
    }
    fn write_word(
        &mut self,
        _: u32,
        _: u16,
        _: WordWrap,
        _: WordWriteOrder,
        _: &mut Vec<SourceCpuBusAccess>,
    ) -> Result<(), SourceCpuError> {
        panic!("trace contains no data word write")
    }
    fn add_cycles(&mut self, cycles: u32) -> Result<(), SourceCpuError> {
        self.timeline
            .advance_synchronous_after_semantics_with(cycles, |event, at| {
                self.events
                    .push((event, (at.master_cycles() % 1364) as u16));
                Ok::<_, std::convert::Infallible>(0)
            })
            .unwrap();
        Ok(())
    }
}

#[test]
fn source_rng_witness_samples_counter_after_operand_transaction() {
    let mut bus = CounterTraceBus::new(1168);
    for _ in 0..5 {
        bus.run_instruction();
    }
    assert_eq!(
        bus.reads,
        vec![
            (0x06_2137, 1192, 1),
            (0x06_213c, 1222, 0x2a),
            (0x00_001a, 1244, 173),
            (0x06_0fa1, 1276, 90)
        ]
    );
    assert_eq!(bus.writes, vec![(0x06_0fa1, 1308, 0x32)]);
    assert_eq!(bus.ram[0xfa1], 0x32);
    assert_eq!(bus.cpu.a, 0x0432);
    assert!(bus.cpu.c);
    assert_eq!(bus.h(), 1316);
    assert!(bus.events.is_empty());
}

#[test]
fn word_operand_drains_due_refresh_before_hardware_read() {
    let mut bus = CounterTraceBus::new(522);
    bus.run_instruction();
    // PCBase fetch adds8 without drain; the word operand's one AddCycles(16)
    // observes refresh at546, then pays40 before sampling the mapped register.
    assert_eq!(
        bus.events,
        vec![(
            CpuSynchronousTimelineEvent::Bus(CpuBusEvent::WramRefresh),
            546
        )]
    );
    assert_eq!(bus.reads, vec![(0x06_2137, 586, 1)]);
    assert_eq!(bus.h(), 592);
}
