#!/usr/bin/env python3
"""Unit tests for local MAME reference trace generation helpers."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("generate_reference_traces.py")
SPEC = importlib.util.spec_from_file_location("generate_reference_traces", MODULE_PATH)
assert SPEC is not None
generate_reference_traces = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = generate_reference_traces
SPEC.loader.exec_module(generate_reference_traces)


class CmosDefaultNvramTests(unittest.TestCase):
    def test_source_cmos_default_nvram_packs_mame_four_bit_cells(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            defaults = Path(directory) / "defaults.tsv"
            defaults.write_text(
                "symbol\toffset\tcells\tbytes\tdescription\tsource\n"
                "CMOSCK\t0x7F\t2\t0x5A\tcheck byte\tromc8.src\n"
                "REPLAY\t0x81\t4\t0x01 0x00\treplay\tromc8.src\n",
                encoding="utf-8",
            )

            nvram = generate_reference_traces.source_cmos_default_nvram(defaults)

        self.assertEqual(len(nvram), 256)
        self.assertEqual(nvram[0x00], 0xF0)
        self.assertEqual(nvram[0x7F], 0xF5)
        self.assertEqual(nvram[0x80], 0xFA)
        self.assertEqual(nvram[0x81:0x85], bytes([0xF0, 0xF1, 0xF0, 0xF0]))

    def test_source_cmos_default_nvram_rejects_cell_count_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            defaults = Path(directory) / "defaults.tsv"
            defaults.write_text(
                "symbol\toffset\tcells\tbytes\tdescription\tsource\n"
                "CMOSCK\t0x7F\t4\t0x5A\tcheck byte\tromc8.src\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "declared 4 cell"):
                generate_reference_traces.source_cmos_default_nvram(defaults)

    def test_source_cmos_default_nvram_rejects_out_of_range_defaults(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            defaults = Path(directory) / "defaults.tsv"
            defaults.write_text(
                "symbol\toffset\tcells\tbytes\tdescription\tsource\n"
                "TOO_FAR\t0xFF\t2\t0x5A\tpast end\tromc8.src\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "exceed 256"):
                generate_reference_traces.source_cmos_default_nvram(defaults)

    def test_write_source_cmos_default_nvram_uses_mame_defender_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            nvram_root = Path(directory) / "nvram"

            nvram_path = generate_reference_traces.write_source_cmos_default_nvram(nvram_root)

            self.assertEqual(nvram_path, nvram_root / "defender" / "nvram")
            self.assertEqual(nvram_path.read_bytes()[0x7F:0x81], bytes([0xF5, 0xFA]))


if __name__ == "__main__":
    unittest.main()
