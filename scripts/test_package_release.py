"""Check the downloadable archive contract without needing native compilers."""

import hashlib
from pathlib import Path
import struct
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import package_release


class ReleasePackageTests(unittest.TestCase):
    def test_archives_contain_executable_and_docs_with_matching_checksum(self):
        for target, (name, suffix) in package_release.TARGETS.items():
            with self.subTest(target=target), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                binary = root / "target" / target / "release" / name
                binary.parent.mkdir(parents=True)
                if target == "aarch64-apple-darwin":
                    binary.write_bytes(struct.pack("<II", 0xFEEDFACF, 0x0100000C))
                else:
                    header = bytearray(70)
                    header[:2] = b"MZ"
                    struct.pack_into("<I", header, 60, 64)
                    header[64:] = b"PE\0\0\x64\x86"
                    binary.write_bytes(header)
                (root / "docs").mkdir()
                (root / "docs" / "PLAYING.md").write_text("Player instructions")
                (root / "SUPPORT.md").write_text("Support")
                with patch.object(package_release, "ROOT", root):
                    package_release.package(target, "v0.1.0", root / "dist")
                stem = f"defender-v0.1.0-{target}"
                archive = root / "dist" / f"{stem}{suffix}"
                expected = {f"{stem}/{name}", f"{stem}/README.md", f"{stem}/SUPPORT.md"}
                if suffix == ".zip":
                    with zipfile.ZipFile(archive) as bundle:
                        self.assertEqual(set(bundle.namelist()), expected)
                        self.assertEqual(bundle.read(f"{stem}/{name}"), binary.read_bytes())
                else:
                    with tarfile.open(archive) as bundle:
                        files = {entry.name for entry in bundle if entry.isfile()}
                        self.assertEqual(files, expected)
                        self.assertEqual(bundle.getmember(f"{stem}/{name}").mode, 0o755)
                        self.assertEqual(bundle.extractfile(f"{stem}/{name}").read(), binary.read_bytes())
                checksum = archive.with_name(archive.name + ".sha256").read_text()
                self.assertEqual(checksum, f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")

    def test_wrong_and_truncated_executables_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            binary = Path(temporary) / "defender"
            for data in (b"", b"MZ", b"\xcf\xfa\xed\xfe", b"MZ" + bytes(62)):
                binary.write_bytes(data)
                for target in package_release.TARGETS:
                    with self.subTest(data=data, target=target), self.assertRaises(ValueError):
                        package_release.verify_architecture(binary, target)

    def test_version_cannot_escape_output_directory(self):
        for version in ("../escape", "v1/other", "", "v1\nother"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                package_release.package("aarch64-apple-darwin", version, Path("unused"))


if __name__ == "__main__":
    unittest.main()
