"""Package a native Defender executable and instructions for GitHub downloads."""

import argparse
import hashlib
from pathlib import Path
import re
import shutil
import struct
import tarfile
import tempfile
import zipfile


TARGETS = {
    "aarch64-apple-darwin": ("defender", ".tar.gz"),
    "x86_64-pc-windows-msvc": ("defender.exe", ".zip"),
}
ROOT = Path(__file__).resolve().parents[1]


def verify_architecture(binary, target):
    """Reject a package containing an executable for the wrong platform."""
    data = binary.read_bytes()
    if target == "aarch64-apple-darwin":
        valid = (
            len(data) >= 8
            and data[:4] == b"\xcf\xfa\xed\xfe"
            and struct.unpack_from("<I", data, 4)[0] == 0x0100000C
        )
    else:
        valid = False
        if len(data) >= 64 and data[:2] == b"MZ":
            offset = struct.unpack_from("<I", data, 60)[0]
            valid = (
                offset + 6 <= len(data)
                and data[offset:offset + 4] == b"PE\0\0"
                and struct.unpack_from("<H", data, offset + 4)[0] == 0x8664
            )
    if not valid:
        raise ValueError(f"{binary} is not a {target} executable")


def package(target, version, output):
    """Archive only the executable and player docs, with a SHA-256 sidecar."""
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", version):
        raise ValueError("version must contain only letters, digits, dots, underscores or hyphens")
    name, suffix = TARGETS[target]
    binary = ROOT / "target" / target / "release" / name
    verify_architecture(binary, target)
    output.mkdir(parents=True, exist_ok=True)
    stem = f"defender-{version}-{target}"
    archive = output / f"{stem}{suffix}"
    with tempfile.TemporaryDirectory() as temporary:
        folder = Path(temporary) / stem
        folder.mkdir()
        executable = folder / name
        shutil.copy2(binary, executable)
        executable.chmod(0o755)
        shutil.copy2(ROOT / "docs" / "PLAYING.md", folder / "README.md")
        shutil.copy2(ROOT / "SUPPORT.md", folder / "SUPPORT.md")
        if suffix == ".zip":
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as bundle:
                for entry in sorted(folder.iterdir()):
                    bundle.write(entry, f"{stem}/{entry.name}")
        else:
            with tarfile.open(archive, "w:gz") as bundle:
                bundle.add(folder, arcname=stem)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(
        f"{digest}  {archive.name}\n", encoding="ascii"
    )
    print(archive)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "dist")
    args = parser.parse_args()
    package(args.target, args.version, args.output)


if __name__ == "__main__":
    main()
