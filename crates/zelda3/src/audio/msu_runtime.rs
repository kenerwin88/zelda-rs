use super::*;

impl ZeldaState {
    fn remap_msu_deluxe_track(&self, mp: &MsuPlayer, track: u8) -> u8 {
        if mp.enabled & MSU_FEATURE_MSU_DELUXE == 0
            || track as usize >= MSU_DELUXE_TRACK_ROUTE.len()
        {
            return track;
        }
        match MSU_DELUXE_TRACK_ROUTE[track as usize] {
            1 => {
                let area = self.game_state.world.region.overworld_area_index() as usize & 0xff;
                if area < MSU_DELUXE_OVERWORLD_TRACKS.len() {
                    MSU_DELUXE_OVERWORLD_TRACKS[area]
                } else {
                    track
                }
            }
            2 => {
                let entrance = self.game_state.world.region.which_entrance() as usize;
                if entrance >= MSU_DELUXE_ENTRANCE_TRACKS.len()
                    || MSU_DELUXE_ENTRANCE_TRACKS[entrance] == 242
                {
                    track
                } else {
                    MSU_DELUXE_ENTRANCE_TRACKS[entrance]
                }
            }
            _ => track,
        }
    }

    pub fn zelda_is_playing_music_track(&self, track: u8) -> bool {
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            self.remap_msu_deluxe_track(mp, track) == mp.resume_info.actual_track
        } else {
            track == self.game_state.system_signals.current_music_control()
        }
    }

    pub fn zelda_is_playing_music_track_with_bug(&self, track: u8) -> bool {
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            self.remap_msu_deluxe_track(mp, track) == mp.resume_info.actual_track
        } else if self
            .game_state
            .enhanced_features
            .has(FEATURES0_MISC_BUG_FIXES_AUDIO)
        {
            track == self.game_state.system_signals.current_music_control()
        } else {
            track == self.game_state.system_signals.last_music_control()
        }
    }

    pub fn zelda_get_entrance_music_track(&self, i: i32) -> u8 {
        let mut rv = self
            .assets
            .as_ref()
            .and_then(|assets| assets.asset(27))
            .and_then(|asset| asset.get(i as usize))
            .copied()
            .unwrap_or(0);
        let mp = &self.audio.msu_player;
        if mp.state != MSU_STATE_IDLE && mp.enabled & MSU_FEATURE_MSU_DELUXE != 0 {
            let entrance = self.game_state.world.region.which_entrance() as usize;
            if rv == 242
                && entrance < MSU_DELUXE_ENTRANCE_TRACKS.len()
                && MSU_DELUXE_ENTRANCE_TRACKS[entrance] != 242
            {
                rv = 16;
            }
        }
        rv
    }

    pub fn zelda_play_msu_audio_track(&mut self, music_ctrl: u8) {
        if self.audio.msu_player.enabled == 0 {
            self.audio.msu_player.resume_info.tag = 0;
            self.zelda_apu_write(0x2140, music_ctrl);
            return;
        }
        if music_ctrl & 0xf0 != 0xf0 {
            self.msu_player_open(music_ctrl as i32, false);
        } else if (0xf1..=0xf3).contains(&music_ctrl) {
            let i = (music_ctrl - 0xf1) as usize;
            self.audio.msu_player.volume_target = self.audio.volume_transition_target_float[i];
            self.audio.msu_player.volume_step = self.audio.volume_transition_step_float[i];
        }
        if self.audio.msu_player.state == 0 {
            self.zelda_apu_write(0x2140, music_ctrl);
        } else {
            self.zelda_apu_write(0x2140, 0xf0);
        }
    }

    fn msu_player_close_file(mp: &mut MsuPlayer) {
        mp.has_file = false;
        mp.has_opus = false;
        mp.opus_decoder = None;
        mp.pcm_data.clear();
        mp.opuz_data.clear();
        if mp.state != MSU_STATE_FINISHED_PLAYING {
            mp.state = MSU_STATE_IDLE;
        }
        mp.resume_info = MsuResumeInfoState::default();
    }

    pub(super) fn msu_player_open(&mut self, orig_track: i32, resume_from_snapshot: bool) {
        let actual_track = {
            let mp = &self.audio.msu_player;
            self.remap_msu_deluxe_track(mp, orig_track as u8)
        };

        let resume = if !resume_from_snapshot {
            let mut resume = MsuResumeInfoState::default();
            if self.game_state.frame.main_module == 9
                && actual_track == self.msu_resume_info(MsuResumeSlot::Alternate).actual_track
                && self.audio.config_resume_msu
            {
                resume = self.msu_resume_info(MsuResumeSlot::Alternate);
            }
            if self.audio.msu_player.state >= MSU_STATE_RESUMING {
                self.save_msu_resume_info(
                    MsuResumeSlot::Alternate,
                    self.audio.msu_player.resume_info,
                );
            }
            resume
        } else {
            self.msu_resume_info(MsuResumeSlot::Primary)
        };

        let volume_target = self.audio.volume_transition_target_float[3];
        let volume_step = self.audio.volume_transition_step_float[3];
        let config_msu_path = self.audio.config_msu_path.clone();
        let mp = &mut self.audio.msu_player;
        mp.volume_target = volume_target;
        mp.volume_step = volume_step;
        mp.state = MSU_STATE_IDLE;
        Self::msu_player_close_file(mp);
        if actual_track == 0 {
            return;
        }

        let ext = if mp.enabled & MSU_FEATURE_OPUZ != 0 {
            "opuz"
        } else {
            "pcm"
        };
        let prefix = config_msu_path.as_deref().unwrap_or("");
        let fname = format!("{prefix}{actual_track}.{ext}");
        let Ok(bytes) = fs::read(config_value_path(&fname)) else {
            Self::msu_player_close_file(mp);
            return;
        };
        if bytes.len() < 8 {
            Self::msu_player_close_file(mp);
            return;
        }

        let Some(file_tag) = read_le_u32(&bytes, 0) else {
            Self::msu_player_close_file(mp);
            return;
        };
        let Some(repeat_position) = read_le_u32(&bytes, 4) else {
            Self::msu_player_close_file(mp);
            return;
        };
        mp.repeat_position = repeat_position;
        mp.state = if resume.actual_track == actual_track && resume.tag == file_tag {
            MSU_STATE_RESUMING
        } else {
            MSU_STATE_PLAYING
        };
        if mp.state == MSU_STATE_RESUMING {
            mp.resume_info = resume;
        } else {
            mp.resume_info.orig_track = orig_track as u8;
            mp.resume_info.actual_track = actual_track;
            mp.resume_info.tag = file_tag;
            mp.resume_info.range_cur = 8;
        }
        mp.cur_file_offs = mp.resume_info.offset;
        mp.samples_until_repeat = mp.resume_info.samples_until_repeat;
        mp.range_cur = mp.resume_info.range_cur;
        mp.range_repeat = mp.resume_info.range_repeat;
        mp.buffer_size = 0;
        mp.buffer_pos = 0;
        mp.preskip = 0;

        if file_tag == OPUZ_TAG {
            let Ok(decoder) = OpusDecoder::new(48_000, Channels::Stereo) else {
                Self::msu_player_close_file(mp);
                return;
            };
            mp.opuz_data = bytes;
            mp.opus_decoder = Some(decoder);
            mp.has_file = true;
            mp.has_opus = true;
        } else if file_tag == MSU1_TAG {
            let sample_frames = ((bytes.len() - 8) / 4) as u32;
            mp.total_samples_in_file = sample_frames;
            mp.samples_until_repeat = sample_frames.wrapping_sub(mp.cur_file_offs);
            mp.pcm_data.clear();
            mp.pcm_data.reserve(sample_frames as usize * 2);
            for chunk in bytes[8..8 + sample_frames as usize * 4].chunks_exact(2) {
                mp.pcm_data.push(i16::from_le_bytes([chunk[0], chunk[1]]));
            }
            mp.has_file = true;
            mp.has_opus = false;
        } else {
            Self::msu_player_close_file(mp);
        }
    }

    pub(super) fn msu_player_prepare_opuz_packet(mp: &mut MsuPlayer) -> OpuzPacketStatus {
        if mp.opus_decoder.is_none() {
            let Ok(decoder) = OpusDecoder::new(48_000, Channels::Stereo) else {
                return OpuzPacketStatus::ReadError;
            };
            mp.opus_decoder = Some(decoder);
        }

        loop {
            if mp.samples_until_repeat == 0 {
                if mp.range_cur == 0 {
                    return OpuzPacketStatus::FinishedPlaying;
                }
                if let Some(decoder) = &mut mp.opus_decoder {
                    if decoder.reset_state().is_err() {
                        return OpuzPacketStatus::ReadError;
                    }
                }
                let range_offset = mp.range_cur as usize;
                let Some(range_header) = mp.opuz_data.get(range_offset..range_offset + 10) else {
                    return OpuzPacketStatus::ReadError;
                };
                let Some(file_offs) = read_le_u32(range_header, 0) else {
                    return OpuzPacketStatus::ReadError;
                };
                if file_offs & 0xf000_0000 != 0 {
                    return OpuzPacketStatus::ReadError;
                }
                let Some(samples_until_repeat) = read_le_u32(range_header, 4) else {
                    return OpuzPacketStatus::ReadError;
                };
                let preskip = u16::from_le_bytes([range_header[8], range_header[9]]);
                mp.samples_until_repeat = samples_until_repeat;
                mp.preskip = (preskip & 0x3fff) as u32;
                if preskip & 0x4000 != 0 {
                    mp.range_repeat = mp.range_cur;
                }
                mp.range_cur = if preskip & 0x8000 != 0 {
                    mp.range_repeat
                } else {
                    mp.range_cur.wrapping_add(10)
                };
                mp.cur_file_offs = file_offs;
                mp.resume_info.range_repeat = mp.range_repeat;
                mp.resume_info.range_cur = mp.range_cur;
            }
            if mp.samples_until_repeat == 0 {
                return OpuzPacketStatus::ReadError;
            }

            let packet_offset = mp.cur_file_offs as usize;
            let Some(packet_header) = mp.opuz_data.get(packet_offset..packet_offset + 2) else {
                return OpuzPacketStatus::ReadError;
            };
            let packet_header = u16::from_le_bytes([packet_header[0], packet_header[1]]);
            let size = (packet_header & 0x7fff) as usize;
            if size > 1275 {
                return OpuzPacketStatus::ReadError;
            }
            let n = usize::from((packet_header >> 15) != 0);
            let packet_end = packet_offset.saturating_add(2).saturating_add(size);
            if packet_end > mp.opuz_data.len() {
                return OpuzPacketStatus::ReadError;
            }

            let mut initial_file_data = [0; 8];
            let initial_len = (2 + size).min(initial_file_data.len());
            initial_file_data[..initial_len]
                .copy_from_slice(&mp.opuz_data[packet_offset..packet_offset + initial_len]);
            let initial_file_data = u64::from_le_bytes(initial_file_data);
            if mp.state == MSU_STATE_RESUMING {
                mp.state = MSU_STATE_PLAYING;
                if mp.resume_info.initial_packet_bytes != initial_file_data {
                    return OpuzPacketStatus::ReadError;
                }
            }
            mp.resume_info.initial_packet_bytes = initial_file_data;
            mp.resume_info.samples_until_repeat = mp.samples_until_repeat.wrapping_add(mp.preskip);
            mp.resume_info.offset = mp.cur_file_offs;
            mp.cur_file_offs = mp.cur_file_offs.wrapping_add(2 + size as u32);

            let mut packet = Vec::with_capacity(size + n);
            if n != 0 {
                packet.push(0xfc);
            }
            packet.extend_from_slice(&mp.opuz_data[packet_offset + 2..packet_end]);

            let Some(decoder) = &mut mp.opus_decoder else {
                return OpuzPacketStatus::ReadError;
            };
            let Ok(r) = decoder.decode(&packet, &mut mp.buffer, false) else {
                return OpuzPacketStatus::ReadError;
            };
            if r == 0 {
                return OpuzPacketStatus::ReadError;
            }
            let r = r as u32;
            if r > mp.preskip {
                return OpuzPacketStatus::Decoded(r);
            }
            mp.preskip = mp.preskip.wrapping_sub(r);
        }
    }

    fn mix_to_buffer_with_volume(dst: &mut [i16], src: &[i16], n: usize, volume: f32) {
        for i in 0..n {
            let left = i * 2;
            let right = left + 1;
            if right >= dst.len() || right >= src.len() {
                break;
            }
            if volume == 1.0 {
                dst[left] = dst[left].wrapping_add(src[left]);
                dst[right] = dst[right].wrapping_add(src[right]);
            } else {
                let vol = (65536.0 * volume) as i32;
                dst[left] = dst[left].wrapping_add(((src[left] as i32 * vol) >> 16) as i16);
                dst[right] = dst[right].wrapping_add(((src[right] as i32 * vol) >> 16) as i16);
            }
        }
    }

    fn mix_to_buffer_with_volume_ramp(
        dst: &mut [i16],
        src: &[i16],
        n: usize,
        volume: f32,
        volume_step: f32,
        _ideal_target: f32,
    ) {
        let mut vol = (volume * 281474976710656.0) as i64;
        let step = (volume_step * 281474976710656.0) as i64;
        for i in 0..n {
            let left = i * 2;
            let right = left + 1;
            if right >= dst.len() || right >= src.len() {
                break;
            }
            let v = (vol >> 32) as i32;
            dst[left] = dst[left].wrapping_add(((src[left] as i32 * v) >> 16) as i16);
            dst[right] = dst[right].wrapping_add(((src[right] as i32 * v) >> 16) as i16);
            vol = vol.wrapping_add(step);
        }
    }

    fn mix_to_buffer(mp: &mut MsuPlayer, dst: &mut [i16], src: &[i16], mut n: u32) {
        if mp.volume != mp.volume_target {
            let step = if mp.volume < mp.volume_target {
                mp.volume_step
            } else {
                -mp.volume_step
            };
            let mut new_vol = mp.volume + step * n as f32;
            let mut curn = n;
            if if step >= 0.0 {
                new_vol >= mp.volume_target
            } else {
                new_vol < mp.volume_target
            } {
                let maxn = ((mp.volume_target - mp.volume) / step) as u32;
                curn = maxn.min(curn);
                new_vol = mp.volume_target;
            }
            let vol = mp.volume;
            mp.volume = new_vol;
            Self::mix_to_buffer_with_volume_ramp(dst, src, curn as usize, vol, step, new_vol);
            let skip = curn as usize * 2;
            if skip >= dst.len() || skip >= src.len() {
                return;
            }
            n -= curn;
            Self::mix_to_buffer_with_volume(&mut dst[skip..], &src[skip..], n as usize, mp.volume);
        } else {
            Self::mix_to_buffer_with_volume(dst, src, n as usize, mp.volume);
        }
    }

    pub fn msu_player_mix(&mut self, audio_buffer: &mut [i16], mut audio_samples: i32) {
        let mut audio_offset = 0usize;
        while audio_samples != 0 {
            let remaining = self
                .audio
                .msu_player
                .buffer_size
                .saturating_sub(self.audio.msu_player.buffer_pos);
            if remaining == 0 {
                if self.audio.msu_player.has_opus {
                    let orig_track = self.audio.msu_player.resume_info.orig_track;
                    match Self::msu_player_prepare_opuz_packet(&mut self.audio.msu_player) {
                        OpuzPacketStatus::FinishedPlaying => {
                            self.audio.msu_player.state = MSU_STATE_FINISHED_PLAYING;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                        }
                        OpuzPacketStatus::Decoded(r) => {
                            let n = r
                                .saturating_sub(self.audio.msu_player.preskip)
                                .min(self.audio.msu_player.samples_until_repeat);
                            self.audio.msu_player.samples_until_repeat =
                                self.audio.msu_player.samples_until_repeat.wrapping_sub(n);
                            self.audio.msu_player.buffer_pos = self.audio.msu_player.preskip;
                            self.audio.msu_player.buffer_size =
                                self.audio.msu_player.buffer_pos + n;
                            self.audio.msu_player.preskip = 0;
                        }
                        OpuzPacketStatus::ReadError => {
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            self.zelda_apu_write(0x2140, orig_track);
                            return;
                        }
                    }
                } else if self.audio.msu_player.has_file {
                    if self.audio.msu_player.samples_until_repeat == 0 {
                        let actual_track = self.audio.msu_player.resume_info.actual_track as usize;
                        if actual_track < MSU_TRACK_REPEATS.len()
                            && MSU_TRACK_REPEATS[actual_track] == 0
                        {
                            self.audio.msu_player.state = MSU_STATE_FINISHED_PLAYING;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            return;
                        }
                        self.audio.msu_player.samples_until_repeat = self
                            .audio
                            .msu_player
                            .total_samples_in_file
                            .wrapping_sub(self.audio.msu_player.repeat_position);
                        if self.audio.msu_player.samples_until_repeat == 0 {
                            let orig_track = self.audio.msu_player.resume_info.orig_track;
                            Self::msu_player_close_file(&mut self.audio.msu_player);
                            self.zelda_apu_write(0x2140, orig_track);
                            return;
                        }
                        self.audio.msu_player.cur_file_offs = self.audio.msu_player.repeat_position;
                    }

                    let r = 960u32.min(self.audio.msu_player.samples_until_repeat);
                    let data_start = self.audio.msu_player.cur_file_offs as usize * 2;
                    let data_end = data_start.saturating_add(r as usize * 2);
                    if data_end > self.audio.msu_player.pcm_data.len() {
                        let orig_track = self.audio.msu_player.resume_info.orig_track;
                        Self::msu_player_close_file(&mut self.audio.msu_player);
                        self.zelda_apu_write(0x2140, orig_track);
                        return;
                    }
                    let source = self.audio.msu_player.pcm_data[data_start..data_end].to_vec();
                    self.audio.msu_player.buffer[..source.len()].copy_from_slice(&source);
                    self.audio.msu_player.resume_info.offset = self.audio.msu_player.cur_file_offs;
                    self.audio.msu_player.cur_file_offs =
                        self.audio.msu_player.cur_file_offs.wrapping_add(r);
                    let n = r
                        .saturating_sub(self.audio.msu_player.preskip)
                        .min(self.audio.msu_player.samples_until_repeat);
                    self.audio.msu_player.samples_until_repeat =
                        self.audio.msu_player.samples_until_repeat.wrapping_sub(n);
                    self.audio.msu_player.buffer_pos = self.audio.msu_player.preskip;
                    self.audio.msu_player.buffer_size = self.audio.msu_player.buffer_pos + n;
                    self.audio.msu_player.preskip = 0;
                } else if self.audio.msu_player.state == MSU_STATE_FINISHED_PLAYING {
                    Self::msu_player_close_file(&mut self.audio.msu_player);
                    return;
                } else {
                    break;
                }
            }
            let remaining = self
                .audio
                .msu_player
                .buffer_size
                .saturating_sub(self.audio.msu_player.buffer_pos);
            let nr = (audio_samples as u32).min(remaining);
            let buffer_pos = self.audio.msu_player.buffer_pos as usize * 2;
            let end = buffer_pos + nr as usize * 2;
            let source = self.audio.msu_player.buffer[buffer_pos..end].to_vec();
            self.audio.msu_player.buffer_pos += nr;
            let dst_end = audio_offset.saturating_add(nr as usize * 2);
            if dst_end > audio_buffer.len() {
                return;
            }
            Self::mix_to_buffer(
                &mut self.audio.msu_player,
                &mut audio_buffer[audio_offset..],
                &source,
                nr,
            );
            audio_samples -= nr as i32;
            audio_offset = dst_end;
        }
    }

}
