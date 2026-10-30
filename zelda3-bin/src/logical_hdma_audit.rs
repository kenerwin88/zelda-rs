//! Counterfactual CPU pictures, never installed in the running engine or matcher.
//! ZELDA3_LOGICAL_HDMA_AUDIT names an evidence directory containing witnesses.jsonl
//! (selected --picture-events stores grouped by host). Results go to audit.jsonl.
use super::*;
use std::collections::BTreeMap;
use std::io::Write;

#[derive(serde::Deserialize)]
struct Store {
    address: usize,
    value: u16,
    previous: u16,
    width: usize,
    at: u64,
    pc: u32,
}

#[derive(serde::Deserialize)]
struct Witness {
    iteration: u64,
    host: usize,
    field: u64,
    groups: Vec<String>,
    stores: Vec<Store>,
    dma: snes::DmaState,
}

pub(super) struct Audit {
    witnesses: BTreeMap<usize, Witness>,
    output: BufWriter<fs::File>,
}

/// SourceCpu::write_word's direct WRAM transaction commits both bytes before
/// draining its 16 clocks (GetSetMemoryAccessX2AfterSemanticDraining).
/// A witness with width=2 therefore gives both bytes the SAME semantic time.
/// The preceding transaction drains events with Cycles >= NextEvent, so a
/// store starting at the nominal HDMA deadline cannot precede that event.
fn visible_byte(store_at: u64, read_at: u64) -> bool {
    store_at < read_at
}

fn apply_store(table: &mut [u8], store: &Store, field_start: u64, timed: bool) {
    for byte in 0..store.width {
        let Some(offset) = (store.address + byte).checked_sub(0x1b00)
            .filter(|&offset| offset < table.len()) else { continue; };
        let read_at = field_start + (offset / 2) as u64 * 1364 + 1106;
        if !timed || visible_byte(store.at, read_at) {
            table[offset] = (store.value >> (8 * byte)) as u8;
        }
    }
}

impl Audit {
    pub(super) fn from_env() -> Option<Self> {
        let directory = PathBuf::from(zelda3::debug_env::var_os!("ZELDA3_LOGICAL_HDMA_AUDIT")?);
        let input = BufReader::new(fs::File::open(directory.join("witnesses.jsonl")).expect("HDMA witnesses"));
        let witnesses = input.lines().map(|line| {
            let witness: Witness = serde_json::from_str(&line.expect("HDMA witness line")).expect("HDMA witness");
            (witness.host, witness)
        }).collect();
        Some(Self { witnesses, output: BufWriter::new(fs::File::create(directory.join("audit.jsonl")).expect("HDMA audit output")) })
    }

    pub(super) fn observe(&mut self, game: &ZeldaState, capture: &LiveGpuFrameCapture,
        iteration: u64, host: usize, expected: &VideoDigest) {
        let Some(witness) = self.witnesses.remove(&host) else { return; };
        assert_eq!(witness.iteration, iteration);
        let base = VideoCheck::render_cpu(capture).0;
        for group in &witness.groups {
            let stores: Vec<_> = witness.stores.iter().filter(|store|
                matches!(store.pc, 0xf3bb | 0xf42f | 0xf432 | 0xf435 | 0xf438
                    | 0xf43b | 0xf43e | 0xf441) == (group == "spotlight")).collect();
            // The ROM $00:F2FB descriptor is F8 00 1B F8 F0 1B 00:
            // two 120-line repeat runs, two bytes per line. $0A:BDDD uses
            // F0 00 1B F0 E0 1B 00, two contiguous 112-line runs. Output row
            // y (physical V=y+1) consumes $1B00+2*y at V=y,H=1106.
            // Unsupported/repointed setups are explicit, never guessed.
            let selected: Vec<_> = (6..=7).filter(|&c| {
                let d = witness.dma.channel[c];
                d.hdma_active && ((d.a_bank == 0 && d.a_adr == 0xf2fb)
                    || (d.a_bank == 0x0a && d.a_adr == 0xbddd))
            }).collect();
            let supported = !selected.is_empty() && selected.iter().all(|&c| {
                let d = witness.dma.channel[c];
                d.indirect && d.ind_bank == 0
                    && !d.from_b && ((d.mode == 1 && d.b_adr == 0x26)
                        || (d.mode == 2 && matches!(d.b_adr, 0x0d | 0x0f)))
            });
            if !supported || stores.is_empty() {
                writeln!(self.output, "{}", serde_json::json!({"iteration":iteration,"host":host,
                    "group":group,"status":"unsupported_setup_or_no_stores"})).expect("audit row");
                continue;
            }
            let ram = game.wram_image();
            let native = &ram[0x1dba0..0x1dba0 + 448];
            let mut old = native.to_vec();
            // Undo in reverse order to recover the source's entry generation.
            // Values are confined to this diagnostic; also measure whether
            // that entry generation equals the native capture's table.
            for store in stores.iter().rev() {
                for byte in 0..store.width {
                    if let Some(offset) = (store.address + byte).checked_sub(0x1b00).filter(|&x| x < 448) {
                        old[offset] = (store.previous >> (8 * byte)) as u8;
                    }
                }
            }
            let mut timed = old.clone();
            let mut new = old.clone();
            let start = snes::CpuFieldTiming::non_interlace(false).field_start_master_cycles(witness.field);
            let mut rows = Vec::new();
            for store in &stores {
                apply_store(&mut new, store, start, false);
                apply_store(&mut timed, store, start, true);
            }
            for row in 0..224 {
                rows.push(serde_json::json!({"line":row+1,"address":0x1b00+2*row,
                    "read_at":start+row as u64*1364+1106,
                    "old":&old[row*2..row*2+2],"timed":&timed[row*2..row*2+2],
                    "new":&new[row*2..row*2+2]}));
            }
            let render = |table: &[u8]| {
                let mut ppu = capture.presented_ppu().clone();
                let mut lines = *capture.presented_scanlines();
                let mut fields = 0;
                for row in 0..224 {
                    for &c in &selected {
                        let d = witness.dma.channel[c];
                        ppu.write(d.b_adr, table[2*row]);
                        ppu.write(d.b_adr + u8::from(d.mode == 1), table[2*row+1]);
                        if d.mode == 1 {
                            lines[row].0 = ppu.window1_left;
                            lines[row].1 = ppu.window1_right;
                            fields |= 3;
                        } else {
                            let bg = usize::from((d.b_adr - 0x0d)/2);
                            lines[row].5[bg] = ppu.bg_layer[bg].h_scroll;
                            fields |= 1 << (5+bg);
                        }
                    }
                }
                VideoCheck::render_cpu(&capture.with_line_register_fields(&lines, fields)).0
            };
            let old_hash = render(&old);
            let timed_hash = render(&timed);
            let new_hash = render(&new);
            let confirmed = old_hash == base && timed_hash == *expected && base != *expected;
            writeln!(self.output, "{}", serde_json::json!({"iteration":iteration,"host":host,
                "group":group,"status":if confirmed {"confirmed"} else {"not_confirmed"},
                "native_old_bytes_different":native.iter().zip(&old).filter(|(a,b)|a!=b).count(),
                "old_equals_native":old_hash==base,"timed_equals_cache":timed_hash==*expected,
                "new_equals_cache":new_hash==*expected,"baseline":base.sha256,"old":old_hash.sha256,
                "timed":timed_hash.sha256,"new":new_hash.sha256,"expected":expected.sha256,
                "rows":rows})).expect("audit row");
        }
        self.output.flush().expect("flush HDMA audit");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_store_before_at_after_hdma() {
        assert!(visible_byte(1105, 1106));
        assert!(!visible_byte(1106, 1106));
        assert!(!visible_byte(1107, 1106));
    }

    #[test]
    fn confirmed_spotlight_and_mirror_witness_boundaries() {
        // i4280/host4804: $1CBA, physical line 222, F808 -> E818.
        // i191998/host210037: $1B20, physical line 17, 088A -> 0889.
        for (host, row, old, new, pc) in [(4804, 221usize, 0xf808u16, 0xe818, 0xf3bb),
            (210037, 16, 0x088a, 0x0889, 0xfe82)] {
            let start = snes::CpuFieldTiming::non_interlace(false).field_start_master_cycles(host);
            let read_at = start + row as u64 * 1364 + 1106;
            for (at, expected) in [(read_at - 1, new), (read_at, old), (read_at + 1, old)] {
                let mut table = vec![0u8; 448];
                table[row*2..row*2+2].copy_from_slice(&old.to_le_bytes());
                apply_store(&mut table, &Store { address: 0x1b00 + row*2, value: new,
                    previous: old, width: 2, at, pc }, start, true);
                assert_eq!(&table[row*2..row*2+2], &expected.to_le_bytes());
            }
        }
    }
}
