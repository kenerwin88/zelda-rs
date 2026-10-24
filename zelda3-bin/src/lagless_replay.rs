//! Native input movies: no cartridge RNG, APU answers, or host-frame timing.
use std::{fs::{self, File}, io::{BufRead, BufReader, BufWriter, Read, Write}, path::Path};
use serde::{Deserialize, Serialize};
use zelda3::ZeldaState;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAGIC: &[u8; 8] = b"Z3LGIN01";
const SRAM_LEN: usize = 8192;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    audio_freq: u32,
    msuvolume: u8,
    resume_msu: bool,
    msu_path: Option<String>,
    language: Option<String>,
    extra_left_right: u8,
}

impl Launch {
    fn current() -> Result<Self> {
        let cfg = zelda3::config::parse_config_file_context(None).config;
        if cfg.enable_msu != 0 { return Err("native input movies require internal audio (MSU files are external inputs)".into()); }
        Ok(Self { audio_freq: if (11025..=48000).contains(&cfg.audio_freq) { u32::from(cfg.audio_freq) } else { 44100 },
            msuvolume: cfg.msuvolume, resume_msu: cfg.resume_msu, msu_path: cfg.msu_path,
            language: cfg.language, extra_left_right: cfg.extended_aspect_ratio.min(snes::consts::PPU_EXTRA_LEFT_RIGHT as u8) })
    }
}

pub(crate) struct MovieWriter(BufWriter<File>);

impl MovieWriter {
    pub(crate) fn create(path: &Path, game: &ZeldaState, samples: usize, channels: usize) -> Result<Self> {
        if !(1..=65536).contains(&samples) || !(1..=32).contains(&channels) {
            return Err("invalid lagless audio window".into());
        }
        let launch = Launch::current()?;
        let fresh = initial_game(&game.sram, &launch)?;
        if !game.lag_free() || bincode::serialize(&fresh)? != bincode::serialize(game)? {
            return Err("native recording requires a fresh lagless boot (no replay/warp state)".into());
        }
        let mut writer = BufWriter::new(File::create(path)?);
        writer.write_all(MAGIC)?;
        for value in [super::fnv1a64(super::EMBEDDED_ASSETS), super::fnv1a64(&game.sram)] {
            writer.write_all(&value.to_le_bytes())?;
        }
        writer.write_all(&u32::try_from(samples)?.to_le_bytes())?;
        writer.write_all(&u32::try_from(channels)?.to_le_bytes())?;
        let settings = serde_json::to_vec(&launch)?;
        writer.write_all(&(settings.len() as u32).to_le_bytes())?;
        writer.write_all(&settings)?;
        writer.write_all(&game.sram)?;
        writer.flush()?;
        Ok(Self(writer))
    }

    pub(crate) fn record(&mut self, input: u16) -> Result<()> {
        self.0.write_all(&input.to_le_bytes())?;
        // Each complete word survives a later gameplay panic.
        self.0.flush()?;
        Ok(())
    }
}

fn initial_game(sram: &[u8], launch: &Launch) -> Result<ZeldaState> {
    let mut game = ZeldaState::new();
    game.set_assets(super::EMBEDDED_ASSETS)?;
    game.ppu.extra_left_right = launch.extra_left_right;
    game.zelda_configure_audio(launch.audio_freq, launch.msuvolume, launch.resume_msu, launch.msu_path.clone());
    game.zelda_enable_msu(0);
    game.zelda_set_language(launch.language.as_deref());
    game.set_lag_free(true);
    game.sram.copy_from_slice(sram);
    Ok(game)
}

struct Movie {
    sram: Vec<u8>,
    inputs: Vec<u16>,
    samples: usize,
    channels: usize,
    hash: u64,
    launch: Launch,
}

impl Movie {
    fn read(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let mut reader = bytes.as_slice();
        let mut magic = [0; 8];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC { return Err("unsupported lagless movie version".into()); }
        let mut word = [0; 8];
        reader.read_exact(&mut word)?;
        if u64::from_le_bytes(word) != super::fnv1a64(super::EMBEDDED_ASSETS) {
            return Err("lagless movie asset pack mismatch".into());
        }
        reader.read_exact(&mut word)?;
        let sram_hash = u64::from_le_bytes(word);
        let mut int = [0; 4];
        reader.read_exact(&mut int)?;
        let samples = u32::from_le_bytes(int) as usize;
        reader.read_exact(&mut int)?;
        let channels = u32::from_le_bytes(int) as usize;
        if !(1..=65536).contains(&samples) || !(1..=32).contains(&channels) {
            return Err("invalid lagless audio window".into());
        }
        reader.read_exact(&mut int)?;
        let settings_len = u32::from_le_bytes(int) as usize;
        if settings_len > 65536 || settings_len > reader.len() { return Err("invalid lagless settings length".into()); }
        let launch: Launch = serde_json::from_slice(&reader[..settings_len])?;
        if !(11025..=48000).contains(&launch.audio_freq) || launch.extra_left_right as usize > snes::consts::PPU_EXTRA_LEFT_RIGHT {
            return Err("invalid lagless launch settings".into());
        }
        reader = &reader[settings_len..];
        let mut sram = vec![0; SRAM_LEN];
        reader.read_exact(&mut sram)?;
        if super::fnv1a64(&sram) != sram_hash { return Err("lagless SRAM hash mismatch".into()); }
        if reader.len() % 2 != 0 { return Err("truncated lagless input word".into()); }
        let inputs = reader.chunks_exact(2).map(|w| u16::from_le_bytes([w[0], w[1]])).collect();
        Ok(Self { sram, inputs, samples, channels, hash: super::fnv1a64(&bytes), launch })
    }
}

#[derive(Deserialize)]
struct Iteration { index: usize, joypad: Option<u16> }

fn convert(args: &[String]) -> Result<()> {
    if args.len() != 3 { return Err("usage: --convert-lagless LEDGER_DIR INITIAL_SRAM OUTPUT".into()); }
    let sram = fs::read(&args[1])?;
    if sram.len() != SRAM_LEN { return Err("expected 8192-byte SRAM".into()); }
    let game = initial_game(&sram, &Launch::current()?)?;
    let mut output = MovieWriter::create(Path::new(&args[2]), &game, 735, 2)?;
    let source = BufReader::new(File::open(Path::new(&args[0]).join("iterations.jsonl"))?);
    let mut count = 0;
    for (index, line) in source.lines().enumerate() {
        let iteration: Iteration = serde_json::from_str(&line?)?;
        if iteration.index != index { return Err("noncontiguous logical ledger".into()); }
        // Main precedes pad sampling: input for frame k is iteration k+1's pad.
        if index != 0 {
            output.record(iteration.joypad.ok_or("ledger iteration has no joypad")?.reverse_bits())?;
            count += 1;
        }
    }
    if count == 0 { return Err("logical ledger needs at least two iterations".into()); }
    // Match logical_replay's zero input after the last available iteration.
    output.record(0)?;
    println!("converted {} iterations", count + 1);
    Ok(())
}

/// A portable save uses deterministic prefix reconstruction. ZeldaState's
/// bincode snapshots omit lagless continuations; do not claim they are saves.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplaySave { version: u32, movie_hash: u64, next_iteration: usize, state_hash_v2: u64 }

/// Adapt an already-rendered movie frame to the playback device without
/// advancing the game's audio clock a second time.
fn device_audio(source: &[i16], channels: usize, target: &mut [i16], target_channels: usize) {
    let frames = source.len() / channels;
    let target_frames = target.len() / target_channels;
    let denominator = target_frames.saturating_sub(1).max(1);
    for frame in 0..target_frames {
        let position = frame * frames.saturating_sub(1);
        let lo = position / denominator;
        let hi = (lo + 1).min(frames - 1);
        let fraction = (position % denominator) as i64;
        for channel in 0..target_channels {
            let sample = |at: usize| -> i64 {
                if target_channels == 1 {
                    source[at * channels..(at + 1) * channels].iter()
                        .map(|&sample| i64::from(sample)).sum::<i64>() / channels as i64
                } else {
                    i64::from(source[at * channels + channel.min(channels - 1)])
                }
            };
            let a = sample(lo);
            let b = sample(hi);
            target[frame * target_channels + channel] =
                (a + (b - a) * fraction / denominator as i64) as i16;
        }
    }
}

fn replay(args: &[String]) -> Result<()> {
    let path = args.first().ok_or("usage: --replay-lagless FILE [--renderless] [--frames N] [--state-hash-every N] [--save-state FILE] [--load-state FILE]")?;
    let movie = Movie::read(Path::new(path))?;
    let mut frames = movie.inputs.len();
    let mut every = 1;
    let mut save = None;
    let mut load = None;
    let mut save_at = Vec::new();
    let mut renderless = false;
    let mut index = 1;
    while index < args.len() {
        let flag = args[index].as_str();
        if flag == "--renderless" { renderless = true; index += 1; continue; }
        let value = args.get(index + 1).ok_or("missing replay option value")?;
        match flag {
            "--frames" => frames = value.parse()?,
            "--state-hash-every" => every = value.parse::<usize>()?,
            "--save-state" => save = Some(value),
            "--load-state" => load = Some(value),
            "--save-state-at" => {
                let (iteration, path) = value.split_once(':').ok_or("expected ITERATION:PATH")?;
                save_at.push((iteration.parse::<usize>()?, path));
            }
            _ => return Err(format!("unknown lagless replay option: {flag}").into()),
        }
        index += 2;
    }
    if frames > movie.inputs.len() || every == 0 { return Err("replay length exceeds movie or hash interval is zero".into()); }
    if save_at.iter().any(|(at, _)| *at == 0 || *at > frames) { return Err("save iteration must be in 1..=frames".into()); }
    let resume: Option<ReplaySave> = load.map(|path| -> Result<_> { Ok(serde_json::from_slice(&fs::read(path)?)?) }).transpose()?;
    if let Some(resume) = &resume {
        if resume.version != 1 || resume.movie_hash != movie.hash || resume.next_iteration > frames {
            return Err("incompatible lagless replay save".into());
        }
    }
    let start = resume.as_ref().map_or(0, |save| save.next_iteration);
    // The game may save during replay. Keep those writes out of the player's
    // saves and out of the read-only movie/ledger directories.
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
    let scratch = Scratch(std::env::temp_dir().join(format!("zelda3-lagless-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos())));
    fs::create_dir(&scratch.0)?;
    // CLI entry executes before frontend/audio threads are created.
    unsafe { std::env::set_var("ZELDA3_SAVE_DIR", &scratch.0); }
    let mut game = initial_game(&movie.sram, &movie.launch)?;
    let mut audio = vec![0; movie.samples * movie.channels];
    let mut renderer = if renderless { None } else {
        Some(super::play_renderer::configured_from_env(256, 224,
            platform::NativeFrontendOptions::from_env(3, true))?)
    };
    let mut playback_audio = renderer.as_ref().map(|renderer| {
        vec![0; renderer.audio_samples_per_frame() * renderer.audio_channels()]
    }).unwrap_or_default();
    for iteration in 0..frames {
        if iteration == start {
            if let Some(resume) = &resume {
                if game.canonical_state_hash() != resume.state_hash_v2 { return Err("reconstructed replay save hash mismatch".into()); }
            }
        }
        game.zelda_run_frame(i32::from(movie.inputs[iteration]));
        game.zelda_render_audio(&mut audio, movie.samples as i32, movie.channels as i32);
        game.zelda_discard_unused_audio_frames();
        if iteration >= start {
            if let Some(renderer) = renderer.as_mut() {
                renderer.poll_input();
                if renderer.audio_samples_per_frame() == movie.samples && renderer.audio_channels() == movie.channels {
                    renderer.push_audio(&audio);
                } else {
                    device_audio(&audio, movie.channels, &mut playback_audio, renderer.audio_channels());
                    renderer.push_audio(&playback_audio);
                }
                renderer.present_frame(&mut game);
                if renderer.quit_requested() { return Err("replay interrupted".into()); }
            } else {
                // Match production's frame publication, without creating a GPU.
                super::gpu_capture::capture_gpu_frame_from_game(&mut game);
            }
            if iteration % every == 0 || iteration + 1 == frames {
                println!("{{\"iteration\":{iteration},\"state_v2\":\"{:016x}\"}}", game.canonical_state_hash());
            }
        } else {
            super::gpu_capture::capture_gpu_frame_from_game(&mut game);
        }
        for (_, path) in save_at.iter().filter(|(at, _)| *at == iteration + 1) {
            fs::write(path, serde_json::to_vec_pretty(&ReplaySave { version: 1, movie_hash: movie.hash,
                next_iteration: iteration + 1, state_hash_v2: game.canonical_state_hash() })?)?;
        }
    }
    if start == frames {
        if let Some(resume) = &resume {
            if game.canonical_state_hash() != resume.state_hash_v2 { return Err("reconstructed replay save hash mismatch".into()); }
        }
    }
    if let Some(path) = save {
        fs::write(path, serde_json::to_vec_pretty(&ReplaySave { version: 1, movie_hash: movie.hash,
            next_iteration: frames, state_hash_v2: game.canonical_state_hash() })?)?;
    }
    Ok(())
}

pub(crate) fn run(args: &[String], conversion: bool) {
    if let Err(error) = if conversion { convert(args) } else { replay(args) } {
        eprintln!("lagless replay: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_recording_replays_the_same_native_boot() {
        let dir = std::env::temp_dir().join(format!("lagless-live-roundtrip-{}", std::process::id()));
        let mut live = super::super::load_romless_play_state();
        live.set_lag_free(true);
        live.sram.fill(0);
        let mut recording = crate::live_input_recording::LiveInputRecording::start(&dir, &live, 735, 2).unwrap();
        let mut audio = vec![0; 735 * 2];
        let mut hashes = Vec::new();
        for frame in 0..1000 {
            let input = if frame % 120 < 8 { 0x80 } else { 0 };
            recording.record(frame, input).unwrap();
            live.zelda_run_frame(i32::from(input));
            live.zelda_render_audio(&mut audio, 735, 2);
            live.zelda_discard_unused_audio_frames();
            super::super::gpu_capture::capture_gpu_frame_from_game(&mut live);
            hashes.push(live.canonical_state_hash());
        }
        recording.finish().unwrap();
        let movie = Movie::read(&dir.join("input.z3lag")).unwrap();
        let mut replayed = initial_game(&movie.sram, &movie.launch).unwrap();
        for (frame, input) in movie.inputs.into_iter().enumerate() {
            replayed.zelda_run_frame(i32::from(input));
            replayed.zelda_render_audio(&mut audio, 735, 2);
            replayed.zelda_discard_unused_audio_frames();
            super::super::gpu_capture::capture_gpu_frame_from_game(&mut replayed);
            assert_eq!(replayed.canonical_state_hash(), hashes[frame], "iteration {frame}");
        }
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn playback_device_conversion_preserves_endpoints_and_channel_mix() {
        let mut stereo = [0; 6];
        device_audio(&[0, 100], 1, &mut stereo, 2);
        assert_eq!(stereo, [0, 0, 50, 50, 100, 100]);
        let mut mono = [0; 2];
        device_audio(&[100, -100, 200, 100], 2, &mut mono, 1);
        assert_eq!(mono, [0, 150]);
        device_audio(&[3, 6, 9, 6, 9, 12], 3, &mut mono, 1);
        assert_eq!(mono, [6, 9]);
    }

    #[test]
    fn conversion_uses_next_iterations_pad_and_zero_tail() {
        let dir = std::env::temp_dir().join(format!("lagless-convert-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("initial.srm"), [0; SRAM_LEN]).unwrap();
        fs::write(dir.join("iterations.jsonl"),
            "{\"index\":0,\"joypad\":0}\n{\"index\":1,\"joypad\":32768}\n{\"index\":2,\"joypad\":128}\n").unwrap();
        let output = dir.join("movie.z3lag");
        convert(&[dir.display().to_string(), dir.join("initial.srm").display().to_string(), output.display().to_string()]).unwrap();
        assert_eq!(Movie::read(&output).unwrap().inputs, [1, 256, 0]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn movie_roundtrip_and_validation() {
        let path = std::env::temp_dir().join(format!("lagless-movie-{}.bin", std::process::id()));
        let game = initial_game(&[0; SRAM_LEN], &Launch::current().unwrap()).unwrap();
        let mut writer = MovieWriter::create(&path, &game, 735, 2).unwrap();
        writer.record(0x8123).unwrap();
        drop(writer);
        let movie = Movie::read(&path).unwrap();
        assert_eq!(movie.inputs, [0x8123]);
        assert_eq!(movie.sram, game.sram.as_slice());
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(1);
        fs::write(&path, &bytes).unwrap();
        assert!(Movie::read(&path).is_err());
        bytes.pop(); *bytes.last_mut().unwrap() ^= 1;
        // Corrupt embedded SRAM, rather than the freely varying pad stream.
        let settings_len = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;
        bytes[36 + settings_len] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert!(Movie::read(&path).is_err());
        let mut writer = MovieWriter::create(&path, &game, 183, 6).unwrap();
        writer.record(0).unwrap();
        drop(writer);
        let low_rate = Movie::read(&path).unwrap();
        assert_eq!((low_rate.samples, low_rate.channels), (183, 6));
        fs::remove_file(path).unwrap();
    }
}
