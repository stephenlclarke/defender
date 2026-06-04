#!/usr/bin/env python3
"""Unit tests for MAME reference-window scanning helpers."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("scan_reference_windows.py")
SPEC = importlib.util.spec_from_file_location("scan_reference_windows", MODULE_PATH)
assert SPEC is not None
scan_reference_windows = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = scan_reference_windows
SPEC.loader.exec_module(scan_reference_windows)


class ReferenceWindowScanTests(unittest.TestCase):
    def test_split_sound_commands_normalizes_supported_separators(self) -> None:
        commands = scan_reference_windows.split_sound_commands("0xfe, 0xFA; -;bad;F8")

        self.assertEqual(commands, ("0xFE", "0xFA", "0xF8"))

    def test_scan_sound_hits_filters_target_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            expected = Path(directory) / "sample.expected.tsv"
            expected.write_text(
                "frame\tsound_commands\n"
                "100\t0xEB\n"
                "101\t0xfe;0xFA\n",
                encoding="utf-8",
            )

            hits = scan_reference_windows.scan_sound_hits(expected, {"0xFE", "0xFA"})

        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0].frame, 101)
        self.assertEqual(hits[0].commands, ("0xFE", "0xFA"))

    def test_scan_object_hits_detects_picture_addresses_in_debug_fields(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            debug = Path(directory) / "sample.debug.tsv"
            debug.write_text(
                "frame\tactive_objects\texpanded_objects\n"
                "98\t-\t-\n"
                "99\t0xA23C:pic=0xF8CE:x=0x10:y=0x20\t-\n"
                "100\t-\tdesc=0xF8E2:center=0x1234\n",
                encoding="utf-8",
            )

            hits = scan_reference_windows.scan_object_hits(
                debug,
                {
                    "0xF8CE": "SCZP1 converted mutant",
                    "0xF8E2": "SWXP1 swarmer explosion",
                },
            )

        self.assertEqual([hit.frame for hit in hits], [99, 100])
        self.assertEqual(hits[0].label, "SCZP1 converted mutant")
        self.assertEqual(hits[1].label, "SWXP1 swarmer explosion")

    def test_scan_trace_tree_pairs_nearby_sound_and_object_hits(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = root / "sample.expected.tsv"
            debug = root / "sample.debug.tsv"
            expected.write_text(
                "frame\tsound_commands\n"
                "100\t0xFE\n"
                "140\t0xF3\n",
                encoding="utf-8",
            )
            debug.write_text(
                "frame\tactive_objects\n"
                "102\t0xA23C:pic=0xF8CE:x=0x10:y=0x20\n"
                "200\t0xA253:pic=0xF8E2:x=0x30:y=0x40\n",
                encoding="utf-8",
            )

            report = scan_reference_windows.scan_trace_tree(
                root,
                {"0xFE", "0xF3"},
                {
                    "0xF8CE": "SCZP1 converted mutant",
                    "0xF8E2": "SWXP1 swarmer explosion",
                },
                proximity_frames=4,
            )

        self.assertEqual(report["target_sound_hit_count"], 2)
        self.assertEqual(report["object_hit_count"], 2)
        self.assertEqual(report["candidate_count"], 1)
        self.assertEqual(report["window_candidates"][0]["frame"], 100)
        self.assertEqual(report["window_candidates"][0]["commands"], ["0xFE"])
        self.assertEqual(
            report["object_label_counts"],
            {
                "SCZP1 converted mutant": 1,
                "SWXP1 swarmer explosion": 1,
            },
        )
        self.assertEqual(report["object_label_spans"][0]["start_frame"], 102)
        self.assertEqual(report["object_label_spans"][0]["end_frame"], 102)
        self.assertEqual(
            [span["label"] for span in report["object_label_best_spans"]],
            ["SCZP1 converted mutant", "SWXP1 swarmer explosion"],
        )
        self.assertEqual(len(report["nearest_sound_object_misses"]), 1)
        self.assertEqual(report["nearest_sound_object_misses"][0]["sound_frame"], 140)
        self.assertEqual(
            report["nearest_sound_object_misses"][0]["distance_frames"],
            38,
        )

    def test_object_spans_merge_contiguous_label_frames_per_trace(self) -> None:
        hits = [
            scan_reference_windows.ObjectHit(
                Path("a.debug.tsv"),
                10,
                "SCZP1 converted mutant",
                "0xF8CE",
            ),
            scan_reference_windows.ObjectHit(
                Path("a.debug.tsv"),
                11,
                "SCZP1 converted mutant",
                "0xF8CE",
            ),
            scan_reference_windows.ObjectHit(
                Path("a.debug.tsv"),
                13,
                "SCZP1 converted mutant",
                "0xF8CE",
            ),
            scan_reference_windows.ObjectHit(
                Path("b.debug.tsv"),
                10,
                "SCZP1 converted mutant",
                "0xF8CE",
            ),
        ]

        spans = scan_reference_windows.object_spans(hits)

        self.assertEqual(
            [
                (span.path, span.start_frame, span.end_frame, span.hit_count)
                for span in spans
            ],
            [
                (Path("a.debug.tsv"), 10, 11, 2),
                (Path("a.debug.tsv"), 13, 13, 1),
                (Path("b.debug.tsv"), 10, 10, 1),
            ],
        )

    def test_best_object_spans_by_label_chooses_longest_span_per_family(self) -> None:
        spans = [
            scan_reference_windows.ObjectSpan(
                Path("b.debug.tsv"),
                "SCZP1 converted mutant",
                "0xF8CE",
                20,
                30,
                11,
            ),
            scan_reference_windows.ObjectSpan(
                Path("a.debug.tsv"),
                "SCZP1 converted mutant",
                "0xF8CE",
                10,
                30,
                21,
            ),
            scan_reference_windows.ObjectSpan(
                Path("c.debug.tsv"),
                "PRBP1 pod",
                "0xF8F7",
                50,
                60,
                11,
            ),
        ]

        best = scan_reference_windows.best_object_spans_by_label(spans)

        self.assertEqual(
            [
                (span.label, span.path, span.start_frame, span.end_frame)
                for span in best
            ],
            [
                ("PRBP1 pod", Path("c.debug.tsv"), 50, 60),
                ("SCZP1 converted mutant", Path("a.debug.tsv"), 10, 30),
            ],
        )

    def test_scan_terrain_hits_detects_terblo_and_last_human_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            debug = Path(directory) / "sample.debug.tsv"
            debug.write_text(
                "frame\tpc\tastcnt\tterrain_blown\tactive_processes\n"
                "100\t0xEE2C\t0x0A\ttrue\t"
                "0xAB88:paddr=0xEDEA:ptime=0x00\n"
                "101\t0xEE44\t0x00\ttrue\t"
                "0xAB88:paddr=0xEE44:ptime=0x01\n"
                "102\t0xEE2C\t0x00\ttrue\t"
                "0xAB88:paddr=0xEDEA:ptime=0x00\n",
                encoding="utf-8",
            )

            hits = scan_reference_windows.scan_terrain_hits(debug)

        self.assertEqual([hit.frame for hit in hits], [100, 101, 102])
        self.assertTrue(hits[0].has_terrain_blow_process)
        self.assertFalse(hits[0].is_last_human_candidate)
        self.assertFalse(hits[1].has_terrain_blow_process)
        self.assertTrue(hits[2].is_last_human_candidate)

    def test_scan_trace_tree_summarizes_terrain_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = root / "sample.expected.tsv"
            debug = root / "sample.debug.tsv"
            expected.write_text("frame\tsound_commands\n100\t-\n", encoding="utf-8")
            debug.write_text(
                "frame\tpc\tastcnt\tterrain_blown\tactive_processes\n"
                "100\t0xEE2C\t0x0A\ttrue\t"
                "0xAB88:paddr=0xEDEA:ptime=0x00\n"
                "101\t0xEE2C\t0x00\ttrue\t"
                "0xAB88:paddr=0xEDEA:ptime=0x00\n",
                encoding="utf-8",
            )

            report = scan_reference_windows.scan_trace_tree(
                root,
                {"0xFE"},
                {},
                proximity_frames=4,
            )

        self.assertEqual(report["terrain_trace_count"], 1)
        self.assertEqual(report["terrain_status_hit_count"], 2)
        self.assertEqual(report["terrain_terblo_process_hit_count"], 2)
        self.assertEqual(report["terrain_last_human_candidate_count"], 1)
        self.assertEqual(report["terrain_astcnt_counts"], {"0x00": 1, "0x0A": 1})
        self.assertEqual(len(report["terrain_process_hits"]), 2)
        self.assertEqual(report["terrain_last_human_candidates"][0]["frame"], 101)

    def test_report_text_lists_nearest_sound_object_misses(self) -> None:
        report = {
            "root": "target/reference-media/mame",
            "excluded_path_fragments": [],
            "expected_trace_count": 1,
            "debug_trace_count": 1,
            "target_commands": ["0xFE"],
            "target_sound_hit_count": 1,
            "object_hit_count": 1,
            "candidate_count": 0,
            "terrain_trace_count": 0,
            "terrain_status_hit_count": 0,
            "terrain_terblo_process_hit_count": 0,
            "terrain_last_human_candidate_count": 0,
            "terrain_astcnt_counts": {},
            "object_labels": ["TIEP3 bomber"],
            "window_candidates": [],
            "target_sound_hits": [
                {
                    "path": "target/reference-media/mame/sample.expected.tsv",
                    "frame": 100,
                    "commands": ["0xFE"],
                }
            ],
            "nearest_sound_object_misses": [
                {
                    "expected_path": "target/reference-media/mame/sample.expected.tsv",
                    "debug_path": "target/reference-media/mame/sample.debug.tsv",
                    "sound_frame": 100,
                    "commands": ["0xFE"],
                    "object_frame": 140,
                    "distance_frames": 40,
                    "nearest_object": {
                        "path": "target/reference-media/mame/sample.debug.tsv",
                        "frame": 140,
                        "label": "TIEP3 bomber",
                        "address": "0xF93D",
                    },
                }
            ],
        }

        text = scan_reference_windows.report_text(report)

        self.assertIn("nearest sound/object misses:", text)
        self.assertIn("sound frame 100 commands 0xFE", text)
        self.assertIn("nearest TIEP3 bomber at frame 140", text)

    def test_scan_trace_tree_excludes_matching_path_fragments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            included = root / "organic.expected.tsv"
            included_debug = root / "organic.debug.tsv"
            excluded_dir = root / "synthetic-sound"
            excluded_dir.mkdir()
            excluded = excluded_dir / "synthetic.expected.tsv"
            excluded_debug = excluded_dir / "synthetic.debug.tsv"
            included.write_text("frame\tsound_commands\n100\t0xFE\n", encoding="utf-8")
            included_debug.write_text("frame\tactive_objects\n100\t-\n", encoding="utf-8")
            excluded.write_text("frame\tsound_commands\n101\t0xFA\n", encoding="utf-8")
            excluded_debug.write_text(
                "frame\tactive_objects\n101\tpic=0xF8F7\n",
                encoding="utf-8",
            )

            report = scan_reference_windows.scan_trace_tree(
                root,
                {"0xFE", "0xFA"},
                {"0xF8F7": "PRBP1 pod"},
                proximity_frames=4,
                excluded_path_fragments=("synthetic-sound",),
            )

        self.assertEqual(report["expected_trace_count"], 1)
        self.assertEqual(report["target_sound_hit_count"], 1)
        self.assertEqual(report["candidate_count"], 0)
        self.assertEqual(report["excluded_path_fragments"], ["synthetic-sound"])

    def test_report_text_names_missing_target_sound_hits(self) -> None:
        report = {
            "root": "target/reference-media/mame",
            "excluded_path_fragments": [],
            "expected_trace_count": 1,
            "debug_trace_count": 1,
            "target_commands": ["0xFE"],
            "target_sound_hit_count": 0,
            "object_hit_count": 2,
            "candidate_count": 0,
            "object_labels": ["SCZP1 converted mutant"],
            "object_label_counts": {"SCZP1 converted mutant": 2},
            "object_label_spans": [
                {
                    "path": "target/reference-media/mame/sample.debug.tsv",
                    "label": "SCZP1 converted mutant",
                    "address": "0xF8CE",
                    "start_frame": 100,
                    "end_frame": 140,
                    "duration_frames": 41,
                    "hit_count": 41,
                }
            ],
            "object_label_best_spans": [
                {
                    "path": "target/reference-media/mame/sample.debug.tsv",
                    "label": "SCZP1 converted mutant",
                    "address": "0xF8CE",
                    "start_frame": 100,
                    "end_frame": 140,
                    "duration_frames": 41,
                    "hit_count": 41,
                }
            ],
            "window_candidates": [],
            "target_sound_hits": [],
        }

        text = scan_reference_windows.report_text(report)

        self.assertIn("no target sound hits", text)
        self.assertIn("object label counts: SCZP1 converted mutant=2", text)
        self.assertIn("longest object spans:", text)
        self.assertIn("best object spans by label:", text)
        self.assertIn("SCZP1 converted mutant frames 100-140", text)

    def test_report_text_names_terrain_boundary_without_last_human_candidate(
        self,
    ) -> None:
        report = {
            "root": "target/reference-media/mame",
            "excluded_path_fragments": [],
            "expected_trace_count": 1,
            "debug_trace_count": 1,
            "target_commands": ["0xFE"],
            "target_sound_hit_count": 0,
            "object_hit_count": 0,
            "candidate_count": 0,
            "object_labels": [],
            "window_candidates": [],
            "target_sound_hits": [],
            "terrain_trace_count": 1,
            "terrain_status_hit_count": 2,
            "terrain_terblo_process_hit_count": 2,
            "terrain_last_human_candidate_count": 0,
            "terrain_astcnt_counts": {"0x0A": 2},
            "terrain_last_human_candidates": [],
        }

        text = scan_reference_windows.report_text(report)

        self.assertIn("terrain TERBLO process rows exist", text)
        self.assertIn("terrain ASTCNT values: 0x0A=2", text)

    def test_report_text_lists_terrain_process_misses(self) -> None:
        report = {
            "root": "target/reference-media/mame",
            "excluded_path_fragments": [],
            "expected_trace_count": 1,
            "debug_trace_count": 1,
            "target_commands": ["0xFE"],
            "target_sound_hit_count": 0,
            "object_hit_count": 0,
            "candidate_count": 0,
            "object_labels": [],
            "window_candidates": [],
            "target_sound_hits": [],
            "terrain_trace_count": 1,
            "terrain_status_hit_count": 1,
            "terrain_terblo_process_hit_count": 1,
            "terrain_last_human_candidate_count": 0,
            "terrain_astcnt_counts": {"0x0A": 1},
            "terrain_last_human_candidates": [],
            "terrain_process_hits": [
                {
                    "path": "target/reference-media/mame/terrain.debug.tsv",
                    "frame": 1450,
                    "pc": "0xEE2C",
                    "astcnt": "0x0A",
                    "terrain_blown": True,
                    "process_addresses": ["0xEDEA"],
                    "has_terrain_blow_process": True,
                    "is_last_human_candidate": False,
                }
            ],
        }

        text = scan_reference_windows.report_text(report)

        self.assertIn("terrain TERBLO process misses:", text)
        self.assertIn("frame 1450 astcnt 0x0A", text)


if __name__ == "__main__":
    unittest.main()
