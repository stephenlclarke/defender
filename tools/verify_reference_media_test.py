#!/usr/bin/env python3
"""Unit tests for reference media verification helpers."""

from __future__ import annotations

import argparse
import importlib.util
import io
import json
import math
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("verify_reference_media.py")
SPEC = importlib.util.spec_from_file_location("verify_reference_media", MODULE_PATH)
assert SPEC is not None
verify_reference_media = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = verify_reference_media
SPEC.loader.exec_module(verify_reference_media)


class RegionParsingTests(unittest.TestCase):
    def test_parse_region_reads_named_rectangles(self) -> None:
        region = verify_reference_media.parse_region("laser:0,300,768,96")

        self.assertEqual(region.name, "laser")
        self.assertEqual((region.x, region.y, region.width, region.height), (0, 300, 768, 96))

    def test_parse_region_rejects_missing_size(self) -> None:
        with self.assertRaises(argparse.ArgumentTypeError):
            verify_reference_media.parse_region("laser:0,300,768")


class VisualMetricTests(unittest.TestCase):
    def test_visual_aggregate_tracks_rms_mae_and_peak_delta(self) -> None:
        aggregate = verify_reference_media.VisualAggregate("full")

        aggregate.add_diff(3)
        aggregate.add_diff(-4)

        self.assertEqual(aggregate.channel_count, 2)
        self.assertAlmostEqual(aggregate.rms, math.sqrt(12.5))
        self.assertEqual(aggregate.mae, 3.5)
        self.assertEqual(aggregate.max_abs, 4)

    def test_add_region_diffs_samples_configured_pixels(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=4,
            audio_window_ms=1,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        aggregate = verify_reference_media.VisualAggregate("full")

        verify_reference_media.add_region_diffs(
            aggregate,
            bytes([10, 20, 30, 40, 50, 60]),
            bytes([8, 21, 35, 44, 50, 55]),
            verify_reference_media.Region("full", 0, 0, 2, 1),
            config,
        )

        self.assertEqual(aggregate.channel_count, 6)
        self.assertEqual(aggregate.max_abs, 5)
        self.assertAlmostEqual(aggregate.mae, (2 + 1 + 5 + 4 + 0 + 5) / 6)


class AudioMetricTests(unittest.TestCase):
    def test_identical_audio_samples_pass_core_thresholds(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=8,
            audio_window_ms=250,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
        )
        samples = [0.0, 0.5, 0.0, -0.5, 0.0, 0.25, 0.0, -0.25]

        comparison = verify_reference_media.compare_audio_samples(
            samples,
            list(samples),
            config,
            thresholds,
        )

        self.assertEqual(comparison.status, "pass")
        self.assertEqual(comparison.compared_samples, len(samples))
        self.assertEqual(comparison.normalized_diff_rms, 0.0)
        self.assertGreater(comparison.correlation, 0.99)

    def test_missing_audio_samples_fail(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=8,
            audio_window_ms=250,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
        )

        comparison = verify_reference_media.compare_audio_samples([], [0.0], config, thresholds)

        self.assertEqual(comparison.status, "fail")
        self.assertIn("no shared audio samples", comparison.failures[0])

    def test_audio_comparison_ignores_constant_dc_offset(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=8,
            audio_window_ms=250,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
        )

        comparison = verify_reference_media.compare_audio_samples(
            [0.25, 0.25, 0.25, 0.25],
            [0.0, 0.0, 0.0, 0.0],
            config,
            thresholds,
        )

        self.assertEqual(comparison.status, "pass")
        self.assertAlmostEqual(comparison.reference_dc_offset, 0.25)
        self.assertEqual(comparison.normalized_diff_rms, 0.0)

    def test_audio_comparison_treats_tiny_ac_noise_as_silence(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=8,
            audio_window_ms=250,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
        )

        comparison = verify_reference_media.compare_audio_samples(
            [0.25, 0.25001, 0.24999, 0.25],
            [0.0, 0.0, 0.0, 0.0],
            config,
            thresholds,
        )

        self.assertEqual(comparison.status, "pass")
        self.assertEqual(comparison.normalized_diff_rms, 0.0)
        self.assertEqual(comparison.correlation, 1.0)

    def test_stochastic_noise_gate_accepts_matching_shape_without_waveform_phase(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=100,
            audio_window_ms=20,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
            audio_noise_zcr_min=0.05,
            audio_noise_zcr_ratio_min=0.49,
        )
        reference = [1.0, -1.0] * 100
        candidate = [1.0, 1.0, -1.0, -1.0] * 50

        comparison = verify_reference_media.compare_audio_samples(
            reference,
            candidate,
            config,
            thresholds,
        )

        self.assertEqual(comparison.status, "pass")
        self.assertTrue(comparison.noise_like_pass)
        self.assertLess(comparison.correlation, thresholds.audio_correlation)
        self.assertGreater(comparison.normalized_diff_rms, thresholds.audio_nrms)


class AcceptanceModeTests(unittest.TestCase):
    def visual(self, status: str) -> verify_reference_media.VisualComparison:
        return verify_reference_media.VisualComparison(
            status=status,
            reference_frames=1,
            candidate_frames=1,
            compared_frames=1,
            reference_signatures=[],
            candidate_signatures=[],
            full=verify_reference_media.VisualAggregate("full"),
            regions=[],
            first_failing_frame=None,
            failures=[] if status == "pass" else ["visual failed"],
        )

    def audio(self, status: str) -> verify_reference_media.AudioComparison:
        return verify_reference_media.AudioComparison(
            status=status,
            failures=[] if status == "pass" else ["audio failed"],
        )

    def test_all_acceptance_requires_visual_and_audio(self) -> None:
        self.assertEqual(
            verify_reference_media.overall_status_for_acceptance(
                self.visual("pass"),
                self.audio("fail"),
                "all",
            ),
            "fail",
        )

    def test_visual_acceptance_ignores_synthetic_audio_failures(self) -> None:
        self.assertEqual(
            verify_reference_media.overall_status_for_acceptance(
                self.visual("pass"),
                self.audio("fail"),
                "visual",
            ),
            "pass",
        )

    def test_audio_acceptance_ignores_synthetic_visual_failures(self) -> None:
        self.assertEqual(
            verify_reference_media.overall_status_for_acceptance(
                self.visual("fail"),
                self.audio("pass"),
                "audio",
            ),
            "pass",
        )

    def test_write_reports_records_acceptance_mode(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=2,
            height=1,
            visual_fps=1.0,
            max_frames=1,
            pixel_stride=1,
            audio_sample_rate=8,
            audio_window_ms=250,
            audio_start_ms=0.0,
            duration_ms=None,
        )
        thresholds = verify_reference_media.Thresholds(
            visual_rms=1.0,
            visual_mae=1.0,
            audio_nrms=0.01,
            audio_correlation=0.99,
            audio_envelope_correlation=0.99,
            audio_silence_rms=0.001,
        )
        with tempfile.TemporaryDirectory() as directory:
            out_dir = Path(directory)

            with redirect_stdout(io.StringIO()):
                status = verify_reference_media.write_reports(
                    out_dir,
                    config,
                    config,
                    config,
                    thresholds,
                    Path("target/reference-media/mame/reference.mp4"),
                    Path("target/reference-media/mame/reference.wav"),
                    Path("target/reference-media/clean/candidate.gif"),
                    Path("target/reference-media/clean/candidate.wav"),
                    self.visual("pass"),
                    self.audio("fail"),
                    "visual",
                )

            report = json.loads((out_dir / "report.json").read_text(encoding="utf-8"))

        self.assertEqual(status, "pass")
        self.assertEqual(report["acceptance_mode"], "visual")


class CommandConstructionTests(unittest.TestCase):
    def test_frame_command_uses_local_file_and_sampling_contract(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=320,
            height=240,
            visual_fps=3.0,
            max_frames=9,
            pixel_stride=2,
            audio_sample_rate=11_025,
            audio_window_ms=20,
            audio_start_ms=500.0,
            duration_ms=2_000.0,
        )

        command = verify_reference_media.build_frame_extract_command(
            "ffmpeg",
            Path("reference.mp4"),
            Path("out/frame-%05d.png"),
            config,
        )

        self.assertIn("-ss", command)
        self.assertIn("0.500000", command)
        self.assertIn("fps=3.0,scale=320:240:flags=neighbor", command)
        self.assertIn("-frames:v", command)
        self.assertIn("9", command)

    def test_audio_command_extracts_mono_float_samples(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=320,
            height=240,
            visual_fps=3.0,
            max_frames=9,
            pixel_stride=2,
            audio_sample_rate=11_025,
            audio_window_ms=20,
            audio_start_ms=0.0,
            duration_ms=None,
        )

        command = verify_reference_media.build_audio_extract_command(
            "ffmpeg",
            Path("reference.mp4"),
            Path("out/reference.f32"),
            config,
        )

        self.assertIn("-vn", command)
        self.assertIn("-ac", command)
        self.assertIn("1", command)
        self.assertIn("-ar", command)
        self.assertIn("11025", command)
        self.assertIn("f32le", command)


class StreamConfigTests(unittest.TestCase):
    def test_stream_configs_can_offset_reference_and_candidate_independently(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=320,
            height=240,
            visual_fps=3.0,
            max_frames=9,
            pixel_stride=2,
            audio_sample_rate=11_025,
            audio_window_ms=20,
            audio_start_ms=500.0,
            duration_ms=2_000.0,
        )

        reference_config, candidate_config = verify_reference_media.stream_configs(
            config,
            52_000.0,
            24_500.0,
        )

        self.assertEqual(reference_config.audio_start_ms, 52_000.0)
        self.assertEqual(candidate_config.audio_start_ms, 24_500.0)
        self.assertEqual(reference_config.width, config.width)
        self.assertEqual(candidate_config.duration_ms, config.duration_ms)

    def test_stream_configs_preserve_legacy_shared_offset_default(self) -> None:
        config = verify_reference_media.MediaConfig(
            width=320,
            height=240,
            visual_fps=3.0,
            max_frames=9,
            pixel_stride=2,
            audio_sample_rate=11_025,
            audio_window_ms=20,
            audio_start_ms=750.0,
            duration_ms=None,
        )

        reference_config, candidate_config = verify_reference_media.stream_configs(
            config,
            None,
            None,
        )

        self.assertEqual(reference_config.audio_start_ms, 750.0)
        self.assertEqual(candidate_config.audio_start_ms, 750.0)


if __name__ == "__main__":
    unittest.main()
