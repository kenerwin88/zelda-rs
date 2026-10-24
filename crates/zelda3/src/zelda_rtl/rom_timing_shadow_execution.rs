//! Exact cartridge timing entry points; excluded from the lagless build.

use super::*;

impl ZeldaState {
    pub(in crate::zelda_rtl) fn advance_native_exact_cpu_host(&mut self) {
        if !self.rom_startup_timing
            || self.original_timing_semantic_receipts.is_some()
            || crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_OWNER").is_none()
        {
            self.native_exact_cpu_host_trace = None;
            return;
        }
        let host = self
            .frame_ctr_dbg
            .checked_sub(1)
            .expect("native exact CPU host starts after frame entry");
        let diagnose_nmi = crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_NMI")
            .is_some()
            && crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_TRACE_HOST")
                .ok()
                .and_then(|value| {
                    let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                    Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
                })
                .is_some_and(|(first, last)| (first..=last).contains(&host));
        if diagnose_nmi {
            eprintln!("native-exact-nmi-entry host={host} work={:?} module={:02x}/{:02x}/{:02x} latch={} request={} gate={}",
                self.game_execution_scheduler.current_work(),
                self.game_state.frame.main_module, self.game_state.frame.submodule,
                self.game_state.frame.subsubmodule,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.core_update_disable_flag);
        }
        let trace_host = crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_TRACE_HOST")
            .ok()
            .and_then(|value| {
                let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
            })
            .is_some_and(|(first, last)| (first..=last).contains(&host));
        let independent_trial_host =
            crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_HOST")
                .ok()
                .and_then(|value| {
                    let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                    Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
                })
                .is_some_and(|(first, last)| (first..=last).contains(&host));
        let contiguous_trial_start =
            crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_CONTIGUOUS_TRIAL_START")
                .ok()
                .and_then(|value| value.parse::<u32>().ok());
        let diagnose_ownership =
            crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_OWNERSHIP").is_some();
        let diagnostic_scroll_phase = diagnose_ownership.then(|| self.dialogue_scroll_phase());
        // Trial/rebase reads are a cartridge image; source CPU writes remain
        // private to its machine and must not be imported into native play.
        let native_wram_image = self.wram_image().into_owned();
        let owner = if let Some(owner) = self.native_exact_cpu_owner.as_mut() {
            owner
        } else {
            self.native_exact_cpu_owner = Some(
                NativeExactCpuOwner::new(&self.rom, &self.sram)
                    .expect("native exact CPU requires a cold Zelda ROM/SRAM seed"),
            );
            self.native_exact_cpu_owner.as_mut().unwrap()
        };
        if trace_host {
            if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_PPU").is_some() {
                eprintln!("native-exact-ppu-owner host={host} native_latch={} source_latch={} native_gate={} source_gate={} scheduler={:?}", self.ram[crate::game_state::constants::NMI_BOOLEAN], owner.cpu.machine().snes().ram[0x12], self.ram[crate::game_state::constants::NMI_DISABLE_CORE_UPDATES], owner.cpu.machine().snes().ram[0x710], self.game_execution_scheduler);
                let source = &owner.cpu.machine().snes().ppu;
                let compare = |label: &str, source: &[u16], native: &[u16]| {
                    let differences: Vec<_> = source.iter().zip(native).enumerate()
                        .filter_map(|(word, (source, native))| (source != native).then_some((word, *source, *native)))
                        .collect();
                    eprintln!("native-exact-ppu-entry host={host} domain={label} differences={} first={:04x?}",
                        differences.len(), differences.iter().take(12).collect::<Vec<_>>());
                };
                compare("live-vram", &source.vram, &self.ppu.vram);
                compare("live-cgram", &source.cgram, &self.ppu.cgram);
                compare("live-oam", &source.oam, &self.ppu.oam);
                for (start, len) in [(0x6000, 0x1800), (0x10000, 0x2000)] {
                    let source = &owner.cpu.machine().snes().ram[start..start + len];
                    let native = &self.ram[start..start + len];
                    let differences: Vec<_> = source.iter().zip(native).enumerate()
                        .filter_map(|(offset, (source, native))| (source != native).then_some((start + offset, *source, *native)))
                        .collect();
                    eprintln!("native-exact-ppu-staging host={host} start={start:05x} differences={} first={:05x?} source_head={:02x?} native_head={:02x?}", differences.len(), differences.iter().take(12).collect::<Vec<_>>(), &source[..16], &native[..16]);
                }
                if let Some(snapshot) = &self.display_snapshot {
                    compare("captured-vram", &source.vram, &snapshot.ppu.vram);
                    compare("captured-cgram", &source.cgram, &snapshot.ppu.cgram);
                    compare("captured-oam", &source.oam, &snapshot.ppu.oam);
                    eprintln!("native-exact-ppu-generation host={host} epoch={} vram={:?} cgram={:?} oam={:?} bg_scroll={:?}",
                        snapshot.publication_epoch, snapshot.vram_generation, snapshot.cgram_scanout_generation,
                        snapshot.oam_scanout_source, snapshot.presented_bg_scroll_override);
                }
            }
            let start = crate::game_state::constants::MESSAGING_RENDER_BUFFER;
            let source = &owner.cpu.machine().snes().ram;
            if let Some((offset, (&source_byte, &native_byte))) = source[start..start + 0x7e0]
                .iter()
                .zip(&self.ram[start..start + 0x7e0])
                .enumerate()
                .find(|(_, (source_byte, native_byte))| source_byte != native_byte)
            {
                eprintln!("native-exact-cpu-dialogue-buffer-entry host={host} first_diff={offset:04x} source={source_byte:02x} native={native_byte:02x}");
            }
        }
        if crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_NMI_PHASE_HOST")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(host)
        {
            eprintln!("native-exact-cpu-entry host={host} source_nmis_accepted={} source_nmis_completed={} translated_nmi_handlers_completed={} carried_nmis={:?} scheduler={:?}",
                owner.source_nmis_accepted, owner.source_nmis_completed,
                owner.translated_nmi_handlers_completed,
                owner.interrupt_stack.iter().filter_map(|interrupt| match interrupt {
                    NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                    NativeExactCpuInterrupt::Irq => None,
                }).collect::<Vec<_>>(), self.game_execution_scheduler);
        }
        let contiguous_trial = contiguous_trial_start.is_some_and(|start| host >= start)
            && !owner.translated_trial_stopped;
        let trial_host = independent_trial_host || contiguous_trial;
        let preserve_stack_segment = trial_host
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_STACK").is_some();
        let preserve_direct_page = preserve_stack_segment
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_DP").is_some();
        let preserve_dialogue_asset_state = preserve_direct_page
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_DIALOGUE").is_some();
        let preserve_overworld_decode_scratch = preserve_stack_segment
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_OVERWORLD_SCRATCH")
                .is_some();
        // A trial never changes the retained owner or translated gameplay.
        // It tests whether the exact CPU can read translated memory at this
        // instruction boundary while retaining its physical PPU/APU/DMA bus.
        let mut trial_reads = Vec::new();
        let mut trial_writes = Vec::new();
        let mut trial_instructions = Vec::new();
        let trial = if trial_host {
            let trial_ram = Self::native_exact_cpu_trial_ram(
                &native_wram_image,
                self.game_execution_scheduler.current_work(),
                self.native_exact_cpu_landing_spotlight_build.is_some(),
            );
            let mut candidate = if contiguous_trial {
                owner
                    .translated_trial
                    .take()
                    .map(|trial| *trial)
                    .unwrap_or_else(|| {
                        let mut trial = owner.clone();
                        trial.translated_trial = None;
                        trial
                    })
            } else {
                owner.clone()
            };
            let pre_rebase_wram =
                diagnose_ownership.then(|| candidate.cpu.machine().snes().ram.clone());
            let carried_nmi = candidate
                .interrupt_stack
                .iter()
                .rev()
                .find_map(|interrupt| match interrupt {
                    NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                    NativeExactCpuInterrupt::Irq => None,
                });
            let phase_aligned_rebase =
                crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_REBASE_AFTER_CARRIED_NMI")
                    .is_some()
                    && self
                        .game_execution_scheduler
                        .resumed_call_stack_is_before_nmi()
                    && carried_nmi.is_some();
            let mut entry_sp = candidate.cpu.machine().snes().cpu.sp;
            let mut rebased_after_handler = false;
            let result: Result<NativeExactCpuHostTrace, Box<dyn std::error::Error>> =
                if phase_aligned_rebase {
                    let carried = carried_nmi.unwrap();
                    candidate.advance_host_with_hook(
                        host,
                        self.previous_host_controller_input,
                        Some(&mut trial_reads),
                        Some(&mut trial_writes),
                        Some(&mut trial_instructions),
                        |cpu, completed| {
                            if !rebased_after_handler
                                && completed
                                    .is_some_and(|nmi| nmi.accepted_at == carried.accepted_at)
                            {
                                entry_sp = rebase_translated_timing_state(
                                    cpu,
                                    &trial_ram,
                                    &self.sram,
                                    preserve_stack_segment,
                                    preserve_direct_page,
                                    preserve_dialogue_asset_state,
                                    preserve_overworld_decode_scratch,
                                )?;
                                rebased_after_handler = true;
                            }
                            Ok(())
                        },
                    )
                } else {
                    match rebase_translated_timing_state(
                        &mut candidate.cpu,
                        &trial_ram,
                        &self.sram,
                        preserve_stack_segment,
                        preserve_direct_page,
                        preserve_dialogue_asset_state,
                        preserve_overworld_decode_scratch,
                    ) {
                        Ok(sp) => {
                            entry_sp = sp;
                            candidate
                                .advance_host(
                                    host,
                                    self.previous_host_controller_input,
                                    Some(&mut trial_reads),
                                    Some(&mut trial_writes),
                                    Some(&mut trial_instructions),
                                )
                                .map_err(Into::into)
                        }
                        Err(error) => Err(error.into()),
                    }
                };
            let cpu = &candidate.cpu.machine().snes().cpu;
            Some((
                result,
                (u32::from(cpu.k) << 16) | u32::from(cpu.pc),
                candidate.cpu.machine().timestamp().master_cycles(),
                entry_sp,
                phase_aligned_rebase,
                rebased_after_handler,
                candidate,
                pre_rebase_wram,
            ))
        } else {
            None
        };
        let mut baseline_reads = Vec::new();
        let mut baseline_writes = Vec::new();
        let mut baseline_instructions = Vec::new();
        let trace = owner
            .advance_host(
                host,
                self.previous_host_controller_input,
                trial_host.then_some(&mut baseline_reads),
                trial_host.then_some(&mut baseline_writes),
                trial_host.then_some(&mut baseline_instructions),
            )
            .expect("retained native exact CPU failed before host return");
        if let Some((
            trial_result,
            candidate_pc,
            candidate_clock,
            entry_sp,
            phase_aligned_rebase,
            rebased_after_handler,
            mut candidate,
            pre_rebase_wram,
        )) = trial
        {
            let first_read_difference = baseline_reads
                .iter()
                .zip(&trial_reads)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            let first_write_difference = baseline_writes
                .iter()
                .zip(&trial_writes)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            let trial_status = trial_result
                .as_ref()
                .map(|result| format!("nmis={:?}", result.nmis))
                .unwrap_or_else(|error| format!("stopped={error}"));
            let cpu = &owner.cpu.machine().snes().cpu;
            let baseline_pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
            let baseline_clock = owner.cpu.machine().timestamp().master_cycles();
            let same_schedule = trial_result.as_ref().is_ok_and(|candidate| {
                candidate.nmis == trace.nmis
                    && baseline_instructions == trial_instructions
                    && candidate_pc == baseline_pc
                    && candidate_clock == baseline_clock
            });
            let same_path =
                same_schedule && baseline_reads == trial_reads && baseline_writes == trial_writes;
            if trace_host {
                let scroll_stores = baseline_writes.iter().filter(|write| {
                    (0x0e_cfe2..=0x0e_d0c2).contains(&write.pc)
                        && (0x7f_0000..0x7f_07e0).contains(&write.address)
                }).collect::<Vec<_>>();
                let scroll_passes = baseline_writes.iter().filter(|write| {
                    (0x0e_cfe2..=0x0e_d0c2).contains(&write.pc)
                        && native_exact_cpu_wram_key(write.address) == Some(0x1cdf)
                }).collect::<Vec<_>>();
                if !scroll_stores.is_empty() || !scroll_passes.is_empty() {
                    eprintln!("native-exact-cpu-scroll-work host={host} stores={} first={:?} last={:?} passes={:?} return_pc={baseline_pc:06x} x={:04x} y={:04x}",
                        scroll_stores.len(), scroll_stores.first(), scroll_stores.last(),
                        scroll_passes, cpu.x, cpu.y);
                }
            }
            if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_TILESET_PHASES").is_some() {
                let phase = |pc: u32| match pc {
                    0x00_e19b => Some("initialize-tilesets"),
                    0x00_e1ae => Some("select-sprite-sheets"),
                    0x00_e6b7 => Some("common-sprite-expand"),
                    0x00_e583 => Some("sprite-sheet-load"),
                    0x00_e772 => Some("sprite-decompress"),
                    0x00_e78f => Some("background-decompress"),
                    0x00_e609 => Some("background-sheet-load"),
                    _ => None,
                };
                let mut work_master_cycles = 0u64;
                for instruction in &baseline_instructions {
                    if let Some(current) = phase(instruction.pc) {
                        eprintln!(
                            "native-tileset-phase host={host} phase={} pc={:06x} at={} work_before={work_master_cycles}",
                            current, instruction.pc, instruction.started_at,
                        );
                    }
                    if (0x00_e100..=0x00_e900).contains(&instruction.pc) {
                        work_master_cycles += instruction.ended_at - instruction.started_at;
                    }
                }
                if work_master_cycles != 0 {
                    eprintln!("native-tileset-work host={host} master_cycles={work_master_cycles}");
                }
                let work_at = |at: u64| baseline_instructions.iter()
                    .filter(|instruction| (0x00_e100..=0x00_e900).contains(&instruction.pc))
                    .map(|instruction| at.min(instruction.ended_at)
                        .saturating_sub(instruction.started_at))
                    .sum::<u64>();
                for slot in 0..4 {
                    let selection = baseline_writes.iter().find(|write| {
                        native_exact_cpu_wram_key(write.address) == Some(0xc2fc + slot)
                    });
                    if let Some(write) = selection {
                        eprintln!(
                            "native-tileset-selection host={host} slot={slot} sheet={} pc={:06x} at={} work_before={}",
                            write.value, write.pc, write.at, work_at(write.at),
                        );
                    }
                    let base = 0x7800 + slot * 0x600;
                    let mut stores = baseline_writes.iter().filter(|write| {
                        native_exact_cpu_wram_key(write.address)
                            .is_some_and(|address| (base..base + 0x600).contains(&address))
                    });
                    if let Some(first) = stores.next() {
                        let last = stores.last().unwrap_or(first);
                        eprintln!(
                            "native-tileset-stores host={host} slot={slot} first_pc={:06x} first_address={:06x} first_at={} first_work={} last_address={:06x} last_at={} last_work={}",
                            first.pc, first.address, first.at, work_at(first.at),
                            last.address, last.at, work_at(last.at),
                        );
                    }
                }
                let mut region_writes = std::collections::BTreeMap::<u32, (usize, u64)>::new();
                for write in &baseline_writes {
                    let Some(address) = native_exact_cpu_wram_key(write.address) else { continue };
                    let region = match address {
                        0x6000..0x9000 => 0x6000 + (address - 0x6000) / 0x600 * 0x600,
                        0xc2fc..0xc300 => 0xc2fc,
                        0x14000..0x14600 => 0x14000,
                        _ => continue,
                    };
                    let entry = region_writes.entry(region).or_insert((0, 0xcbf29ce484222325));
                    entry.0 += 1;
                    entry.1 = (entry.1 ^ (address - region) as u64).wrapping_mul(0x100000001b3);
                }
                for (region, (stores, address_hash)) in region_writes {
                    eprintln!("native-tileset-source-writes host={host} region=${region:05x} stores={stores} address_hash={address_hash:016x}");
                }
            }
            if diagnose_ownership {
                if let Some((source_read, trial_read)) = first_read_difference {
                    let (source_read, trial_read) =
                        native_exact_cpu_differing_read_bytes(source_read, trial_read);
                    let source_writer = native_exact_cpu_last_writer(
                        source_read,
                        &baseline_writes,
                        &owner.diagnostic_writers,
                        host,
                    );
                    let trial_writer = native_exact_cpu_last_writer(
                        trial_read,
                        &trial_writes,
                        &candidate.diagnostic_writers,
                        host,
                    );
                    let rebase = native_exact_cpu_wram_key(source_read.address).and_then(|key| {
                        let index = usize::try_from(key).ok()?;
                        let before = pre_rebase_wram.as_ref()?.get(index).copied()?;
                        let native = self.ram.get(index).copied()?;
                        Some((before, native))
                    });
                    let native_state_owner = match native_exact_cpu_wram_key(source_read.address) {
                        Some(offset @ 0x0500..=0x053f)
                            if (0x02_fd00..=0x02_fe40).contains(&source_read.pc) => {
                            // $02:FD9C/$02:FDAF fill this 32-word stripe before
                            // Map16ToMap8 reads it at $02:FE10. Its native owner
                            // is separate from the map32 definition cache.
                            let index = (offset as usize - 0x0500) / 2;
                            let native_word = self.game_state.world.transient
                                .overworld_map16_stripe_word(index);
                            format!("OverworldMap16Stripe(index={index}, native_word={native_word:04x}, overlay_progress={:?}, screen_progress={:?})",
                                self.native_pre_overworld_overlay_decode,
                                self.native_pre_overworld_screen_build)
                        }
                        Some(0x10000..=0x10fff)
                            if (0x0e_cfe2..=0x0e_d0c2).contains(&source_read.pc) => format!(
                            "MessagingScrollCopy(phase={:?}, remaining_cpu_work={:?}, copy_cursor={:?}, line_counter={})",
                            diagnostic_scroll_phase,
                            self.dialogue_scroll_remaining_master_cycles,
                            self.native_scroll_copy_cursor,
                            self.game_state.messaging.dialogue_source_offset.bank_offset_low_nibble(),
                        ),
                        Some(0x10000..=0x10fff) => format!(
                            "MessagingRenderBuffer(read_pos={:04x}, glyph_phase={:?}, dispatch={:?})",
                            self.game_state.messaging.runtime.dialogue_msg_read_pos(),
                            self.dialogue_vwf_glyph_cpu_phase,
                            self.dialogue_vwf_dispatch_cursor,
                        ),
                        Some(0x14000..=0x14fff) => "GraphicsDecompressionScratch".to_owned(),
                        Some(offset @ 0x7800..=0x8fff)
                            if (0x00_e790..=0x00_e842).contains(&source_read.pc) => {
                            let slot = ((offset - 0x7800) / 0x600) as usize;
                            let native_sheet = self.game_state.sprites.workspace.graphics_subset(slot);
                            let source_sheet = owner.cpu.machine().snes().ram
                                .get(0xc2fc + slot).copied();
                            let subset_read = NativeExactCpuRead {
                                pc: source_read.pc,
                                address: 0x7e_c2fc + slot as u32,
                                value: u16::from(source_sheet.unwrap_or(0)),
                                width: 1,
                                at: source_read.at,
                            };
                            let subset_source_writer = native_exact_cpu_last_writer(
                                subset_read, &baseline_writes, &owner.diagnostic_writers, host,
                            );
                            let buffer_base = 0x7800 + slot * 0x600;
                            let output_offset = offset as usize - buffer_base;
                            let entry = crate::cycle_models::decompress::DecompressEntry::Sprite;
                            let sprite_asset = self.assets.as_ref()
                                .and_then(|assets| assets.asset(64));
                            let describe_sheet = |sheet: u8| {
                                let stream = sprite_asset.map(|asset| {
                                    find_index_in_memblk(MemBlk { ptr: asset }, usize::from(sheet)).ptr
                                })?;
                                let (_, stores) = crate::cycle_models::decompress::decompress_store_timeline(
                                    entry,
                                    sheet,
                                    stream,
                                    crate::cycle_models::decompress::stream_source_offset(entry, sheet),
                                    0x7e_0000 + buffer_base as u32,
                                );
                                let store_time = stores.iter()
                                    .find(|store| usize::from(store.output_offset) == output_offset)
                                    .map(|store| store.after_master_cycles);
                                let data = if sheet < 103 && stream.len() == 0x600 {
                                    stream.to_vec()
                                } else {
                                    decompress_asset(stream)
                                };
                                Some((data.get(output_offset).copied(), store_time))
                            };
                            let source_output = source_sheet.and_then(describe_sheet);
                            let native_output = describe_sheet(native_sheet);
                            format!(
                                "TilesetDecompressionOutput(slot={slot}, source_sheet={source_sheet:?}, native_sheet={native_sheet}, subset_source_writer={subset_source_writer:?}, buffer={buffer_base:04x}, offset={output_offset:04x}, source_output={source_output:?}, native_output={native_output:?})"
                            )
                        }
                        Some(0x1cd9..=0x1cda) => "DialogueDecodeCursor".to_owned(),
                        _ => format!("{:?}", self.game_execution_scheduler.current_work()),
                    };
                    eprintln!("native-exact-cpu-ownership host={host} first_read=({source_read:?}, {trial_read:?}) source_last_writer={source_writer:?} trial_last_writer={trial_writer:?} rebase_before_native={rebase:?} native_state_owner={native_state_owner} native_continuation={:?} cached_checkpoint={:?} dungeon_reset_checkpoint={:?}",
                        self.game_execution_scheduler.current_work(),
                        trace.cached_sprite_progress, trace.dungeon_reset_progress);
                }
                native_exact_cpu_record_writers(
                    &mut owner.diagnostic_writers,
                    &baseline_writes,
                    host,
                );
                native_exact_cpu_record_writers(
                    &mut candidate.diagnostic_writers,
                    &trial_writes,
                    host,
                );
            }
            let first_instruction_difference = baseline_instructions
                .iter()
                .zip(&trial_instructions)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            eprintln!("native-exact-cpu-rebase-trial host={host} contiguous={contiguous_trial} entry_sp={entry_sp:04x} phase_aligned_rebase={phase_aligned_rebase} rebased_after_handler={rebased_after_handler} stack_segment_preserved={preserve_stack_segment} direct_page_preserved={preserve_direct_page} dialogue_asset_state_preserved={preserve_dialogue_asset_state} overworld_decode_scratch_preserved={preserve_overworld_decode_scratch} same_path={same_path} same_schedule={same_schedule} baseline_instructions={} trial_instructions={} first_instruction_difference={first_instruction_difference:?} baseline_reads={} trial_reads={} first_read_difference={first_read_difference:?} baseline_writes={} trial_writes={} first_write_difference={first_write_difference:?} baseline_nmis={:?} trial_status={trial_status} baseline_pc={baseline_pc:06x} trial_pc={candidate_pc:06x} baseline_clock={baseline_clock} trial_clock={candidate_clock}",
                baseline_instructions.len(), trial_instructions.len(),
                baseline_reads.len(), trial_reads.len(), baseline_writes.len(), trial_writes.len(), trace.nmis);
            if trace.wram_watch_address.is_some() && (trace_host || first_read_difference.is_some())
            {
                eprintln!("native-exact-cpu-wram-watch host={host} address={:?} source={:?} translated={:?}",
                    trace.wram_watch_address, trace.wram_watch_accesses,
                    trial_result.as_ref().map(|candidate| &candidate.wram_watch_accesses));
            }
            if contiguous_trial {
                // A data read can differ without changing the executed CPU
                // instruction timeline. Retain that distinction rather than
                // treating rendered content as a timing failure.
                if same_schedule {
                    owner.translated_trial = Some(Box::new(candidate));
                } else {
                    owner.translated_trial_stopped = true;
                }
            }
        }
        if trace_host {
            eprintln!("native-exact-cpu host={} return_pc={:06x} room_load_returned={} auxiliary_sprite_graphics_returned={} nmis={:?} nmi_completions={:?} pending_nmis_at_return={:?} nmi_update_decisions={:?} main_wait_returns={:?} main_loop_entries={:?} sprite_preparation_interruptions={:?} dungeon_header_index_writes={:?} nmi_gate_accesses={:?} ppu_reads={:?} overworld_scan_events={:?}", trace.host, trace.return_pc, trace.room_load_returned, trace.auxiliary_sprite_graphics_returned,
                trace.nmis, trace.nmi_completions, trace.pending_nmis_at_return,
                trace.nmi_update_decisions, trace.main_wait_returns, trace.main_loop_entries, trace.sprite_preparation_interruptions, trace.dungeon_header_index_writes, trace.nmi_gate_accesses, trace.ppu_reads, trace.overworld_scan_events);
            if trace.wram_watch_address.is_some() && !trial_host {
                eprintln!(
                    "native-exact-cpu-wram-watch host={host} address={:?} source={:?}",
                    trace.wram_watch_address, trace.wram_watch_accesses
                );
            }
            if trace.track_floor_tile_writes && !trace.floor_tile_writes.is_empty() {
                eprintln!(
                    "native-exact-cpu-floor-writes host={host} count={} first={:?} last={:?}",
                    trace.floor_tile_writes.len(),
                    trace.floor_tile_writes.first(),
                    trace.floor_tile_writes.last()
                );
            }
            if !trace.object_completions.is_empty() {
                eprintln!(
                    "native-exact-cpu-object-completions host={host} count={} first={:?} last={:?}",
                    trace.object_completions.len(),
                    trace.object_completions.first(),
                    trace.object_completions.last()
                );
            }
            if !trace.room_upload_events.is_empty() {
                eprintln!(
                    "native-exact-cpu-room-upload-events host={host} events={:?}",
                    trace.room_upload_events
                );
            }
            if !trace.item_receipt_step_writes.is_empty() {
                eprintln!(
                    "native-exact-cpu-item-receipt-steps host={host} writes={:?}",
                    trace.item_receipt_step_writes
                );
            }
            if !trace.spotlight_events.is_empty() {
                let starts = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event,
                            NativeExactCpuSpotlightEvent::WindowYInitialized { .. }
                        )
                    })
                    .count();
                let reads = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(event, NativeExactCpuSpotlightEvent::CircleInputRead { .. })
                    })
                    .count();
                let rows = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(event, NativeExactCpuSpotlightEvent::RowAdvanced { .. })
                    })
                    .count();
                eprintln!("native-exact-cpu-spotlight-events host={host} starts={starts} reads={reads} rows={rows} last={:?}",
                    trace.spotlight_events.last());
            }
            if !trace.spotlight_reset_stores.is_empty() {
                eprintln!(
                    "native-exact-cpu-spotlight-reset host={host} stores={} first={:?} last={:?}",
                    trace.spotlight_reset_stores.len(),
                    trace.spotlight_reset_stores.first(),
                    trace.spotlight_reset_stores.last()
                );
            }
            if !trace.music_port_accesses.is_empty() {
                eprintln!(
                    "native-exact-cpu-music-port host={host} accesses={:?}",
                    trace.music_port_accesses
                );
            }
        }
        self.reconcile_native_exact_cpu_landing_music_nmi(&trace);
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_VWF_WORK").is_some()
            && trace_host
        {
            eprintln!("native-exact-vwf-work host={host} cpu={} since_nmi={} first={:?} last={:?} cursor_work={:?} draw_entry_work={:?} nmi_blocks={:?} nmis={:?} completions={:?}",
                trace.vwf_cpu_work_master_cycles, trace.vwf_work_since_nmi,
                trace.vwf_first_step_at,
                trace.vwf_last_step_at, trace.vwf_cursor_work, trace.vwf_draw_entry_work,
                trace.nmi_cpu_work_by_block,
                trace.nmis, trace.nmi_completions);
        }
        self.native_exact_cpu_host_trace = Some(trace);
    }
}

impl NativeExactCpuOwner {
    pub(super) fn advance_host(
        &mut self,
        host: u32,
        raw_input: u16,
        reads: Option<&mut Vec<NativeExactCpuRead>>,
        writes: Option<&mut Vec<NativeExactCpuWrite>>,
        instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
    ) -> Result<NativeExactCpuHostTrace, SourceCpuError> {
        self.advance_host_with_hook(host, raw_input, reads, writes, instructions, |_, _| Ok(()))
    }
}

impl NativeExactCpuOwner {
    pub(super) fn advance_host_with_hook<E>(
        &mut self,
        host: u32,
        raw_input: u16,
        mut reads: Option<&mut Vec<NativeExactCpuRead>>,
        mut writes: Option<&mut Vec<NativeExactCpuWrite>>,
        mut instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
        mut hook: impl FnMut(
            &mut Snes9xColdCpuExecutor,
            Option<NativeExactCpuNmiCompletion>,
        ) -> Result<(), E>,
    ) -> Result<NativeExactCpuHostTrace, E>
    where
        E: From<SourceCpuError>,
    {
        assert_eq!(
            host, self.next_host,
            "exact CPU host sequence must be contiguous"
        );
        self.cpu.set_libretro_joypad_words(raw_input, 0);
        let mut trace = NativeExactCpuHostTrace {
            host,
            track_story_decompression_writes: crate::debug_env::var_os(
                "ZELDA3_NATIVE_EXACT_CPU_STORY_DECOMP_LIVE_TRIAL",
            ).is_some(),
            track_dialogue_decode_cursor_writes: crate::debug_env::var_os(
                "ZELDA3_NATIVE_EXACT_CPU_DIALOGUE_DECODE_LIVE_TRIAL",
            ).is_some(),
            track_vwf_cpu_work: crate::debug_env::var_os(
                "ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_VWF_WORK",
            ).is_some(),
            track_floor_tile_writes: crate::debug_env::var_os(
                "ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL",
            )
            .is_some(),
            wram_watch_address: crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_WATCH_WRAM_ADDR")
                .ok()
                .and_then(|value| {
                    let value = value.trim_start_matches("0x");
                    u16::from_str_radix(value, 16).ok()
                }),
            ..Default::default()
        };
        let trace_decompression_entry = crate::debug_env::var!("ZELDA3_NATIVE_EXACT_CPU_TRACE_HOST")
            .ok()
            .and_then(|value| {
                let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
            })
            .is_some_and(|(first, last)| (first..=last).contains(&host));
        // Diagnostic CPU work only: hardware stalls and interrupt-entry bus
        // transactions must not be mistaken for native game-thread work.
        let mut cpu_work = [0u64; 4]; // main, wait, handler, interrupt entry
        let mut main_loop_own_work = 0u64;
        let mut main_work_frames: Vec<(u32, u16)> = Vec::new();
        let mut main_work_by_routine = std::collections::BTreeMap::<u32, u64>::new();
        let mut main_work_uninterrupted = false;
        self.cpu
            .run_until_main_loop_return_with_quiescent_step_hook(|step, cpu| {
                if trace_decompression_entry {
                    let interrupt_start = step.accepted_interrupt.map(|interrupt| match interrupt {
                        SourceCpuAcceptedInterrupt::Nmi { started_at, .. }
                        | SourceCpuAcceptedInterrupt::Irq { started_at, .. } => started_at.master_cycles(),
                    });
                    let instruction_owner = if !self.interrupt_stack.is_empty() { 2 }
                        else if matches!(step.origin_pc, 0x00_8034 | 0x00_8036) { 1 }
                        else { 0 };
                    if instruction_owner == 0 && step.origin_pc == 0x00_8038 {
                        main_work_frames.clear();
                        main_work_frames.push((0x00_8034, cpu.machine().snes().cpu.sp));
                        main_work_by_routine.clear();
                        main_work_uninterrupted = true;
                    }
                    for transaction in &step.transactions {
                        let entry = interrupt_start.is_some_and(|at|
                            transaction.started_at.master_cycles() >= at);
                        let cycles = u64::from(transaction.duration_master_cycles);
                        cpu_work[if entry { 3 } else { instruction_owner }] += cycles;
                        if !entry && instruction_owner == 0 && main_work_uninterrupted {
                            let routine = main_work_frames.last().expect("main profile root").0;
                            *main_work_by_routine.entry(routine).or_default() += cycles;
                        }
                        if !entry && instruction_owner == 0 && matches!(step.origin_pc,
                            0x00_8038 | 0x00_8039 | 0x00_8051 | 0x00_8053 | 0x00_8056
                            | 0x00_805a | 0x00_805d | 0x00_805f)
                        {
                            main_loop_own_work += cycles;
                        }
                    }
                    // Emit only complete uninterrupted iterations: a host
                    // starting inside a suspended callee has no reconstructed
                    // call stack and must not be presented as a full profile.
                    if step.accepted_interrupt.is_some() && instruction_owner == 0 {
                        main_work_uninterrupted = false;
                    }
                    if instruction_owner == 0 && main_work_uninterrupted {
                        let registers = &cpu.machine().snes().cpu;
                        while main_work_frames.len() > 1
                            && registers.sp > main_work_frames.last().unwrap().1
                        {
                            main_work_frames.pop();
                        }
                        if matches!(step.opcode, 0x20 | 0x22 | 0xfc) {
                            main_work_frames.push((u32::from(registers.k) << 16
                                | u32::from(registers.pc), registers.sp));
                        }
                        if step.origin_pc == 0x00_805f {
                            for (routine, cycles) in &main_work_by_routine {
                                eprintln!("native-exact-main-work host={host} routine={routine:06x} cycles={cycles}");
                            }
                            main_work_uninterrupted = false;
                        }
                    }
                }
                if trace_decompression_entry
                    && matches!(step.origin_pc, 0x0e_c984 | 0x0e_ccec | 0x0e_ccf1)
                {
                    eprintln!("native-exact-vwf-instruction host={} pc={:06x} start={} end={} after={:?}",
                        host, step.origin_pc, step.started_at.master_cycles(),
                        step.ended_at.master_cycles(), cpu.raster_position());
                }
                if trace_decompression_entry && step.origin_pc == 0x06_8328 {
                    let ram = &cpu.machine().snes().ram;
                    let sprites: Vec<_> = (0..16).filter(|&slot| ram[0xdd0 + slot] != 0)
                        .map(|slot| (slot, ram[0xdd0 + slot], ram[0xe20 + slot])).collect();
                    let ancilla: Vec<_> = (0..10).filter(|&slot| ram[0xc4a + slot] != 0)
                        .map(|slot| (slot, ram[0xc4a + slot])).collect();
                    let overlords: Vec<_> = (0..8).filter(|&slot| ram[0xb00 + slot] != 0)
                        .map(|slot| (slot, ram[0xb00 + slot])).collect();
                    eprintln!("native-exact-sprite-entry host={host} at={} module={:02x}/{:02x}/{:02x} indoor={} follower={:02x} garnish={:02x} sprites={sprites:02x?} ancilla={ancilla:02x?} overlords={overlords:02x?}",
                        step.started_at.master_cycles(), ram[0x10], ram[0x11], ram[0xb0],
                        ram[0x1b], ram[0xf3cc], ram[0xfb4]);
                }
                if trace_decompression_entry
                    && matches!(step.origin_pc, 0x00_e772 | 0x00_e783)
                {
                    let snes = cpu.machine().snes();
                    let sp = usize::from(snes.cpu.sp);
                    let return_pc = u16::from_le_bytes([
                        snes.ram[(sp + 1) & 0x1ff],
                        snes.ram[(sp + 2) & 0x1ff],
                    ]).wrapping_add(1);
                    eprintln!(
                        "native-exact-cpu-decompress-entry host={} pc={:06x} caller={:04x} y={:04x} dest={:02x}:{:04x}",
                        host, step.origin_pc, return_pc, snes.cpu.y,
                        snes.ram[0x02], u16::from_le_bytes([snes.ram[0], snes.ram[1]]),
                    );
                }
                if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_TILESET_PHASES").is_some()
                    && step.origin_pc == 0x00_e19b
                {
                    eprintln!("native-tileset-source-raster host={} pc={:06x} start={} end={} after={:?}",
                        trace.host, step.origin_pc, step.started_at.master_cycles(),
                        step.ended_at.master_cycles(), cpu.raster_position());
                }
                self.sprite_coordinates.observe(step.origin_pc, cpu.machine().snes().ram[0x0fa0]);
                let completed = Self::record_step(
                    &mut self.interrupt_stack,
                    &mut self.overworld_scan,
                    &mut self.sprite_preparation,
                    &mut self.scroll_copy,
                    &mut self.floor_draw,
                    &mut trace,
                    step,
                    reads.as_deref_mut(),
                    writes.as_deref_mut(),
                    instructions.as_deref_mut(),
                );
                if matches!(
                    step.accepted_interrupt,
                    Some(SourceCpuAcceptedInterrupt::Nmi {
                        interrupted_pc: 0x1d_e9f8,
                        ..
                    })
                ) {
                    let slot = cpu.machine().snes().ram[0x0fa0];
                    assert!(slot < 16, "cached sprite slot outside source loop");
                    trace.cached_slot_returned_at_nmi = Some(slot);
                }
                self.cached_sprite.observe(step, cpu);
                self.follower_graphics.observe(step, cpu.machine().snes().ram[0x0fa0]);
                if let Some(progress) = self.animated_overworld_graphics.observe(step) {
                    trace.animated_overworld_graphics_progress = Some(progress);
                }
                if let Some(progress) = self.animated_dungeon_graphics.observe(step) {
                    trace.animated_dungeon_graphics_progress = Some(progress);
                }
                self.pre_overworld_tilesets.observe(step, cpu.machine().snes().cpu.y, &mut trace.pre_overworld_tileset_publications);
                self.pre_dungeon_tilesets.observe_call(step, cpu.machine().snes().cpu.y,
                    &mut trace.pre_dungeon_tileset_publications, 0x02_8286, 0x02_828a);
                if let Some(words) = self.pre_dungeon_palettes.observe(step) {
                    trace.pre_dungeon_palette_cache = Some(words);
                }
                if let Some(progress) = self.overworld_map8.observe(step) {
                    trace.overworld_map8_progress = Some(progress);
                }
                if let Some(stores) = self.overworld_quadrants.observe(step) {
                    trace.overworld_quadrant_stores = Some(stores);
                }
                if let Some(steps) = self.initial_scroll.observe(step) {
                    trace.initial_scroll_steps = Some(steps);
                }
                trace.observe_uncle_priest_dispatch(step, cpu.machine().snes().ram[0x0fa0]);
                if step.origin_pc == 0x05_aa97 {
                    let ram = &cpu.machine().snes().ram;
                    let slot = ram[0x0fa0];
                    trace.keese_inactive_check_slot = (slot < 16
                        && ram[0x0dd0 + usize::from(slot)] == 9
                        && ram[0x0e20 + usize::from(slot)] == 0x6f).then_some(slot);
                } else if matches!(step.origin_pc, 0x05_aa9a | 0x06_84e2) {
                    trace.keese_inactive_check_slot = None;
                }
                if step.origin_pc == 0x06_e48d {
                    let ram = &cpu.machine().snes().ram;
                    let slot = ram[0x0fa0];
                    trace.guard_offscreen_kill_self_slot = (slot < 16
                        && ram[0x0dd0 + usize::from(slot)] == 9
                        && (0x41..=0x44).contains(&ram[0x0e20 + usize::from(slot)]))
                        .then_some(slot);
                } else if step.origin_pc == 0x09_f24a {
                    trace.guard_offscreen_kill_self_slot = None;
                }
                self.dungeon_cache.observe(step, cpu);
                if let Some(SourceCpuAcceptedInterrupt::Nmi { interrupted_pc, .. }) = step.accepted_interrupt {
                    trace.quadrant_copy_boundary = crate::zelda_rtl::dungeon::DungeonQuadrantUploadWork::completed_words_at_source_pc(
                        interrupted_pc, cpu.machine().snes().cpu.y,
                    ).map(|words| (words, OriginalTimingBoundary::NmiAccepted));
                    trace.dungeon_load_boundary = crate::DungeonSpriteLoadCheckpoint::from_next_source_pc(interrupted_pc)
                        .map(|checkpoint| (cpu.machine().snes().cpu.x as u8, checkpoint, OriginalTimingBoundary::NmiAccepted));
                    trace.object_4x2_boundary = if self.floor_draw.object_in_progress {
                        let ram = &cpu.machine().snes().ram;
                        let remaining = u16::from_le_bytes([ram[0xb2], ram[0xb3]]);
                        crate::zelda_rtl::dungeon::Dungeon4x2DrawBoundary::from_source_pc(interrupted_pc, remaining)
                            .map(|boundary| (self.floor_draw.completed_objects + 1, boundary))
                    } else { None };
                }
                hook(cpu, completed)
            })?;
        self.source_nmis_accepted += trace.nmis.len() as u64;
        self.source_nmis_completed += trace.nmi_completions.len() as u64;
        trace.pending_nmis_at_return = self
            .interrupt_stack
            .iter()
            .filter_map(|interrupt| match interrupt {
                NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                NativeExactCpuInterrupt::Irq => None,
            })
            .collect();
        let registers = &self.cpu.machine().snes().cpu;
        trace.return_pc = (u32::from(registers.k) << 16) | u32::from(registers.pc);
        trace.zelda_follower_graphics_progress = self.follower_graphics.active;
        if self.scroll_copy.active {
            trace.scroll_copy_progress = Some(self.scroll_copy.progress);
        }
        if (0x00_85fe..=0x00_865a).contains(&trace.return_pc) {
            let progress = self.sprite_preparation.progress
                .expect("source returned inside extended-OAM packing without a cursor");
            progress.validate();
            trace.sprite_preparation_progress_at_return = Some(progress);
        }
        if trace_decompression_entry && trace.sprite_decompression_progress.is_some() {
            eprintln!("native-exact-cpu-decompress-progress host={} progress={:?}",
                host, trace.sprite_decompression_progress);
        }
        if trace_decompression_entry && !trace.dialogue_decode_cursor_writes.is_empty() {
            eprintln!("native-exact-cpu-dialogue-cursor host={} first={:?} last={:?} stores={}",
                host,
                trace.dialogue_decode_cursor_writes.first(),
                trace.dialogue_decode_cursor_writes.last(),
                trace.dialogue_decode_cursor_writes.len());
        }
        if trace_decompression_entry && (0x00_e79e..=0x00_e851).contains(&trace.return_pc) {
            let ram = &self.cpu.machine().snes().ram;
            let sp = usize::from(registers.sp);
            let return_words = (0..5)
                .map(|offset| {
                    u16::from_le_bytes([
                        ram[(sp + 1 + offset * 2) & 0x1ff],
                        ram[(sp + 2 + offset * 2) & 0x1ff],
                    ])
                })
                .collect::<Vec<_>>();
            eprintln!(
                "native-exact-cpu-decompress-return host={} pc={:06x} y={:04x} sp={:04x} stack={return_words:04x?}",
                host, trace.return_pc, registers.y, registers.sp,
            );
        }
        let cached_boundary_nmi = trace
            .pending_nmis_at_return
            .iter()
            .rev()
            .find(|nmi| trace.nmis.contains(nmi));
        let cached_stop_pc = cached_boundary_nmi.map_or(trace.return_pc, |nmi| nmi.interrupted_pc);
        // These are the shared helper's initial register-only state test.
        // The observed AA97 JSR identifies Keese, including an NMI whose
        // handler has changed SP; do not guess a return-address stack offset.
        if !matches!(cached_stop_pc, 0x05_f94e | 0x05_f951 | 0x05_f953) {
            trace.keese_inactive_check_slot = None;
        }
        if cached_stop_pc == 0x05_a8c1 {
            let ram = &self.cpu.machine().snes().ram;
            let slot = ram[0x0fa0];
            if slot < 16 && ram[0x0dd0 + usize::from(slot)] == 9
                && ram[0x0e20 + usize::from(slot)] == 0x6d {
                trace.rat_oam_flags_pending_slot = Some(slot);
            }
        }
        trace.sprite_coordinate_prefix = self.sprite_coordinates.prefix(cached_stop_pc)
            .map(|(slot, next_instruction)| (slot, next_instruction,
                if cached_boundary_nmi.is_some() { OriginalTimingBoundary::NmiAccepted }
                else { OriginalTimingBoundary::HostReturn }));
        if cached_boundary_nmi.is_none() && cached_stop_pc == 0x05_f955 {
            let ram = &self.cpu.machine().snes().ram;
            let slot = ram[0x0fa0];
            let sp = usize::from(registers.sp);
            let caller = u16::from_le_bytes([ram[(sp + 1) & 0x1ff], ram[(sp + 2) & 0x1ff]]);
            // The shared pause helper has many callers. Only Guard_Main's
            // JSR at C25C proves that its draw and pose restore have returned.
            if caller == 0xc25e && slot < 16
                && ram[0x0dd0 + usize::from(slot)] == 9
                && (0x41..=0x44).contains(&ram[0x0e20 + usize::from(slot)]) {
                trace.guard_active_state_checked_slot = Some(slot);
            }
        }
        if trace_decompression_entry {
            let ram = &self.cpu.machine().snes().ram;
            let slot = usize::from(ram[0x0fa0]);
            if slot < 16 {
                let sp = usize::from(registers.sp);
                let stack = (1..=16).map(|offset| ram[(sp + offset) & 0x1ff]).collect::<Vec<_>>();
                eprintln!("native-exact-cpu-sprite-stop host={host} pc={cached_stop_pc:06x} slot={slot} state={:02x} type={:02x} x={:04x} y={:04x} sp={:04x} stack={stack:02x?}",
                    ram[0x0dd0 + slot], ram[0x0e20 + slot], registers.x, registers.y, registers.sp);
            }
        }
        trace.follower_body_draw_prefix = crate::zelda_rtl::tagalong::follower_body_head_prefix_cycles(cached_stop_pc)
            .map(|head_prefix_cycles| (SpriteMainCpuBoundary::FollowerBodyDraw {
                head_prefix_cycles, continuation: None,
            }, if cached_boundary_nmi.is_some() {
                OriginalTimingBoundary::NmiAccepted
            } else {
                OriginalTimingBoundary::HostReturn
            }));
        // Animation selection before AA00 has only register/stack and direct-
        // page scratch effects. OAM allocation and every draw remain pending.
        if (0x09_a959..0x09_aa00).contains(&cached_stop_pc) {
            trace.follower_body_draw_prefix = Some((
                SpriteMainCpuBoundary::FollowerAnimationSelection { inputs: None },
                if cached_boundary_nmi.is_some() {
                    OriginalTimingBoundary::NmiAccepted
                } else {
                    OriginalTimingBoundary::HostReturn
                },
            ));
        }
        if matches!(cached_stop_pc, 0x09_9f97 | 0x09_9f98) {
            trace.follower_body_draw_prefix = Some((
                SpriteMainCpuBoundary::FollowerWrapperReturn {
                    plb_completed: cached_stop_pc == 0x09_9f98,
                },
                if cached_boundary_nmi.is_some() { OriginalTimingBoundary::NmiAccepted }
                else { OriginalTimingBoundary::HostReturn },
            ));
        }
        if cached_boundary_nmi.is_none() {
            let ram = &self.cpu.machine().snes().ram;
            let sp = usize::from(registers.sp);
            let caller = u16::from_le_bytes([ram[(sp + 1) & 0x1ff], ram[(sp + 2) & 0x1ff]]);
            // The same coordinate helper draws splash and body sprites too.
            // This return address names only the head's JSR at AB62.
            if caller == 0xab64 {
                if let Some(prefix) = crate::zelda_rtl::tagalong::follower_coordinate_entry_prefix(cached_stop_pc) {
                    trace.follower_body_draw_prefix = Some((SpriteMainCpuBoundary::FollowerHeadCoordinates {
                        prefix, continuation: None,
                    }, OriginalTimingBoundary::HostReturn));
                }
            }
        }
        // Through A98E, body selection touches only registers and DP scratch;
        // the graphics-index store has returned, but no body OAM store has.
        trace.link_oam_body_selection = matches!(cached_stop_pc, 0x0d_a96d | 0x0d_a98e);
        // The equipment eligibility tests after initial sprite banks modify
        // only registers/CPU stack; the sword call at A635 is still pending.
        trace.link_oam_equipment_selection = (0x0d_a5ce..=0x0d_a635).contains(&cached_stop_pc);
        trace.link_oam_lower_body_progress =
            player_oam::LinkOamLowerBodyProgress::from_source_pc(cached_stop_pc);
        // All entries in KillSelf's deflection/indoor tests precede its first
        // store ($09:F203). Resuming native re-evaluates only those reads.
        if !matches!(cached_stop_pc, 0x09_f1f8 | 0x09_f1fb | 0x09_f1fd
            | 0x09_f1ff | 0x09_f201 | 0x09_f203) {
            trace.guard_offscreen_kill_self_slot = None;
        }
        // Keep a concrete shared JumpTableLocal entry only while that call
        // is suspended. Its caller was observed, never inferred from NMI's SP.
        if !(0x00_8781..=0x00_8799).contains(&cached_stop_pc) {
            trace.sprite_timer_return_slot = None;
        }
        // SpriteActive_Main dispatches solely through CPU registers and stack.
        // Guard_Main saves graphics/facing in CPU registers and on the stack
        // before its first WRAM store at $05:C237. Those statements require
        // no native gameplay mutation beyond the generic timer/OAM prefix.
        if matches!(cached_stop_pc, 0x06_9271..=0x06_9282
            // The second active dispatcher (types $41..$70) also only
            // selects a handler through registers and stack. Its timer/OAM
            // prefix has returned, but the selected sprite body has not run.
            // Exclude the trampoline's PLB/RTL and bounce RTS after that body.
            | 0x06_bfea | 0x05_b5c3 | 0x05_b5c4 | 0x05_b5c5 | 0x05_b5c6
            | 0x05_b5d3 | 0x05_b5d6 | 0x05_b5d7 | 0x05_b5d9
            | 0x05_b5db | 0x05_b5de | 0x05_b5df | 0x05_b5e0
            | 0x05_b5e3 | 0x05_b5e4 | 0x05_b5e5 | 0x05_b5e7
            // BlueGuard's LDA/BNE/JMP dispatch has no persistent writes;
            // either Guard_Main or Probe still owns all its body work.
            | 0x05_c155 | 0x05_c158 | 0x05_c15a
            | 0x05_c227 | 0x05_c22a | 0x05_c22b | 0x05_c22e
            | 0x05_c22f | 0x05_c232 | 0x05_c234 | 0x05_c237) {
            let ram = &self.cpu.machine().snes().ram;
            let slot = ram[0x0fa0];
            if slot < 16 && ram[0x0dd0 + usize::from(slot)] == 9 {
                trace.sprite_timer_return_slot = Some(slot);
            }
        }
        trace.cached_sprite_progress = (cached_stop_pc == 0x1d_e9f8)
            .then_some(trace.cached_slot_returned_at_nmi)
            .flatten()
            .map(crate::CachedSpriteExecutionProgress::AfterSlot)
            .or_else(|| self.cached_sprite.progress_at_return(cached_stop_pc, &self.cpu))
            .map(|progress| crate::CachedSpriteExecutionProgressReceipt {
                progress,
                boundary: if cached_boundary_nmi.is_some() {
                    crate::OriginalTimingBoundary::NmiAccepted
                } else {
                    crate::OriginalTimingBoundary::HostReturn
                },
            });
        trace.dungeon_reset_progress = self.dungeon_cache.progress_at_return(
            cached_stop_pc,
            if cached_boundary_nmi.is_some() {
                OriginalTimingBoundary::NmiAccepted
            } else {
                OriginalTimingBoundary::HostReturn
            },
        );
        if cached_boundary_nmi.is_none() {
            trace.quadrant_copy_boundary = crate::zelda_rtl::dungeon::DungeonQuadrantUploadWork::completed_words_at_source_pc(
                trace.return_pc, registers.y,
            ).map(|words| (words, OriginalTimingBoundary::HostReturn));
            trace.dungeon_load_boundary = crate::DungeonSpriteLoadCheckpoint::from_next_source_pc(trace.return_pc)
                .map(|checkpoint| (registers.x as u8, checkpoint, OriginalTimingBoundary::HostReturn));
            trace.object_4x2_boundary = if self.floor_draw.object_in_progress {
                let ram = &self.cpu.machine().snes().ram;
                let remaining = u16::from_le_bytes([ram[0xb2], ram[0xb3]]);
                crate::zelda_rtl::dungeon::Dungeon4x2DrawBoundary::from_source_pc(trace.return_pc, remaining)
                    .map(|boundary| (self.floor_draw.completed_objects + 1, boundary))
            } else { None };
        }
        drop(self.cpu.take_dsp_samples());
        if trace_decompression_entry {
            eprintln!("native-exact-cpu-work host={host} main={} wait={} handler={} interrupt_entry={} main_loop_own={main_loop_own_work}",
                cpu_work[0], cpu_work[1], cpu_work[2], cpu_work[3]);
        }
        self.next_host += 1;
        Ok(trace)
    }
}

impl ZeldaState {
    pub fn restore_native_exact_cpu_checkpoint(&mut self, bytes: &[u8]) -> Result<(), String> {
        let owner = if bytes.first() == Some(&4) {
            let checkpoint: NativeExactCpuOwnerCheckpointV4 =
                bincode::deserialize(bytes).map_err(|error| error.to_string())?;
            if checkpoint.version != 4 {
                return Err(format!("unsupported native exact CPU checkpoint wrapper version {}", checkpoint.version));
            }
            let mut owner = NativeExactCpuOwner::from_checkpoint(checkpoint.owner)?;
            let mut cursors = checkpoint.scroll_cursors.into_iter();
            owner.restore_scroll_cursors(&mut cursors)?;
            if cursors.next().is_some() {
                return Err("native exact CPU checkpoint has extra scroll cursors".to_string());
            }
            let dialogue = checkpoint.dialogue;
            self.dialogue_fast_forward_hold_pending = dialogue.fast_forward_hold_pending;
            self.dialogue_fast_forward_hold_active = dialogue.fast_forward_hold_active;
            self.dialogue_vwf_completion_stops_before_scroll = dialogue.completion_stops_before_scroll;
            self.dialogue_vwf_handler_completed_at_endpoint = dialogue.handler_completed_at_endpoint;
            self.dialogue_live_message_read_position_target = dialogue.live_message_read_position_target;
            self.dialogue_vwf_handler_entry_phase = dialogue.handler_entry_phase;
            self.dialogue_vwf_glyph_cpu_phase = dialogue.glyph_cpu_phase;
            self.dialogue_vwf_pending_glyph_read_pos = dialogue.pending_glyph_read_pos;
            self.dialogue_vwf_dispatch_cursor = dialogue.dispatch_cursor;
            self.dialogue_character_load_progress = dialogue.character_load_progress;
            self.dialogue_vwf_deferred_handler_exits = dialogue.deferred_handler_exits;
            self.native_scroll_copy_cursor = dialogue.scroll_copy_cursor;
            self.dialogue_vwf_transition_remaining_master_cycles = checkpoint.transition_remaining_master_cycles;
            owner
        } else if bytes.first() == Some(&3) {
            let checkpoint: NativeExactCpuOwnerCheckpointV3 =
                bincode::deserialize(bytes).map_err(|error| error.to_string())?;
            if checkpoint.version != 3 {
                return Err(format!("unsupported native exact CPU checkpoint wrapper version {}", checkpoint.version));
            }
            let mut owner = NativeExactCpuOwner::from_checkpoint(checkpoint.owner)?;
            let mut cursors = checkpoint.scroll_cursors.into_iter();
            owner.restore_scroll_cursors(&mut cursors)?;
            if cursors.next().is_some() {
                return Err("native exact CPU checkpoint has extra scroll cursors".to_string());
            }
            let dialogue = checkpoint.dialogue;
            self.dialogue_fast_forward_hold_pending = dialogue.fast_forward_hold_pending;
            self.dialogue_fast_forward_hold_active = dialogue.fast_forward_hold_active;
            self.dialogue_vwf_completion_stops_before_scroll = dialogue.completion_stops_before_scroll;
            self.dialogue_vwf_handler_completed_at_endpoint = dialogue.handler_completed_at_endpoint;
            self.dialogue_live_message_read_position_target = dialogue.live_message_read_position_target;
            self.dialogue_vwf_handler_entry_phase = dialogue.handler_entry_phase;
            self.dialogue_vwf_glyph_cpu_phase = dialogue.glyph_cpu_phase;
            self.dialogue_vwf_pending_glyph_read_pos = dialogue.pending_glyph_read_pos;
            self.dialogue_vwf_dispatch_cursor = dialogue.dispatch_cursor;
            self.dialogue_character_load_progress = dialogue.character_load_progress;
            self.dialogue_vwf_deferred_handler_exits = dialogue.deferred_handler_exits;
            self.native_scroll_copy_cursor = dialogue.scroll_copy_cursor;
            owner
        } else if bytes.first() == Some(&2) {
            let checkpoint: NativeExactCpuOwnerCheckpointV2 =
                bincode::deserialize(bytes).map_err(|error| error.to_string())?;
            if checkpoint.version != 2 {
                return Err(format!("unsupported native exact CPU checkpoint wrapper version {}", checkpoint.version));
            }
            let mut owner = NativeExactCpuOwner::from_checkpoint(checkpoint.owner)?;
            let mut cursors = checkpoint.scroll_cursors.into_iter();
            owner.restore_scroll_cursors(&mut cursors)?;
            if cursors.next().is_some() {
                return Err("native exact CPU checkpoint has extra scroll cursors".to_string());
            }
            if owner.scroll_copy.active {
                return Err("version-2 native exact CPU checkpoint omitted the active native scroll cursor".to_string());
            }
            owner
        } else {
            let checkpoint: NativeExactCpuOwnerCheckpoint =
                bincode::deserialize(bytes).map_err(|error| error.to_string())?;
            NativeExactCpuOwner::from_checkpoint(checkpoint)?
        };
        if owner.next_host != self.frame_ctr_dbg {
            return Err(format!(
                "native exact CPU checkpoint host {} differs from game host {}",
                owner.next_host, self.frame_ctr_dbg
            ));
        }
        self.native_exact_cpu_owner = Some(owner);
        self.native_exact_cpu_host_trace = None;
        Ok(())
    }
}

impl ZeldaState {
    pub fn capture_native_exact_cpu_checkpoint(&self) -> Result<Option<Vec<u8>>, String> {
        let Some(owner) = self.native_exact_cpu_owner.as_ref() else {
            return Ok(None);
        };
        if owner.next_host != self.frame_ctr_dbg {
            return Err("native exact CPU checkpoint is not at the game host boundary".to_string());
        }
        if !self.native_exact_cpu_dialogue_checkpoint_is_quiescent() {
            return Err("native exact CPU checkpoint cannot capture a held VWF timing anchor".to_string());
        }
        if self.native_pre_overworld_animated_graphics.is_some()
            || self.native_pre_dungeon_animated_graphics.is_some()
            || self.native_pre_overworld_tileset_publication.is_some()
            || self.native_pre_dungeon_tileset_publication.is_some()
            || self.native_pre_dungeon_palette_cache.is_some()
            || self.native_overworld_map8.is_some()
            || self.native_overworld_quadrants.is_some()
            || self.native_initial_scroll.is_some()
            || self.native_overworld_spotlight_link_oam.is_some() {
            return Err("native exact CPU checkpoint cannot capture active animated overworld graphics".to_string());
        }
        if self.native_exact_cpu_overworld_live_scan.is_some()
            || self.native_exact_cpu_overworld_scan_trial.is_some() {
            return Err("native exact CPU checkpoint cannot capture an active native overworld scan".to_string());
        }
        if matches!(self.dialogue_scroll_phase(),
            crate::zelda_rtl::DialogueScrollPhase::CompletionStagedAfterFrozenScanout
            | crate::zelda_rtl::DialogueScrollPhase::CompletionStagedAfterSnapshot)
        {
            return Err("native exact CPU checkpoint cannot capture staged dialogue scroll publication".to_string());
        }
        if self.active_spiral_aux_graphics.is_some() {
            return Err("native exact CPU checkpoint cannot capture partial auxiliary graphics".to_string());
        }
        let mut scroll_cursors = Vec::new();
        owner.collect_scroll_cursors(&mut scroll_cursors);
        bincode::serialize(&NativeExactCpuOwnerCheckpointV4 {
            version: 4,
            owner: owner.capture_checkpoint()?,
            scroll_cursors,
            dialogue: NativeExactCpuDialogueContinuationCheckpoint {
                fast_forward_hold_pending: self.dialogue_fast_forward_hold_pending,
                fast_forward_hold_active: self.dialogue_fast_forward_hold_active,
                completion_stops_before_scroll: self.dialogue_vwf_completion_stops_before_scroll,
                handler_completed_at_endpoint: self.dialogue_vwf_handler_completed_at_endpoint,
                live_message_read_position_target: self.dialogue_live_message_read_position_target,
                handler_entry_phase: self.dialogue_vwf_handler_entry_phase,
                glyph_cpu_phase: self.dialogue_vwf_glyph_cpu_phase,
                pending_glyph_read_pos: self.dialogue_vwf_pending_glyph_read_pos,
                dispatch_cursor: self.dialogue_vwf_dispatch_cursor,
                character_load_progress: self.dialogue_character_load_progress.clone(),
                deferred_handler_exits: self.dialogue_vwf_deferred_handler_exits,
                scroll_copy_cursor: self.native_scroll_copy_cursor,
            },
            transition_remaining_master_cycles: self.dialogue_vwf_transition_remaining_master_cycles,
        })
            .map(Some)
            .map_err(|error| error.to_string())
    }
}
