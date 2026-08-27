#!/usr/bin/env python3
"""Regression tests for parity provenance, caching, and timeline safety."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT_DIR = Path(__file__).resolve().parent


def load(name: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPT_DIR / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


evidence = load("parity_evidence")
microscope = load("parity_microscope")


class ParityMicroscopeTests(unittest.TestCase):
    def test_diagnostic_trace_path_follows_live_rng_trace_ownership(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            session = Path(directory)
            ordinary = session / "snes9x-trace.jsonl"
            live_rng = session / "oracle-rom-random.jsonl"

            self.assertEqual(microscope.diagnostic_trace_path(session), ordinary)
            self.assertEqual(
                microscope.diagnostic_trace_path(session, live_oracle_rng=True),
                live_rng,
            )

            live_rng.write_text("{}\n", encoding="utf-8")
            self.assertEqual(microscope.diagnostic_trace_path(session), ordinary)

            (session / "microscope-plan.json").write_text(
                json.dumps({"trace": {"artifact": live_rng.name}}),
                encoding="utf-8",
            )
            self.assertEqual(microscope.diagnostic_trace_path(session), live_rng)

            ordinary.write_text("{}\n", encoding="utf-8")
            self.assertEqual(microscope.diagnostic_trace_path(session), live_rng)

    def write_session(self, root: Path, name: str, *, frames: int = 100) -> Path:
        session = root / name
        session.mkdir()
        for artifact, contents in (
            ("input.txt", b"0 0x0000\n"),
            ("rom-random.txt", b"9 0xa5 carry=1\n"),
            ("initial.srm", b"sram"),
            ("oracle_initial.state", b"initial"),
            ("oracle_last_before.state", b"before"),
            ("oracle_final.state", b"final"),
        ):
            (session / artifact).write_bytes(contents)
        rng_sha = evidence.sha256_file(session / "rom-random.txt")
        (session / "manifest.json").write_text(
            json.dumps(
                {
                    "schema": 1,
                    "core": {"sha256": "c" * 64},
                    "rom": {"sha256": "r" * 64},
                    "rom_random_replay": {"sha256": rng_sha},
                    "timing": {
                        "frames_requested": frames,
                        "start_frame": 0,
                        "compare_from_frame": 0,
                    },
                    "comparison_lanes": {"video": True, "audio": True},
                }
            ),
            encoding="utf-8",
        )
        (session / "result.json").write_text(
            json.dumps(
                {
                    "status": "passed",
                    "parity_eligible": True,
                    "frames_completed": frames,
                    "video": {"matched": True, "first_mismatch": None},
                    "audio": {"matched": True, "first_mismatch": None},
                }
            ),
            encoding="utf-8",
        )
        (session / "frame_receipts.jsonl").write_text(
            json.dumps(
                {
                    "frame": 0,
                    "input": "0x0000",
                    "oracle_engine": {"main_module": 7},
                    "rust_engine": {"main_module": 7},
                    "oracle_audio_sample_frames": 533,
                    "oracle_video_width": 256,
                    "oracle_video_height": 224,
                    "vram": {"oracle_words": 32768, "rust_words": 32768},
                }
            )
            + "\n",
            encoding="utf-8",
        )
        (session / "av_hashes.jsonl").write_text(
            json.dumps(
                {
                    "schema": 1,
                    "frame": 0,
                    "input": "0x0000",
                    "video": {
                        "rust": {"width": 256, "height": 224, "sha256": "rust-video"},
                        "oracle": {"width": 256, "height": 224, "sha256": "oracle-video"},
                    },
                    "audio": {
                        "rust": {"sample_frames": 533, "channels": 2, "sha256": "rust-audio"},
                        "oracle": {
                            "sample_frames": 533,
                            "channels": 2,
                            "sha256": "oracle-audio",
                        },
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )
        return session

    def test_pc_filters_expand_lorom_mirrors_and_symbols(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            symbols = Path(directory) / "names.txt"
            symbols.write_text("0x86eb43: RestoreSprite\n", encoding="utf-8")
            table = microscope.SymbolTable(symbols)

            self.assertEqual(table.trace_filter(["06:eb43"]), "06:eb43,86:eb43")
            self.assertEqual(table.trace_filter(["RestoreSprite"]), "06:eb43,86:eb43")
            self.assertEqual(table.describe(0x06EB43), "RestoreSprite")

    def test_unfiltered_high_volume_trace_domains_are_rejected(self) -> None:
        with self.assertRaisesRegex(SystemExit, "unfiltered PC"):
            microscope.parse_events("frame,pc", has_pc=False, has_wram=False)
        with self.assertRaisesRegex(SystemExit, "unfiltered WRAM"):
            microscope.parse_events("frame,wram", has_pc=False, has_wram=False)

    def test_internal_trace_window_must_be_ordered(self) -> None:
        self.assertEqual(microscope.parse_internal_frame_range("31280-31290"), "31280-31290")
        with self.assertRaisesRegex(Exception, "ordered FIRST-LAST"):
            microscope.parse_internal_frame_range("31290-31280")

    def test_rust_evidence_subcommands_preserve_explicit_coordinates(self) -> None:
        trace_query = microscope.parser().parse_args(
            [
                "trace-query",
                "/tmp/session",
                "--host-frame",
                "31285",
                "--internal-frame",
                "87",
                "--event",
                "wram",
            ]
        )
        self.assertEqual(trace_query.host_frame, 31285)
        self.assertEqual(trace_query.internal_frame, 87)
        self.assertEqual(trace_query.event, ["wram"])
        cache_verify = microscope.parser().parse_args(
            ["cache-verify", "--cache-root", "/tmp/oracle-cache"]
        )
        self.assertEqual(cache_verify.cache_root, Path("/tmp/oracle-cache"))
        receipt_compare = microscope.parser().parse_args(
            ["receipt-compare", "/tmp/session", "--allow-incomplete"]
        )
        self.assertEqual(receipt_compare.session, Path("/tmp/session"))
        self.assertTrue(receipt_compare.allow_incomplete)
        av_compare = microscope.parser().parse_args(
            ["av-compare", "/tmp/session", "--max-differing-frames", "4"]
        )
        self.assertEqual(av_compare.session, Path("/tmp/session"))
        self.assertEqual(av_compare.max_differing_frames, 4)
        cached_av = microscope.parser().parse_args(
            [
                "cached-av",
                "/tmp/cache",
                "--rom",
                "/tmp/zelda3.sfc",
                "--resume-paired",
                "/tmp/paired",
                "--compare-from-frame",
                "49060",
                "--ignore-audio",
            ]
        )
        self.assertEqual(cached_av.cache, Path("/tmp/cache"))
        self.assertEqual(cached_av.rom, Path("/tmp/zelda3.sfc"))
        self.assertEqual(cached_av.resume_paired, Path("/tmp/paired"))
        self.assertEqual(cached_av.compare_from_frame, 49060)
        self.assertTrue(cached_av.ignore_audio)
        oracle_capture = microscope.parser().parse_args(
            [
                "oracle-av-capture",
                "/tmp/session",
                "--frames",
                "29505",
                "--resume-paired",
                "/tmp/paired",
            ]
        )
        self.assertEqual(oracle_capture.source_session, Path("/tmp/session"))
        self.assertEqual(oracle_capture.frames, 29505)
        self.assertEqual(oracle_capture.resume_paired, Path("/tmp/paired"))

    def test_timeline_uses_retro_run_not_internal_frame(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            symbols = root / "names.txt"
            symbols.write_text("0x868051: MainEntry\n", encoding="utf-8")
            trace = root / "trace.jsonl"
            records = [
                {"event": "frame", "stage": "entry", "run": 3, "frame": 900, "pc": 0x068051, "v": 2, "cycles": 4},
                {"event": "nmi", "run": 3, "frame": 901, "pc": 0x0080C9, "v": 225, "cycles": 8},
                {"event": "frame", "stage": "return", "run": 3, "frame": 901, "pc": 0x068051, "v": 1, "cycles": 2},
            ]
            trace.write_text("".join(json.dumps(record) + "\n" for record in records))

            report, rendered = microscope.build_timeline(
                trace, start_frame=29_010, symbols=microscope.SymbolTable(symbols)
            )

            self.assertEqual(report["runs"][0]["host_frame"], 29_013)
            self.assertEqual(report["runs"][0]["internal_frames"], [900, 901])
            self.assertEqual(report["incomplete_or_ambiguous_runs"], [])
            self.assertIn("host=29013 run=3 internal=900", rendered)
            self.assertIn("MainEntry", rendered)

    def test_timeline_marks_ambiguous_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace.jsonl"
            trace.write_text(
                json.dumps({"event": "nmi", "run": 0, "frame": 1, "pc": 0, "v": 0, "cycles": 0}) + "\n"
            )
            report, _ = microscope.build_timeline(
                trace, start_frame=0, symbols=microscope.SymbolTable(Path("missing"))
            )
            self.assertEqual(report["incomplete_or_ambiguous_runs"], [0])

    def test_explain_frame_identifies_an_exact_early_vram_generation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            failure = Path(directory) / "failure"
            failure.mkdir()
            (failure / "diff.json").write_text(
                json.dumps({"frame": 104}), encoding="utf-8"
            )
            before = b"abcd0000"
            after = b"abcd1111"
            (failure / "oracle_before_vram.bin").write_bytes(before)
            (failure / "oracle_after_vram.bin").write_bytes(after)
            (failure / "rust_visible_vram.bin").write_bytes(after)
            (failure / "rust_after_vram.bin").write_bytes(after)
            trace = Path(directory) / "trace.jsonl"
            events = [
                {"event": "frame", "stage": "entry", "run": 4, "frame": 4},
                {
                    "event": "dma",
                    "run": 4,
                    "frame": 4,
                    "pc": 0x008B67,
                    "v": 236,
                    "cycles": 1130,
                    "channel": 0,
                    "source": 0x7EAE80,
                    "bytes": 4,
                    "mode": 1,
                    "b_address": 0x18,
                    "vram_address": 2,
                },
                {"event": "frame", "stage": "return", "run": 4, "frame": 5},
            ]
            trace.write_text(
                "".join(json.dumps(event) + "\n" for event in events),
                encoding="utf-8",
            )

            report = microscope.build_frame_explanation(
                failure, trace=trace, resume_frame=100
            )

            self.assertEqual(report["classification"], "post-frame-vram-presented-early")
            self.assertEqual(report["exact_generation_matches"]["rust_live"], ["oracle_live"])
            self.assertEqual(report["exact_generation_matches"]["rust_visible"], ["oracle_live"])
            self.assertEqual(report["oracle_trace"]["run"], 4)
            self.assertTrue(
                report["oracle_trace"]["dma"][0][
                    "starts_at_first_visible_mismatch_word"
                ]
            )

    def test_timeline_accepts_only_the_leading_run_clipped_by_explicit_filter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            trace = Path(directory) / "trace.jsonl"
            records = [
                {"event": "frame", "stage": "return", "run": 74, "frame": 75, "pc": 0},
                {"event": "frame", "stage": "entry", "run": 75, "frame": 75, "pc": 0},
                {"event": "frame", "stage": "return", "run": 75, "frame": 76, "pc": 0},
            ]
            trace.write_text("".join(json.dumps(record) + "\n" for record in records))

            report, _ = microscope.build_timeline(
                trace,
                start_frame=31_200,
                symbols=microscope.SymbolTable(Path("missing")),
                internal_frame_filter="75-87",
            )

            self.assertEqual(report["incomplete_or_ambiguous_runs"], [])
            self.assertEqual(report["clipped_leading_runs"], [74])

    def test_cpu_checkpoint_correlation_maps_resumed_host_frame(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "session", frames=31_287)
            manifest = json.loads((session / "manifest.json").read_text())
            manifest["timing"]["start_frame"] = 31_200
            (session / "manifest.json").write_text(json.dumps(manifest))
            state = {
                "main": 7,
                "sub": 2,
                "subsub": 5,
                "frame_counter": 188,
                "room": 33,
                "lights_out": 1,
                "palette_countdown": 0,
                "palette_direction": 2,
                "link_y": 0x215a,
                "link_x": 0x0937,
                "bg2_v": 0x2110,
                "bg2_h": 0x0900,
            }
            (session / "rust-cpu-checkpoints.jsonl").write_text(
                json.dumps(
                    {
                        "schema": 2,
                        "event": "rust-cpu-checkpoint",
                        "coordinate": "absolute comparison host frame",
                        "host_frame": 31_286,
                        "pc": 0x00_8051,
                        "v": 244,
                        "cycles": 830,
                        **state,
                    }
                )
                + "\n"
            )
            oracle = {
                "event": "pc",
                "run": 86,
                "pc": 0x80_8051,
                "v": 244,
                "cycles": 842,
                **state,
            }
            (session / "snes9x-trace.jsonl").write_text(json.dumps(oracle) + "\n")

            report = microscope.cpu_checkpoint_correlation(session)

            self.assertEqual(report["status"], "compared")
            self.assertEqual(report["host_frames"], [31_286])
            self.assertEqual(report["oracle_minus_rust_master_cycles"], [12])

    def test_cpu_checkpoint_correlation_rejects_legacy_resumed_coordinate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "session", frames=31_287)
            manifest = json.loads((session / "manifest.json").read_text())
            manifest["timing"]["start_frame"] = 31_200
            (session / "manifest.json").write_text(json.dumps(manifest))
            (session / "rust-cpu-checkpoints.jsonl").write_text(
                json.dumps(
                    {
                        "schema": 1,
                        "event": "rust-cpu-checkpoint",
                        "coordinate": "zero-based libretro retro_run",
                        "run": 31_286,
                        "pc": 0x00_8051,
                    }
                )
                + "\n"
            )
            (session / "snes9x-trace.jsonl").write_text("")

            report = microscope.cpu_checkpoint_correlation(session)

            self.assertEqual(report["status"], "invalid")
            self.assertIn("falsely claims", report["problem"])

    def test_automatic_trace_tail_is_derived_from_bound_checkpoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory) / "full_run"
            project.mkdir()
            checkpoint = Path(directory) / "checkpoint"
            checkpoint.mkdir()
            generation = checkpoint / "frame-00031200"
            generation.mkdir()
            (generation / "manifest.json").write_text(json.dumps({"frame": 31_200}))
            (checkpoint / "latest.json").write_text(
                json.dumps({"frame": 31_200, "checkpoint": generation.name})
            )
            with mock.patch.object(
                microscope.parity_probe,
                "default_frontier_checkpoint_dir",
                return_value=checkpoint,
            ):
                selected = microscope.automatic_trace_frame_range(
                    project=project, frontier=31_287, tail_frames=12
                )

            self.assertEqual(selected, "75-87")

    def test_explicit_checkpoint_controls_trace_coordinates_and_probe_options(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            checkpoint = root / "selected-checkpoint"
            checkpoint.mkdir()
            generation = checkpoint / "frame-00020142"
            generation.mkdir()
            (generation / "manifest.json").write_text(
                json.dumps({"frame": 20_142}), encoding="utf-8"
            )
            (checkpoint / "latest.json").write_text(
                json.dumps({"frame": 20_142, "checkpoint": generation.name}),
                encoding="utf-8",
            )
            args = microscope.parser().parse_args(
                [
                    "microscope",
                    "--frontier",
                    "22456",
                    "--checkpoint-dir",
                    str(checkpoint),
                    "--checkpoint-frame",
                    "20142",
                    "--trust-cross-build-checkpoint",
                ]
            )

            selected, frame = microscope.resolve_microscope_checkpoint(
                args, project=root
            )

            self.assertEqual(selected, checkpoint.resolve())
            self.assertEqual(frame, 20_142)
            self.assertEqual(
                microscope.automatic_trace_frame_range(
                    project=root,
                    frontier=22_456,
                    tail_frames=12,
                    checkpoint_dir=selected,
                ),
                "2302-2314",
            )
            self.assertEqual(
                microscope.microscope_checkpoint_probe_args(
                    selected, frame, trust_cross_build=True
                ),
                [
                    "--checkpoint-frame",
                    "20142",
                    "--checkpoint-dir",
                    str(checkpoint.resolve()),
                    "--trust-cross-build-checkpoint",
                ],
            )

    def test_explicit_checkpoint_frame_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory)
            (checkpoint / "manifest.json").write_text(
                json.dumps({"frame": 20_142}), encoding="utf-8"
            )
            args = microscope.parser().parse_args(
                [
                    "microscope",
                    "--checkpoint-dir",
                    str(checkpoint),
                    "--checkpoint-frame",
                    "20143",
                ]
            )

            with self.assertRaisesRegex(SystemExit, "does not match"):
                microscope.resolve_microscope_checkpoint(args, project=checkpoint)

    def test_state_directory_is_rejected_instead_of_silently_running_cold(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            args = microscope.parser().parse_args(
                ["microscope", "--frontier", "22456", "--state", directory]
            )
            with self.assertRaisesRegex(SystemExit, "not a checkpoint directory"):
                microscope.command_microscope(args)

    def test_oracle_cache_is_content_addressed_and_tamper_evident(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "session")
            cache_root = root / "cache"

            cache, reused = evidence.cache_oracle_session(session, cache_root=cache_root)
            same_cache, reused_again = evidence.cache_oracle_session(session, cache_root=cache_root)

            self.assertFalse(reused)
            self.assertTrue(reused_again)
            self.assertEqual(cache, same_cache)
            extracted = json.loads((cache / "oracle-frame-receipts.jsonl").read_text())
            self.assertIn("oracle_engine", extracted)
            self.assertNotIn("rust_engine", extracted)
            av_extracted = json.loads((cache / "oracle-av-hashes.jsonl").read_text())
            self.assertEqual(av_extracted["video"]["sha256"], "oracle-video")
            self.assertEqual(av_extracted["audio"]["sha256"], "oracle-audio")
            self.assertNotIn("rust", av_extracted["video"])
            verified = evidence.verify_oracle_cache_root(cache_root)
            self.assertEqual(verified["entries"], 1)
            (cache / "oracle_final.state").write_bytes(b"corrupt")
            with self.assertRaisesRegex(SystemExit, "cache is corrupt"):
                evidence.verify_oracle_cache_root(cache_root)

    def test_oracle_cache_identity_includes_enabled_av_lanes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            both = self.write_session(root, "both")
            video_only = self.write_session(root, "video-only")
            manifest = json.loads((video_only / "manifest.json").read_text())
            manifest["comparison_lanes"]["audio"] = False
            (video_only / "manifest.json").write_text(json.dumps(manifest))
            record = json.loads((video_only / "av_hashes.jsonl").read_text())
            record["audio"] = None
            (video_only / "av_hashes.jsonl").write_text(json.dumps(record) + "\n")

            both_cache, _ = evidence.cache_oracle_session(both, cache_root=root / "cache")
            video_cache, _ = evidence.cache_oracle_session(
                video_only, cache_root=root / "cache"
            )

            self.assertNotEqual(both_cache, video_cache)

    def test_oracle_cache_identity_includes_initial_oracle_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = self.write_session(root, "first")
            second = self.write_session(root, "second")
            (second / "oracle_initial.state").write_bytes(b"different initial state")

            first_cache, _ = evidence.cache_oracle_session(first, cache_root=root / "cache")
            second_cache, _ = evidence.cache_oracle_session(second, cache_root=root / "cache")

            self.assertNotEqual(first_cache, second_cache)

    def test_oracle_cache_identity_includes_timing_host_receipt_schema(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = self.write_session(root, "receipt-v1")
            second = self.write_session(root, "receipt-v2")
            for session, schema in ((first, 1), (second, 2)):
                (session / "original-timing-host-receipts.jsonl.zst").write_bytes(
                    b"same receipt bytes"
                )
                manifest = json.loads((session / "manifest.json").read_text())
                manifest["original_timing_host_receipts"] = {
                    "schema": schema,
                    "artifact": "original-timing-host-receipts.jsonl.zst",
                }
                (session / "manifest.json").write_text(json.dumps(manifest))

            first_cache, _ = evidence.cache_oracle_session(first, cache_root=root / "cache")
            second_cache, _ = evidence.cache_oracle_session(second, cache_root=root / "cache")

            self.assertNotEqual(first_cache, second_cache)

    def test_live_rng_is_materialized_and_bound_to_the_diagnostic_checkpoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "bootstrap")
            (session / "oracle-rom-random.jsonl").write_text(
                json.dumps(
                    {
                        "event": "rng-write",
                        "run": 77,
                        "pc": 0x8DBA7F,
                        "address": 0x0FA1,
                        "value": 0xA5,
                        "carry": 1,
                    }
                )
                + "\n"
            )
            checkpoint = root / "checkpoint"
            checkpoint.mkdir()
            (checkpoint / microscope.parity_probe.IDENTITY_NAME).write_text(
                json.dumps({"rom_random_sha256": None})
            )
            with mock.patch.object(
                microscope.parity_probe,
                "default_frontier_checkpoint_dir",
                return_value=checkpoint,
            ):
                script, count = microscope.bind_live_rng_to_diagnostic_checkpoint(
                    session, root
                )
                cached = microscope.cached_diagnostic_rng(root)

            self.assertEqual(count, 1)
            self.assertIn("77 0xa5 carry=1", script.read_text())
            self.assertEqual(cached, script)
            identity = json.loads(
                (checkpoint / microscope.parity_probe.IDENTITY_NAME).read_text()
            )
            self.assertEqual(identity["rom_random_sha256"], evidence.sha256_file(script))

    def test_oracle_av_capture_materializes_only_manifest_bound_live_rng(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "source")
            binary = root / "zelda3"
            core = root / "snes9x.dylib"
            rom = root / "zelda3.sfc"
            for path in (binary, core, rom):
                path.write_bytes(path.name.encode())

            (session / "rom-random.txt").unlink()
            trace = session / "oracle-rom-random.jsonl"
            trace.write_text(
                json.dumps(
                    {
                        "event": "rng-write",
                        "run": 12,
                        "pc": 0x8DBA7F,
                        "address": 0x0FA1,
                        "value": 0x5A,
                        "carry": 0,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            manifest = json.loads((session / "manifest.json").read_text())
            manifest["core"]["path"] = str(core)
            manifest["rom"]["path"] = str(rom)
            manifest["rom_random_authority"] = {
                "mode": "live_oracle_trace",
                "artifact": trace.name,
            }
            (session / "manifest.json").write_text(json.dumps(manifest))

            captured_rng_path = None

            def inspect_capture(command: list[str], **_kwargs: object) -> mock.Mock:
                nonlocal captured_rng_path
                captured_rng_path = Path(command[6])
                self.assertTrue(captured_rng_path.is_file())
                self.assertEqual(
                    captured_rng_path.read_text().splitlines()[-1],
                    "12 0x5a carry=0",
                )
                return mock.Mock(returncode=0)

            args = microscope.parser().parse_args(
                [
                    "oracle-av-capture",
                    str(session),
                    "--frames",
                    "100",
                    "--binary",
                    str(binary),
                    "--output",
                    str(root / "capture"),
                    "--cache-root",
                    str(root / "cache"),
                ]
            )
            with mock.patch.object(
                microscope.subprocess, "run", side_effect=inspect_capture
            ), mock.patch.object(
                microscope.evidence,
                "cache_oracle_session",
                return_value=(root / "cache" / "entry", False),
            ):
                self.assertEqual(microscope.command_oracle_av_capture(args), 0)

            self.assertIsNotNone(captured_rng_path)
            self.assertFalse(captured_rng_path.is_file())

    def test_resumed_oracle_capture_uses_pair_core_and_route_artifact_authorities(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self.write_session(root, "source", frames=100)
            (source / "result.json").unlink()
            binary = root / "zelda3"
            selected_core = root / "paired-core.dylib"
            selected_rom = root / "zelda3.sfc"
            for path in (binary, selected_core, selected_rom):
                path.write_bytes(path.name.encode())
            pair = root / "paired"
            pair.mkdir()
            (pair / "oracle.state").write_bytes(b"paired oracle")
            (pair / "initial.srm").write_bytes((source / "initial.srm").read_bytes())
            paired_manifest = {
                "schema": 1,
                "boundary": "pre-frame",
                "frame": 50,
                "oracle_state": "oracle.state",
                "rust_state": "rust.z3state",
                "core": {"sha256": evidence.sha256_file(selected_core)},
                "rom": {"sha256": evidence.sha256_file(selected_rom)},
                "input_script": {"sha256": evidence.sha256_file(source / "input.txt")},
                "rom_random_script": {
                    "sha256": evidence.sha256_file(source / "rom-random.txt")
                },
                "initial_sram": {
                    "artifact": "initial.srm",
                    "sha256": evidence.sha256_file(source / "initial.srm")
                },
            }
            (pair / "manifest.json").write_text(json.dumps(paired_manifest))

            def inspect_capture(command: list[str], **_kwargs: object) -> mock.Mock:
                self.assertEqual(Path(command[2]), selected_core.resolve())
                self.assertEqual(Path(command[3]), selected_rom.resolve())
                self.assertEqual(command[9], paired_manifest["core"]["sha256"])
                self.assertEqual(command[10], paired_manifest["rom"]["sha256"])
                self.assertEqual(command[-4], "--resume-oracle-state")
                self.assertEqual(Path(command[-3]), (pair / "oracle.state").resolve())
                self.assertEqual(command[-2:], ["--start-frame", "50"])
                return mock.Mock(returncode=0)

            args = microscope.parser().parse_args(
                [
                    "oracle-av-capture",
                    str(source),
                    "--frames",
                    "100",
                    "--core",
                    str(selected_core),
                    "--rom",
                    str(selected_rom),
                    "--binary",
                    str(binary),
                    "--output",
                    str(root / "capture"),
                    "--cache-root",
                    str(root / "cache"),
                    "--resume-paired",
                    str(pair),
                ]
            )
            with mock.patch.object(
                microscope.subprocess, "run", side_effect=inspect_capture
            ), mock.patch.object(
                microscope.evidence,
                "cache_oracle_session",
                return_value=(root / "cache" / "entry", False),
            ):
                self.assertEqual(microscope.command_oracle_av_capture(args), 0)

    def test_reset_origin_oracle_capture_accepts_manifest_bounded_failed_rust_source(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self.write_session(root, "source", frames=64000)
            (source / "result.json").unlink()
            binary = root / "zelda3"
            core = root / "snes9x.dylib"
            rom = root / "zelda3.sfc"
            for path in (binary, core, rom):
                path.write_bytes(path.name.encode())
            manifest = json.loads((source / "manifest.json").read_text())
            manifest["core"] = {
                "path": str(core),
                "sha256": evidence.sha256_file(core),
            }
            manifest["rom"] = {
                "path": str(rom),
                "sha256": evidence.sha256_file(rom),
            }
            (source / "manifest.json").write_text(json.dumps(manifest))

            def inspect_capture(command: list[str], **_kwargs: object) -> mock.Mock:
                self.assertEqual(command[4], "64000")
                self.assertNotIn("--resume-oracle-state", command)
                self.assertEqual(command[9], manifest["core"]["sha256"])
                self.assertEqual(command[10], manifest["rom"]["sha256"])
                return mock.Mock(returncode=0)

            args = microscope.parser().parse_args(
                [
                    "oracle-av-capture",
                    str(source),
                    "--frames",
                    "64000",
                    "--binary",
                    str(binary),
                    "--output",
                    str(root / "capture"),
                    "--cache-root",
                    str(root / "cache"),
                ]
            )
            with mock.patch.object(
                microscope.subprocess, "run", side_effect=inspect_capture
            ), mock.patch.object(
                microscope.evidence,
                "cache_oracle_session",
                return_value=(root / "cache" / "entry", False),
            ):
                self.assertEqual(microscope.command_oracle_av_capture(args), 0)

    def test_microscope_rng_is_materialized_from_its_selected_source(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self.write_session(root, "source")
            (source / "rom-random.txt").unlink()
            trace = source / "oracle-rom-random.jsonl"
            trace.write_text(
                json.dumps(
                    {
                        "event": "rng-write",
                        "run": 91,
                        "pc": 0x8DBA7F,
                        "address": 0x0FA1,
                        "value": 0x37,
                        "carry": 1,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            manifest = json.loads((source / "manifest.json").read_text())
            manifest["rom_random_authority"] = {
                "mode": "live_oracle_trace",
                "artifact": trace.name,
            }
            (source / "manifest.json").write_text(json.dumps(manifest))

            destination = root / "diagnostic" / "source-rom-random.txt"
            resolved = microscope.materialize_source_session_rng(source, destination)

            self.assertIsNotNone(resolved)
            script, count = resolved
            self.assertEqual(script, destination)
            self.assertEqual(count, 1)
            self.assertEqual(script.read_text().splitlines()[-1], "91 0x37 carry=1")

    def test_cold_pass_rejects_resumed_or_disabled_lanes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            session = self.write_session(root, "session")
            binary = root / "zelda3"
            binary.write_bytes(b"binary")
            manifest = json.loads((session / "manifest.json").read_text())
            manifest["timing"]["start_frame"] = 50
            (session / "manifest.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(SystemExit, "resumed session"):
                evidence.record_cold_pass(
                    session=session,
                    route_signature={"route": "test"},
                    binary=binary,
                    output_root=root / "passes",
                )

    def test_promotion_requires_two_distinct_sessions_and_clean_commit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "zelda3"
            binary.write_bytes(b"binary")
            passes = root / "passes"
            route_signature = {"project": "routes/full_run", "core_sha256": "c" * 64}
            first_session = self.write_session(root, "first", frames=120)
            second_session = self.write_session(root, "second", frames=140)
            with mock.patch.object(
                evidence,
                "git_identity",
                return_value={
                    "head": "a" * 40,
                    "describe": "test",
                    "clean": True,
                    "status_sha256": "s",
                    "tracked_diff_sha256": "d",
                },
            ):
                evidence.record_cold_pass(
                    session=first_session,
                    route_signature=route_signature,
                    binary=binary,
                    output_root=passes,
                )
                evidence.record_cold_pass(
                    session=second_session,
                    route_signature=route_signature,
                    binary=binary,
                    output_root=passes,
                )
                ledger_path = root / "frontier.json"
                ledger = evidence.promote_frontier(
                    ledger_path=ledger_path, binary=binary, pass_root=passes
                )

            self.assertEqual(ledger["promoted"]["last_exact_video_frame"], 120)
            self.assertEqual(ledger["promoted"]["commit"], "a" * 40)
            self.assertEqual(len(ledger["promoted"]["cold_confirmation_receipts"]), 2)

    def test_promotion_refuses_dirty_tree(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "zelda3"
            binary.write_bytes(b"binary")
            with mock.patch.object(evidence, "git_identity", return_value={"clean": False}):
                with self.assertRaisesRegex(SystemExit, "clean committed tree"):
                    evidence.promote_frontier(
                        ledger_path=root / "frontier.json",
                        binary=binary,
                        pass_root=root / "passes",
                    )


if __name__ == "__main__":
    unittest.main()
