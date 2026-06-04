#!/usr/bin/env python3
"""Unit tests for the MAME reference capture helper."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("capture_mame_reference.py")
SPEC = importlib.util.spec_from_file_location("capture_mame_reference", MODULE_PATH)
assert SPEC is not None
capture_mame_reference = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = capture_mame_reference
SPEC.loader.exec_module(capture_mame_reference)


class CapturePathTests(unittest.TestCase):
    def test_capture_paths_keep_outputs_under_requested_directory(self) -> None:
        paths = capture_mame_reference.capture_paths(Path("target/ref"), "sample")

        self.assertEqual(paths.raw_avi, Path("target/ref/raw/sample.avi"))
        self.assertEqual(paths.wav, Path("target/ref/sample.wav"))
        self.assertEqual(paths.mp4, Path("target/ref/sample.mp4"))
        self.assertEqual(paths.nvram_dir, Path("target/ref/nvram"))
        self.assertEqual(paths.trace_dir, Path("target/ref/traces"))


class NvramSeedTests(unittest.TestCase):
    def test_zeroed_high_score_cmos_defaults_preserve_non_score_cells(self) -> None:
        original = capture_mame_reference.source_cmos_default_nvram()
        zeroed = capture_mame_reference.zeroed_high_score_cmos_defaults()
        offset = capture_mame_reference.CMOS_ALL_TIME_HIGH_SCORE_CELL_OFFSET

        self.assertEqual(len(zeroed), len(original))
        self.assertEqual(zeroed[offset : offset + 6], bytes([0xF0] * 6))
        self.assertEqual(zeroed[offset + 6 : offset + 12], original[offset + 6 : offset + 12])
        self.assertEqual(zeroed[0:offset], original[0:offset])

    def test_zeroed_high_score_cmos_defaults_clear_every_all_time_score(self) -> None:
        zeroed = capture_mame_reference.zeroed_high_score_cmos_defaults()

        for entry in range(capture_mame_reference.CMOS_HIGH_SCORE_ENTRIES):
            offset = (
                capture_mame_reference.CMOS_ALL_TIME_HIGH_SCORE_CELL_OFFSET
                + entry * capture_mame_reference.CMOS_HIGH_SCORE_ENTRY_CELLS
            )
            self.assertEqual(zeroed[offset : offset + 6], bytes([0xF0] * 6))


class CommandConstructionTests(unittest.TestCase):
    def test_mame_command_uses_seeded_state_dirs_and_basename_aviwrite(self) -> None:
        paths = capture_mame_reference.capture_paths(Path("target/ref"), "sample")

        command = capture_mame_reference.build_mame_capture_command(
            "mame",
            Path("assets/roms"),
            paths,
            seconds=30,
            width=768,
            height=576,
            sample_rate=48_000,
            basename="sample",
        )

        self.assertIn("-rompath", command)
        self.assertIn("assets/roms", command)
        self.assertIn("-nvram_directory", command)
        self.assertIn("target/ref/nvram", command)
        self.assertIn("-snapsize", command)
        self.assertIn("768x576", command)
        self.assertIn("-aviwrite", command)
        self.assertIn("sample", command)
        self.assertIn("-wavwrite", command)
        self.assertIn("target/ref/sample.wav", command)
        self.assertIn("-seconds_to_run", command)
        self.assertNotIn("-autoboot_script", command)

    def test_scripted_mame_command_uses_autoboot_script_without_seconds(self) -> None:
        paths = capture_mame_reference.capture_paths(Path("target/ref"), "sample")
        scripted = capture_mame_reference.ScriptedCapture(
            inputs_path=Path("target/ref/traces/sample.inputs.txt"),
            output_path=Path("target/ref/traces/sample.expected.tsv"),
            debug_path=Path("target/ref/traces/sample.debug.tsv"),
            sound_dac_path=Path("target/ref/traces/sample.sound-dac.tsv"),
            schema_path=Path("assets/red-label/trace-schema.tsv"),
            frame_limit=42,
        )

        command = capture_mame_reference.build_mame_capture_command(
            "mame",
            Path("assets/roms"),
            paths,
            seconds=None,
            width=768,
            height=576,
            sample_rate=48_000,
            basename="sample",
            scripted=scripted,
        )
        env = capture_mame_reference.build_scripted_capture_env(scripted)

        self.assertIn("-autoboot_script", command)
        self.assertIn(str(capture_mame_reference.MAME_LUA), command)
        self.assertNotIn("-seconds_to_run", command)
        self.assertEqual(env["DEFENDER_TRACE_INPUTS"], "target/ref/traces/sample.inputs.txt")
        self.assertEqual(env["DEFENDER_TRACE_DEBUG"], "target/ref/traces/sample.debug.tsv")
        self.assertEqual(
            env["DEFENDER_TRACE_SOUND_DAC_OUTPUT"],
            "target/ref/traces/sample.sound-dac.tsv",
        )
        self.assertEqual(env["DEFENDER_TRACE_FRAMES"], "42")
        self.assertNotIn("DEFENDER_TRACE_SKIP_VIDEO_CRC", env)

    def test_trace_only_scripted_mame_command_skips_video_audio_and_media_outputs(self) -> None:
        paths = capture_mame_reference.capture_paths(Path("target/ref"), "sample")
        scripted = capture_mame_reference.ScriptedCapture(
            inputs_path=Path("target/ref/traces/sample.inputs.txt"),
            output_path=Path("target/ref/traces/sample.expected.tsv"),
            debug_path=Path("target/ref/traces/sample.debug.tsv"),
            sound_dac_path=Path("target/ref/traces/sample.sound-dac.tsv"),
            schema_path=Path("assets/red-label/trace-schema.tsv"),
            frame_limit=42,
        )

        command = capture_mame_reference.build_mame_capture_command(
            "mame",
            Path("assets/roms"),
            paths,
            seconds=None,
            width=768,
            height=576,
            sample_rate=48_000,
            basename="sample",
            scripted=scripted,
            trace_only=True,
        )
        env = capture_mame_reference.build_scripted_capture_env(scripted, trace_only=True)

        self.assertIn("-autoboot_script", command)
        self.assertIn("-video", command)
        self.assertIn("none", command)
        self.assertIn("-sound", command)
        self.assertNotIn("-aviwrite", command)
        self.assertNotIn("-wavwrite", command)
        self.assertNotIn("-snapsize", command)
        self.assertEqual(env["DEFENDER_TRACE_SKIP_VIDEO_CRC"], "1")

    def test_scripted_env_can_request_trace_state_steering(self) -> None:
        scripted = capture_mame_reference.ScriptedCapture(
            inputs_path=Path("target/ref/traces/sample.inputs.txt"),
            output_path=Path("target/ref/traces/sample.expected.tsv"),
            debug_path=Path("target/ref/traces/sample.debug.tsv"),
            sound_dac_path=Path("target/ref/traces/sample.sound-dac.tsv"),
            schema_path=Path("assets/red-label/trace-schema.tsv"),
            frame_limit=42,
        )

        env = capture_mame_reference.build_scripted_capture_env(
            scripted,
            state_steer="enemy_explosion_matrix",
            state_steer_frame=1234,
        )

        self.assertEqual(env["DEFENDER_TRACE_STEER"], "enemy_explosion_matrix")
        self.assertEqual(env["DEFENDER_TRACE_STEER_FRAME"], "1234")

    def test_count_expanded_input_frames_matches_trace_grammar(self) -> None:
        expanded = capture_mame_reference.expand_input_program("none*2;fire;reverse,thrust")

        self.assertEqual(capture_mame_reference.count_expanded_input_frames(expanded), 4)

    def test_ffmpeg_command_compresses_mame_raw_avi_to_mp4(self) -> None:
        paths = capture_mame_reference.capture_paths(Path("target/ref"), "sample")

        command = capture_mame_reference.build_ffmpeg_command("ffmpeg", paths)

        self.assertIn("target/ref/raw/sample.avi", command)
        self.assertIn("libx264", command)
        self.assertIn("target/ref/sample.mp4", command)


class OutputValidationTests(unittest.TestCase):
    def test_verify_media_outputs_accepts_non_empty_mp4_and_wav(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = capture_mame_reference.capture_paths(Path(directory), "sample")
            paths.mp4.write_bytes(b"mp4")
            paths.wav.write_bytes(b"wav")

            capture_mame_reference.verify_media_outputs(paths)

    def test_verify_media_outputs_rejects_missing_mp4(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = capture_mame_reference.capture_paths(Path(directory), "sample")
            paths.wav.write_bytes(b"wav")

            with self.assertRaises(SystemExit) as caught:
                capture_mame_reference.verify_media_outputs(paths)

            self.assertIn("MP4 output was not created", str(caught.exception))

    def test_verify_media_outputs_rejects_empty_wav(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = capture_mame_reference.capture_paths(Path(directory), "sample")
            paths.mp4.write_bytes(b"mp4")
            paths.wav.touch()

            with self.assertRaises(SystemExit) as caught:
                capture_mame_reference.verify_media_outputs(paths)

            self.assertIn("WAV output is empty", str(caught.exception))


if __name__ == "__main__":
    unittest.main()
