#!/usr/bin/env python3
"""Unit tests for the reference report closure gate."""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_reference_reports.py")
SPEC = importlib.util.spec_from_file_location("check_reference_reports", MODULE_PATH)
assert SPEC is not None
check_reference_reports = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = check_reference_reports
SPEC.loader.exec_module(check_reference_reports)


def write_json(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data), encoding="utf-8")


def touch(path: Path) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"artifact")
    return str(path)


def touch_empty(path: Path) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"")
    return str(path)


def relative_to_cwd(path: Path) -> Path:
    return path.resolve().relative_to(Path.cwd().resolve())


class ReferenceReportGateTests(unittest.TestCase):
    def test_check_reports_accepts_all_mode_pass(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "all",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                        "failures": [],
                    },
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                        "failures": [],
                    },
                    "reference_video": touch(root / "mame.mp4"),
                    "reference_audio": touch(root / "mame.wav"),
                    "candidate_media": touch(root / "clean.gif"),
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "all report",
                        report_path,
                        "all",
                    )
                ]
            )

        self.assertEqual(failures, [])

    def test_check_reports_rejects_missing_report_acceptance_mode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                        "failures": [],
                    },
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                        "failures": [],
                    },
                    "reference_video": touch(root / "mame.mp4"),
                    "reference_audio": touch(root / "mame.wav"),
                    "candidate_media": touch(root / "clean.gif"),
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "missing mode",
                        report_path,
                        "all",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("acceptance_mode is ''", failures[0])

    def test_check_reports_accepts_axis_scoped_modes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            visual_path = root / "visual.json"
            audio_path = root / "audio.json"
            write_json(
                visual_path,
                {
                    "status": "pass",
                    "acceptance_mode": "visual",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                        "failures": [],
                    },
                    "audio": {"status": "fail"},
                    "reference_video": touch(root / "visual-mame.mp4"),
                    "candidate_media": touch(root / "visual-clean.gif"),
                },
            )
            write_json(
                audio_path,
                {
                    "status": "pass",
                    "acceptance_mode": "audio",
                    "visual": {"status": "fail"},
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                        "failures": [],
                    },
                    "reference_audio": touch(root / "audio-mame.wav"),
                    "candidate_audio": touch(root / "audio-clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "visual report",
                        visual_path,
                        "visual",
                        ("explosion_visual",),
                    ),
                    check_reference_reports.ReportExpectation(
                        "audio report",
                        audio_path,
                        "audio",
                        ("explosion_audio",),
                    ),
                ]
            )

        self.assertEqual(failures, [])

    def test_check_reports_rejects_missing_required_media_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "all",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                    },
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                    },
                    "reference_video": str(root / "missing-mame.mp4"),
                    "reference_audio": touch(root / "mame.wav"),
                    "candidate_media": touch(root / "clean.gif"),
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "stale report",
                        report_path,
                        "all",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("missing reference visual media artifact", failures[0])

    def test_check_reports_rejects_required_media_outside_expected_roots(self) -> None:
        check_reference_reports.REPORT_ROOT.mkdir(parents=True, exist_ok=True)
        check_reference_reports.REFERENCE_MEDIA_ROOT.mkdir(parents=True, exist_ok=True)
        check_reference_reports.CANDIDATE_MEDIA_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(
            dir=check_reference_reports.REPORT_ROOT
        ) as report_directory:
            with tempfile.TemporaryDirectory(
                dir=check_reference_reports.REFERENCE_MEDIA_ROOT
            ) as mame_directory:
                with tempfile.TemporaryDirectory(
                    dir=check_reference_reports.CANDIDATE_MEDIA_ROOT
                ) as clean_directory:
                    report_root = relative_to_cwd(Path(report_directory))
                    mame_root = relative_to_cwd(Path(mame_directory))
                    clean_root = relative_to_cwd(Path(clean_directory))
                    report_path = report_root / "report.json"
                    write_json(
                        report_path,
                        {
                            "status": "pass",
                            "acceptance_mode": "visual",
                            "visual": {
                                "status": "pass",
                                "reference_frames": 2,
                                "candidate_frames": 2,
                                "compared_frames": 2,
                                "failures": [],
                            },
                            "reference_video": touch(report_root / "mame.mp4"),
                            "candidate_media": touch(report_root / "clean.gif"),
                            "reference_audio": touch(mame_root / "mame.wav"),
                            "candidate_audio": touch(clean_root / "clean.wav"),
                        },
                    )

                    failures = check_reference_reports.check_reports(
                        [
                            check_reference_reports.ReportExpectation(
                                "root mismatch",
                                report_path,
                                "visual",
                            )
                        ]
                    )

        self.assertEqual(len(failures), 2)
        self.assertTrue(
            any("reference visual media artifact" in failure for failure in failures)
        )
        self.assertTrue(
            any("candidate visual media artifact" in failure for failure in failures)
        )

    def test_check_reports_rejects_required_media_with_wrong_suffixes(self) -> None:
        check_reference_reports.REPORT_ROOT.mkdir(parents=True, exist_ok=True)
        check_reference_reports.REFERENCE_MEDIA_ROOT.mkdir(parents=True, exist_ok=True)
        check_reference_reports.CANDIDATE_MEDIA_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(
            dir=check_reference_reports.REPORT_ROOT
        ) as report_directory:
            with tempfile.TemporaryDirectory(
                dir=check_reference_reports.REFERENCE_MEDIA_ROOT
            ) as mame_directory:
                with tempfile.TemporaryDirectory(
                    dir=check_reference_reports.CANDIDATE_MEDIA_ROOT
                ) as clean_directory:
                    report_root = relative_to_cwd(Path(report_directory))
                    mame_root = relative_to_cwd(Path(mame_directory))
                    clean_root = relative_to_cwd(Path(clean_directory))
                    report_path = report_root / "report.json"
                    write_json(
                        report_path,
                        {
                            "status": "pass",
                            "acceptance_mode": "all",
                            "visual": {
                                "status": "pass",
                                "reference_frames": 2,
                                "candidate_frames": 2,
                                "compared_frames": 2,
                                "failures": [],
                            },
                            "audio": {
                                "status": "pass",
                                "reference_samples": 64,
                                "candidate_samples": 64,
                                "compared_samples": 64,
                                "failures": [],
                            },
                            "reference_video": touch(mame_root / "mame.gif"),
                            "candidate_media": touch(clean_root / "clean.mp4"),
                            "reference_audio": touch(mame_root / "mame.mp3"),
                            "candidate_audio": touch(clean_root / "clean.mp3"),
                        },
                    )

                    failures = check_reference_reports.check_reports(
                        [
                            check_reference_reports.ReportExpectation(
                                "wrong suffixes",
                                report_path,
                                "all",
                            )
                        ]
                    )

        self.assertEqual(len(failures), 4)
        self.assertTrue(all("must use" in failure for failure in failures))
        self.assertTrue(any(".mp4" in failure for failure in failures))
        self.assertTrue(any(".gif" in failure for failure in failures))
        self.assertTrue(any(".wav" in failure for failure in failures))

    def test_check_reports_rejects_missing_required_media_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "audio",
                    "visual": {"status": "fail"},
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                    },
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "incomplete audio report",
                        report_path,
                        "audio",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("missing reference audio path", failures[0])

    def test_check_reports_rejects_empty_required_media_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "visual",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                    },
                    "audio": {"status": "fail"},
                    "reference_video": touch_empty(root / "empty-mame.mp4"),
                    "candidate_media": touch(root / "clean.gif"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "empty visual report",
                        report_path,
                        "visual",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("empty reference visual media artifact", failures[0])

    def test_check_reports_rejects_visual_report_without_compared_frames(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "visual",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 0,
                    },
                    "audio": {"status": "fail"},
                    "reference_video": touch(root / "mame.mp4"),
                    "candidate_media": touch(root / "clean.gif"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "empty visual comparison",
                        report_path,
                        "visual",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("visual compared_frames is not positive", failures[0])

    def test_check_reports_rejects_audio_report_without_compared_samples(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "audio",
                    "visual": {"status": "fail"},
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 0,
                    },
                    "reference_audio": touch(root / "mame.wav"),
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "empty audio comparison",
                        report_path,
                        "audio",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("audio compared_samples is not positive", failures[0])

    def test_check_reports_rejects_accepted_visual_failures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "visual",
                    "visual": {
                        "status": "pass",
                        "reference_frames": 2,
                        "candidate_frames": 2,
                        "compared_frames": 2,
                        "failures": ["visual threshold failed"],
                    },
                    "audio": {"status": "fail", "failures": ["diagnostic only"]},
                    "reference_video": touch(root / "mame.mp4"),
                    "candidate_media": touch(root / "clean.gif"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "stale visual failure",
                        report_path,
                        "visual",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("visual failures are present on accepted axis", failures[0])

    def test_check_reports_rejects_accepted_audio_failures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "audio",
                    "visual": {"status": "fail", "failures": ["diagnostic only"]},
                    "audio": {
                        "status": "pass",
                        "reference_samples": 64,
                        "candidate_samples": 64,
                        "compared_samples": 64,
                        "failures": ["audio threshold failed"],
                    },
                    "reference_audio": touch(root / "mame.wav"),
                    "candidate_audio": touch(root / "clean.wav"),
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "stale audio failure",
                        report_path,
                        "audio",
                    )
                ]
            )

        self.assertEqual(len(failures), 1)
        self.assertIn("audio failures are present on accepted axis", failures[0])

    def test_check_coverage_requirements_accepts_axis_coverage(self) -> None:
        expectations = [
            check_reference_reports.ReportExpectation(
                "full report",
                Path("full.json"),
                "all",
                ("laser_visual", "laser_audio"),
            ),
            check_reference_reports.ReportExpectation(
                "visual report",
                Path("visual.json"),
                "visual",
                ("coalescence_visual",),
            ),
        ]
        requirements = [
            check_reference_reports.CoverageRequirement(
                "laser visual",
                "laser_visual",
                "visual",
                1,
            ),
            check_reference_reports.CoverageRequirement(
                "laser audio",
                "laser_audio",
                "audio",
                1,
            ),
            check_reference_reports.CoverageRequirement(
                "coalescence visual",
                "coalescence_visual",
                "visual",
                1,
            ),
        ]

        failures = check_reference_reports.check_coverage_requirements(
            expectations,
            requirements,
        )

        self.assertEqual(failures, [])

    def test_check_coverage_requirements_rejects_missing_axis_coverage(self) -> None:
        expectations = [
            check_reference_reports.ReportExpectation(
                "audio-only report",
                Path("audio.json"),
                "audio",
                ("laser_visual",),
            )
        ]
        requirements = [
            check_reference_reports.CoverageRequirement(
                "laser visual",
                "laser_visual",
                "visual",
                1,
            )
        ]

        failures = check_reference_reports.check_coverage_requirements(
            expectations,
            requirements,
        )

        self.assertEqual(len(failures), 1)
        self.assertIn("laser visual", failures[0])
        self.assertIn("laser_visual", failures[0])

    def test_check_coverage_requirements_rejects_insufficient_report_breadth(self) -> None:
        expectations = [
            check_reference_reports.ReportExpectation(
                "laser proof one",
                Path("laser-one.json"),
                "all",
                ("laser_visual",),
            ),
            check_reference_reports.ReportExpectation(
                "diagnostic laser proof",
                Path("laser-diagnostic.json"),
                "audio",
                ("laser_visual",),
            ),
        ]
        requirements = [
            check_reference_reports.CoverageRequirement(
                "laser visual breadth",
                "laser_visual",
                "visual",
                2,
            )
        ]

        failures = check_reference_reports.check_coverage_requirements(
            expectations,
            requirements,
        )

        self.assertEqual(len(failures), 1)
        self.assertIn("1 accepted visual report(s)", failures[0])
        self.assertIn("expected at least 2", failures[0])

    def test_check_reports_rejects_mode_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report_path = Path(directory) / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "visual",
                    "visual": {"status": "pass"},
                    "audio": {"status": "fail"},
                },
            )

            failures = check_reference_reports.check_reports(
                [
                    check_reference_reports.ReportExpectation(
                        "mode mismatch",
                        report_path,
                        "all",
                    )
                ]
            )

        self.assertTrue(any("acceptance_mode" in failure for failure in failures))
        self.assertTrue(any("audio status" in failure for failure in failures))

    def test_check_reports_rejects_missing_report(self) -> None:
        failures = check_reference_reports.check_reports(
            [
                check_reference_reports.ReportExpectation(
                    "missing",
                    Path("does/not/exist/report.json"),
                    "all",
                )
            ]
        )

        self.assertEqual(len(failures), 1)
        self.assertIn("missing report", failures[0])

    def test_load_manifest_validates_acceptance_mode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "reports": [
                        {
                            "name": "bad mode",
                            "path": "target/reference-media/report.json",
                            "acceptance_mode": "partial",
                        }
                    ]
                },
            )

            with self.assertRaises(ValueError):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_missing_acceptance_mode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser.json",
                            "coverage": ["player_laser_visual"],
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "explicit acceptance_mode"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_report_path_outside_reference_media_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "report.json",
                            "acceptance_mode": "visual",
                            "coverage": ["player_laser_visual"],
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "path must be under"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_validates_coverage_requirement_axis(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "bad requirement",
                            "coverage": "laser_visual",
                            "accepted_axis": "partial",
                        }
                    ],
                    "reports": [],
                },
            )

            with self.assertRaises(ValueError):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_validates_coverage_requirement_min_reports(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "bad breadth",
                            "coverage": "laser_visual",
                            "accepted_axis": "visual",
                            "min_reports": 0,
                        }
                    ],
                    "reports": [],
                },
            )

            with self.assertRaises(ValueError):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_duplicate_coverage_requirement_name(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser proof",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        },
                        {
                            "name": "laser proof",
                            "coverage": "player_laser_audio",
                            "accepted_axis": "audio",
                        },
                    ],
                    "reports": [],
                },
            )

            with self.assertRaisesRegex(ValueError, "duplicates requirement name"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_duplicate_coverage_requirement_key(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser proof one",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        },
                        {
                            "name": "laser proof two",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        },
                    ],
                    "reports": [],
                },
            )

            with self.assertRaisesRegex(ValueError, "duplicates coverage"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_duplicate_report_name(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser-one.json",
                            "acceptance_mode": "visual",
                            "coverage": ["player_laser_visual"],
                        },
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser-two.json",
                            "acceptance_mode": "visual",
                            "coverage": ["player_laser_visual"],
                        },
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "duplicates report name"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_duplicate_report_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof one",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "visual",
                            "coverage": ["player_laser_visual"],
                        },
                        {
                            "name": "laser proof two",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "visual",
                            "coverage": ["player_laser_visual"],
                        },
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "duplicates report path"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_duplicate_report_coverage_tags(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "visual",
                            "coverage": [
                                "player_laser_visual",
                                "player_laser_visual",
                            ],
                        }
                    ]
                },
            )

            with self.assertRaisesRegex(ValueError, "duplicate tags"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_empty_report_coverage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "visual",
                            "coverage": [],
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "at least one tag"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_undeclared_report_coverage_tag(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "visual",
                            "coverage": ["enemy_explosion_visual"],
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "no matching coverage requirement"):
                check_reference_reports.load_manifest(manifest)

    def test_load_manifest_rejects_mode_incompatible_report_coverage_tag(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "manifest.json"
            write_json(
                manifest,
                {
                    "coverage_requirements": [
                        {
                            "name": "laser visual",
                            "coverage": "player_laser_visual",
                            "accepted_axis": "visual",
                        }
                    ],
                    "reports": [
                        {
                            "name": "laser proof",
                            "path": "target/reference-media/laser.json",
                            "acceptance_mode": "audio",
                            "coverage": ["player_laser_visual"],
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(ValueError, "not accepted"):
                check_reference_reports.load_manifest(manifest)

    def test_build_summary_reports_coverage_and_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "all",
                    "reference_video": "mame.mp4",
                    "candidate_media": "clean.gif",
                    "visual": {
                        "status": "pass",
                        "full": {"rms": 12.34567},
                        "regions": [
                            {"name": "playfield", "rms": 3.25},
                            {"name": "laser_band", "rms": 4.5},
                        ],
                    },
                    "audio": {
                        "status": "pass",
                        "envelope_correlation": 0.81234,
                        "rms_ratio": 1.125,
                        "zero_crossing_ratio": 0.75,
                        "noise_like_pass": True,
                    },
                },
            )
            manifest = check_reference_reports.ReportManifest(
                reports=[
                    check_reference_reports.ReportExpectation(
                        "laser proof",
                        report_path,
                        "all",
                        ("player_laser_visual", "player_laser_audio"),
                    )
                ],
                coverage_requirements=[
                    check_reference_reports.CoverageRequirement(
                        "laser visual",
                        "player_laser_visual",
                        "visual",
                        1,
                    )
                ],
            )

            summary = check_reference_reports.build_summary(manifest)

        self.assertIn("# Defender Reference Report Signoff Summary", summary)
        self.assertIn("- Accepted visual comparisons: `1`", summary)
        self.assertIn("- Accepted audio comparisons: `1`", summary)
        self.assertIn("| laser visual | player_laser_visual | visual | 1 | laser proof |", summary)
        self.assertIn("laser visual", summary)
        self.assertIn("laser proof", summary)
        self.assertIn(
            "| laser proof | all | all | player_laser_visual, player_laser_audio |",
            summary,
        )
        self.assertIn("full RMS 12.346", summary)
        self.assertIn("env 0.812", summary)
        self.assertIn("mame.mp4", summary)
        self.assertIn("clean.gif", summary)

    def test_build_summary_lists_each_accepted_report_once_per_requirement(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first_report = root / "first.json"
            second_report = root / "second.json"
            for report_path in (first_report, second_report):
                write_json(
                    report_path,
                    {
                        "status": "pass",
                        "acceptance_mode": "visual",
                        "visual": {"status": "pass"},
                        "audio": {"status": "fail"},
                    },
                )
            manifest = check_reference_reports.ReportManifest(
                reports=[
                    check_reference_reports.ReportExpectation(
                        "first proof",
                        first_report,
                        "visual",
                        ("sprite_visuals",),
                    ),
                    check_reference_reports.ReportExpectation(
                        "second proof",
                        second_report,
                        "visual",
                        ("sprite_visuals",),
                    ),
                ],
                coverage_requirements=[
                    check_reference_reports.CoverageRequirement(
                        "sprite visuals",
                        "sprite_visuals",
                        "visual",
                        2,
                    )
                ],
            )

            summary = check_reference_reports.build_summary(manifest)

        self.assertIn(
            "| sprite visuals | sprite_visuals | visual | 2 | first proof, second proof |",
            summary,
        )
        self.assertNotIn("first proof, second proof, first proof", summary)

    def test_build_summary_includes_window_scan_boundaries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report_path = root / "report.json"
            scan_path = root / "reference-window-scan.json"
            write_json(
                report_path,
                {
                    "status": "pass",
                    "acceptance_mode": "all",
                    "visual": {"status": "pass"},
                    "audio": {"status": "pass"},
                },
            )
            write_json(
                scan_path,
                {
                    "expected_trace_count": 3,
                    "debug_trace_count": 2,
                    "target_sound_hit_count": 1,
                    "object_hit_count": 5,
                    "object_label_counts": {
                        "PRBP1 pod": 3,
                        "TIEP3 bomber": 2,
                    },
                    "object_label_spans": [
                        {
                            "label": "PRBP1 pod",
                            "start_frame": 210,
                            "end_frame": 260,
                            "duration_frames": 51,
                        },
                        {
                            "label": "TIEP3 bomber",
                            "start_frame": 80,
                            "end_frame": 100,
                            "duration_frames": 21,
                        },
                    ],
                    "object_label_best_spans": [
                        {
                            "label": "BXPIC bomb explosion",
                            "start_frame": 3709,
                            "end_frame": 7000,
                            "duration_frames": 3292,
                        },
                        {
                            "label": "PRBP1 pod",
                            "start_frame": 210,
                            "end_frame": 260,
                            "duration_frames": 51,
                        },
                    ],
                    "candidate_count": 0,
                    "terrain_status_hit_count": 8,
                    "terrain_terblo_process_hit_count": 1,
                    "terrain_last_human_candidate_count": 0,
                    "excluded_path_fragments": ["state-steered"],
                    "terrain_astcnt_counts": {"0x00": 4, "0x0A": 4},
                    "nearest_sound_object_misses": [
                        {
                            "sound_frame": 100,
                            "commands": ["0xFE"],
                            "object_frame": 140,
                            "distance_frames": 40,
                            "nearest_object": {"label": "TIEP3 bomber"},
                        }
                    ],
                    "terrain_process_hits": [
                        {"frame": 1450, "astcnt": "0x0A"},
                    ],
                },
            )
            manifest = check_reference_reports.ReportManifest(
                reports=[
                    check_reference_reports.ReportExpectation(
                        "basic proof",
                        report_path,
                        "all",
                    )
                ],
                coverage_requirements=[],
            )

            summary = check_reference_reports.build_summary(
                manifest,
                [
                    check_reference_reports.WindowScanExpectation(
                        "organic only",
                        scan_path,
                    )
                ],
            )

        self.assertIn("## Reference Window Scans", summary)
        self.assertIn("organic only", summary)
        self.assertIn("state-steered", summary)
        self.assertIn("PRBP1 pod=3, TIEP3 bomber=2", summary)
        self.assertIn("BXPIC bomb explosion frames 3709-7000", summary)
        self.assertIn("PRBP1 pod frames 210-260 (51 frame(s))", summary)
        self.assertIn("0x00=4, 0x0A=4", summary)
        self.assertIn("sound 100 0xFE; TIEP3 bomber at 140", summary)
        self.assertIn("frame 1450 ASTCNT 0x0A", summary)
        self.assertIn("| organic only |", summary)


if __name__ == "__main__":
    unittest.main()
