#!/usr/bin/env python3
"""Compare clean Defender candidate media against a local reference video.

The harness intentionally uses local files. Downloaded YouTube captures, MAME
recordings, and owner-provided videos should live outside the repository or
under ignored output directories. The script extracts deterministic visual
samples and mono float audio with ffmpeg, then writes machine-readable reports.
"""

from __future__ import annotations

import argparse
import json
import math
import shutil
import struct
import subprocess
import sys
import zlib
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any


DEFAULT_WIDTH = 768
DEFAULT_HEIGHT = 576
DEFAULT_VISUAL_FPS = 6.0
DEFAULT_MAX_FRAMES = 240
DEFAULT_PIXEL_STRIDE = 4
DEFAULT_AUDIO_SAMPLE_RATE = 22_050
DEFAULT_AUDIO_WINDOW_MS = 20
DEFAULT_VISUAL_RMS_THRESHOLD = 38.0
DEFAULT_VISUAL_MAE_THRESHOLD = 28.0
DEFAULT_AUDIO_NRMS_THRESHOLD = 0.90
DEFAULT_AUDIO_CORRELATION_THRESHOLD = 0.10
DEFAULT_AUDIO_ENVELOPE_CORRELATION_THRESHOLD = 0.25
DEFAULT_AUDIO_SILENCE_RMS_THRESHOLD = 0.001
DEFAULT_AUDIO_NOISE_RMS_RATIO_MIN = 0.70
DEFAULT_AUDIO_NOISE_RMS_RATIO_MAX = 1.35
DEFAULT_AUDIO_NOISE_ZCR_MIN = 0.009
DEFAULT_AUDIO_NOISE_ZCR_RATIO_MIN = 0.45
DEFAULT_AUDIO_NOISE_ZCR_RATIO_MAX = 1.75
RGB_CHANNELS = 3
FLOAT32_BYTES = 4


@dataclass(frozen=True)
class Region:
    name: str
    x: int
    y: int
    width: int
    height: int

    def clamped(self, width: int, height: int) -> "Region":
        x = max(0, min(self.x, width))
        y = max(0, min(self.y, height))
        right = max(x, min(self.x + self.width, width))
        bottom = max(y, min(self.y + self.height, height))
        return Region(self.name, x, y, right - x, bottom - y)


DEFAULT_REGIONS = (
    Region("hud", 0, 0, DEFAULT_WIDTH, 120),
    Region("scanner", 220, 0, 330, 120),
    Region("playfield", 0, 120, DEFAULT_WIDTH, 300),
    Region("laser_band", 0, 300, DEFAULT_WIDTH, 96),
    Region("terrain", 0, 390, DEFAULT_WIDTH, 186),
)


@dataclass
class VisualAggregate:
    name: str
    channel_count: int = 0
    sum_squared: int = 0
    sum_abs: int = 0
    max_abs: int = 0

    def add_diff(self, diff: int) -> None:
        absolute = abs(diff)
        self.channel_count += 1
        self.sum_squared += absolute * absolute
        self.sum_abs += absolute
        self.max_abs = max(self.max_abs, absolute)

    @property
    def rms(self) -> float:
        if self.channel_count == 0:
            return 0.0
        return math.sqrt(self.sum_squared / self.channel_count)

    @property
    def mae(self) -> float:
        if self.channel_count == 0:
            return 0.0
        return self.sum_abs / self.channel_count

    def to_report(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "sampled_channels": self.channel_count,
            "rms": self.rms,
            "mae": self.mae,
            "max_abs": self.max_abs,
        }


@dataclass(frozen=True)
class MediaConfig:
    width: int
    height: int
    visual_fps: float
    max_frames: int
    pixel_stride: int
    audio_sample_rate: int
    audio_window_ms: int
    audio_start_ms: float
    duration_ms: float | None


@dataclass(frozen=True)
class Thresholds:
    visual_rms: float
    visual_mae: float
    audio_nrms: float
    audio_correlation: float
    audio_envelope_correlation: float
    audio_silence_rms: float
    audio_noise_rms_ratio_min: float = DEFAULT_AUDIO_NOISE_RMS_RATIO_MIN
    audio_noise_rms_ratio_max: float = DEFAULT_AUDIO_NOISE_RMS_RATIO_MAX
    audio_noise_zcr_min: float = DEFAULT_AUDIO_NOISE_ZCR_MIN
    audio_noise_zcr_ratio_min: float = DEFAULT_AUDIO_NOISE_ZCR_RATIO_MIN
    audio_noise_zcr_ratio_max: float = DEFAULT_AUDIO_NOISE_ZCR_RATIO_MAX


@dataclass
class VisualComparison:
    status: str
    reference_frames: int
    candidate_frames: int
    compared_frames: int
    reference_signatures: list[str]
    candidate_signatures: list[str]
    full: VisualAggregate
    regions: list[VisualAggregate]
    first_failing_frame: int | None
    failures: list[str] = field(default_factory=list)

    def to_report(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "reference_frames": self.reference_frames,
            "candidate_frames": self.candidate_frames,
            "compared_frames": self.compared_frames,
            "reference_signatures": self.reference_signatures,
            "candidate_signatures": self.candidate_signatures,
            "first_failing_frame": self.first_failing_frame,
            "full": self.full.to_report(),
            "regions": [region.to_report() for region in self.regions],
            "failures": self.failures,
        }


@dataclass
class AudioComparison:
    status: str
    reference_samples: int = 0
    candidate_samples: int = 0
    compared_samples: int = 0
    reference_dc_offset: float = 0.0
    candidate_dc_offset: float = 0.0
    reference_rms: float = 0.0
    candidate_rms: float = 0.0
    diff_rms: float = 0.0
    normalized_diff_rms: float = 0.0
    reference_peak: float = 0.0
    candidate_peak: float = 0.0
    correlation: float = 0.0
    envelope_correlation: float = 0.0
    reference_zero_crossing_rate: float = 0.0
    candidate_zero_crossing_rate: float = 0.0
    rms_ratio: float = 0.0
    zero_crossing_ratio: float = 0.0
    noise_like_pass: bool = False
    failures: list[str] = field(default_factory=list)
    extraction_error: str | None = None

    def to_report(self) -> dict[str, Any]:
        return {
            "status": self.status,
            "reference_samples": self.reference_samples,
            "candidate_samples": self.candidate_samples,
            "compared_samples": self.compared_samples,
            "reference_dc_offset": self.reference_dc_offset,
            "candidate_dc_offset": self.candidate_dc_offset,
            "reference_rms": self.reference_rms,
            "candidate_rms": self.candidate_rms,
            "diff_rms": self.diff_rms,
            "normalized_diff_rms": self.normalized_diff_rms,
            "reference_peak": self.reference_peak,
            "candidate_peak": self.candidate_peak,
            "correlation": self.correlation,
            "envelope_correlation": self.envelope_correlation,
            "reference_zero_crossing_rate": self.reference_zero_crossing_rate,
            "candidate_zero_crossing_rate": self.candidate_zero_crossing_rate,
            "rms_ratio": self.rms_ratio,
            "zero_crossing_ratio": self.zero_crossing_ratio,
            "noise_like_pass": self.noise_like_pass,
            "failures": self.failures,
            "extraction_error": self.extraction_error,
        }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Verify clean Defender candidate visuals and audio against a local "
            "reference video."
        )
    )
    parser.add_argument("--reference-video", required=True, help="Local reference video path.")
    parser.add_argument(
        "--reference-audio",
        help=(
            "Optional local reference audio file. If omitted, audio is extracted "
            "from --reference-video."
        ),
    )
    parser.add_argument(
        "--candidate-media",
        required=True,
        help="Clean candidate video/GIF/image-sequence container to compare visually.",
    )
    parser.add_argument(
        "--candidate-audio",
        help=(
            "Optional clean candidate audio file. If omitted, audio is extracted "
            "from --candidate-media."
        ),
    )
    parser.add_argument("--out-dir", default="target/reference-media")
    parser.add_argument("--ffmpeg", default=None, help="ffmpeg executable path.")
    parser.add_argument("--width", type=int, default=DEFAULT_WIDTH)
    parser.add_argument("--height", type=int, default=DEFAULT_HEIGHT)
    parser.add_argument("--visual-fps", type=float, default=DEFAULT_VISUAL_FPS)
    parser.add_argument("--max-frames", type=int, default=DEFAULT_MAX_FRAMES)
    parser.add_argument("--pixel-stride", type=int, default=DEFAULT_PIXEL_STRIDE)
    parser.add_argument("--audio-sample-rate", type=int, default=DEFAULT_AUDIO_SAMPLE_RATE)
    parser.add_argument("--audio-window-ms", type=int, default=DEFAULT_AUDIO_WINDOW_MS)
    parser.add_argument("--audio-start-ms", type=float, default=0.0)
    parser.add_argument(
        "--reference-start-ms",
        type=float,
        help=(
            "Optional visual/audio start offset for the reference stream. "
            "Defaults to --audio-start-ms for backward compatibility."
        ),
    )
    parser.add_argument(
        "--candidate-start-ms",
        type=float,
        help=(
            "Optional visual/audio start offset for the candidate stream. "
            "Defaults to --audio-start-ms for backward compatibility."
        ),
    )
    parser.add_argument("--duration-ms", type=float)
    parser.add_argument(
        "--region",
        action="append",
        default=[],
        help="Additional or replacement region as name:x,y,width,height.",
    )
    parser.add_argument(
        "--visual-rms-threshold",
        type=float,
        default=DEFAULT_VISUAL_RMS_THRESHOLD,
    )
    parser.add_argument(
        "--visual-mae-threshold",
        type=float,
        default=DEFAULT_VISUAL_MAE_THRESHOLD,
    )
    parser.add_argument(
        "--audio-nrms-threshold",
        type=float,
        default=DEFAULT_AUDIO_NRMS_THRESHOLD,
    )
    parser.add_argument(
        "--audio-correlation-threshold",
        type=float,
        default=DEFAULT_AUDIO_CORRELATION_THRESHOLD,
    )
    parser.add_argument(
        "--audio-envelope-correlation-threshold",
        type=float,
        default=DEFAULT_AUDIO_ENVELOPE_CORRELATION_THRESHOLD,
    )
    parser.add_argument(
        "--audio-silence-rms-threshold",
        type=float,
        default=DEFAULT_AUDIO_SILENCE_RMS_THRESHOLD,
        help=(
            "AC-coupled RMS below this value is treated as silence for both "
            "reference and candidate audio."
        ),
    )
    parser.add_argument(
        "--audio-noise-rms-ratio-min",
        type=float,
        default=DEFAULT_AUDIO_NOISE_RMS_RATIO_MIN,
        help="Minimum candidate/reference RMS ratio for stochastic-noise audio.",
    )
    parser.add_argument(
        "--audio-noise-rms-ratio-max",
        type=float,
        default=DEFAULT_AUDIO_NOISE_RMS_RATIO_MAX,
        help="Maximum candidate/reference RMS ratio for stochastic-noise audio.",
    )
    parser.add_argument(
        "--audio-noise-zcr-min",
        type=float,
        default=DEFAULT_AUDIO_NOISE_ZCR_MIN,
        help="Minimum zero-crossing rate for treating audio as stochastic noise.",
    )
    parser.add_argument(
        "--audio-noise-zcr-ratio-min",
        type=float,
        default=DEFAULT_AUDIO_NOISE_ZCR_RATIO_MIN,
        help="Minimum candidate/reference zero-crossing ratio for stochastic-noise audio.",
    )
    parser.add_argument(
        "--audio-noise-zcr-ratio-max",
        type=float,
        default=DEFAULT_AUDIO_NOISE_ZCR_RATIO_MAX,
        help="Maximum candidate/reference zero-crossing ratio for stochastic-noise audio.",
    )
    parser.add_argument(
        "--allow-missing-candidate-audio",
        action="store_true",
        help="Report missing candidate audio as not_comparable instead of failing.",
    )
    parser.add_argument(
        "--acceptance-mode",
        choices=("all", "visual", "audio"),
        default="all",
        help=(
            "Which comparison axes determine the top-level status. Use visual "
            "for synthetic sprite/explosion clips with intentionally ignored "
            "audio, and audio for synthetic sound-command clips with "
            "intentionally ignored visuals."
        ),
    )
    parser.add_argument(
        "--report-only",
        action="store_true",
        help="Always exit zero after writing reports.",
    )
    return parser.parse_args()


def parse_region(value: str) -> Region:
    try:
        name, body = value.split(":", 1)
        x, y, width, height = (int(part.strip()) for part in body.split(",", 3))
    except ValueError as error:
        raise argparse.ArgumentTypeError(
            f"region must use name:x,y,width,height, got {value!r}"
        ) from error
    if not name or width <= 0 or height <= 0:
        raise argparse.ArgumentTypeError(
            f"region must have a name and positive size, got {value!r}"
        )
    return Region(name, x, y, width, height)


def resolve_tool(explicit: str | None, env_name: str, default: str) -> str:
    requested = explicit or os_environ(env_name) or default
    if Path(requested).is_absolute() or "/" in requested:
        path = Path(requested)
        if path.exists() and path.is_file():
            return str(path)
        raise SystemExit(f"{default} executable was not found at {requested}")
    resolved = shutil.which(requested)
    if resolved:
        return resolved
    raise SystemExit(f"{default} executable {requested!r} was not found")


def os_environ(name: str) -> str | None:
    import os

    return os.environ.get(name)


def build_frame_extract_command(
    ffmpeg: str,
    media: Path,
    output_pattern: Path,
    config: MediaConfig,
) -> list[str]:
    command = [ffmpeg, "-hide_banner", "-loglevel", "error", "-y"]
    if config.audio_start_ms > 0:
        command.extend(["-ss", seconds_arg(config.audio_start_ms)])
    command.extend(["-i", str(media)])
    if config.duration_ms is not None:
        command.extend(["-t", seconds_arg(config.duration_ms)])
    command.extend(
        [
            "-vf",
            (
                f"fps={config.visual_fps},"
                f"scale={config.width}:{config.height}:flags=neighbor"
            ),
            "-frames:v",
            str(config.max_frames),
            str(output_pattern),
        ]
    )
    return command


def build_audio_extract_command(
    ffmpeg: str,
    media: Path,
    output_path: Path,
    config: MediaConfig,
) -> list[str]:
    command = [ffmpeg, "-hide_banner", "-loglevel", "error", "-y"]
    if config.audio_start_ms > 0:
        command.extend(["-ss", seconds_arg(config.audio_start_ms)])
    command.extend(["-i", str(media)])
    if config.duration_ms is not None:
        command.extend(["-t", seconds_arg(config.duration_ms)])
    command.extend(
        [
            "-vn",
            "-ac",
            "1",
            "-ar",
            str(config.audio_sample_rate),
            "-f",
            "f32le",
            str(output_path),
        ]
    )
    return command


def seconds_arg(milliseconds: float) -> str:
    return f"{milliseconds / 1000.0:.6f}"


def extract_frames(ffmpeg: str, media: Path, frame_dir: Path, prefix: str, config: MediaConfig) -> None:
    reset_dir(frame_dir)
    command = build_frame_extract_command(
        ffmpeg, media, frame_dir / f"{prefix}-%05d.png", config
    )
    subprocess.run(command, check=True)


def extract_audio(
    ffmpeg: str,
    media: Path,
    output_path: Path,
    config: MediaConfig,
) -> tuple[bool, str | None]:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    command = build_audio_extract_command(ffmpeg, media, output_path, config)
    completed = subprocess.run(command, check=False, text=True, stderr=subprocess.PIPE)
    if completed.returncode == 0 and output_path.exists() and output_path.stat().st_size > 0:
        return True, None
    return False, completed.stderr.strip() or "ffmpeg did not extract audio samples"


def reset_dir(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def png_paths(frame_dir: Path) -> list[Path]:
    return sorted(frame_dir.glob("*.png"))


def load_rgb(path: Path) -> bytes:
    try:
        from PIL import Image
    except ImportError as error:
        raise SystemExit("Pillow is required to read extracted frames") from error

    with Image.open(path) as image:
        return image.convert("RGB").tobytes()


def compare_visuals(
    reference_dir: Path,
    candidate_dir: Path,
    config: MediaConfig,
    thresholds: Thresholds,
    regions: list[Region],
) -> VisualComparison:
    reference_paths = png_paths(reference_dir)
    candidate_paths = png_paths(candidate_dir)
    compared_frames = min(len(reference_paths), len(candidate_paths))
    full = VisualAggregate("full")
    region_aggregates = [VisualAggregate(region.name) for region in regions]
    failures: list[str] = []
    first_failing_frame: int | None = None
    reference_signatures: list[str] = []
    candidate_signatures: list[str] = []

    if compared_frames == 0:
        failures.append("no shared visual frames were extracted")
        return VisualComparison(
            "fail",
            len(reference_paths),
            len(candidate_paths),
            compared_frames,
            reference_signatures,
            candidate_signatures,
            full,
            region_aggregates,
            first_failing_frame,
            failures,
        )

    for frame_index, (reference_path, candidate_path) in enumerate(
        zip(reference_paths, candidate_paths), start=1
    ):
        reference_rgb = load_rgb(reference_path)
        candidate_rgb = load_rgb(candidate_path)
        reference_signatures.append(f"{zlib.crc32(reference_rgb):08x}")
        candidate_signatures.append(f"{zlib.crc32(candidate_rgb):08x}")
        frame_full = VisualAggregate("frame")
        add_region_diffs(
            frame_full,
            reference_rgb,
            candidate_rgb,
            Region("full", 0, 0, config.width, config.height),
            config,
        )
        add_visual_aggregate(full, frame_full)
        for region, aggregate in zip(regions, region_aggregates):
            add_region_diffs(aggregate, reference_rgb, candidate_rgb, region, config)
        if (
            first_failing_frame is None
            and (frame_full.rms > thresholds.visual_rms or frame_full.mae > thresholds.visual_mae)
        ):
            first_failing_frame = frame_index

    if full.rms > thresholds.visual_rms:
        failures.append(
            f"visual RMS {full.rms:.2f} exceeds threshold {thresholds.visual_rms:.2f}"
        )
    if full.mae > thresholds.visual_mae:
        failures.append(
            f"visual MAE {full.mae:.2f} exceeds threshold {thresholds.visual_mae:.2f}"
        )

    status = "pass" if not failures else "fail"
    return VisualComparison(
        status,
        len(reference_paths),
        len(candidate_paths),
        compared_frames,
        reference_signatures[:5],
        candidate_signatures[:5],
        full,
        region_aggregates,
        first_failing_frame,
        failures,
    )


def add_region_diffs(
    aggregate: VisualAggregate,
    reference_rgb: bytes,
    candidate_rgb: bytes,
    region: Region,
    config: MediaConfig,
) -> None:
    clamped = region.clamped(config.width, config.height)
    stride = max(1, config.pixel_stride)
    for y in range(clamped.y, clamped.y + clamped.height, stride):
        row_offset = y * config.width * RGB_CHANNELS
        for x in range(clamped.x, clamped.x + clamped.width, stride):
            offset = row_offset + x * RGB_CHANNELS
            for channel in range(RGB_CHANNELS):
                diff = reference_rgb[offset + channel] - candidate_rgb[offset + channel]
                aggregate.add_diff(diff)


def add_visual_aggregate(target: VisualAggregate, source: VisualAggregate) -> None:
    target.channel_count += source.channel_count
    target.sum_squared += source.sum_squared
    target.sum_abs += source.sum_abs
    target.max_abs = max(target.max_abs, source.max_abs)


def read_float32_samples(path: Path) -> list[float]:
    raw = path.read_bytes()
    usable = len(raw) - (len(raw) % FLOAT32_BYTES)
    if usable == 0:
        return []
    return list(struct.unpack(f"<{usable // FLOAT32_BYTES}f", raw[:usable]))


def compare_audio_files(
    reference_audio: Path,
    candidate_audio: Path,
    config: MediaConfig,
    thresholds: Thresholds,
) -> AudioComparison:
    reference = read_float32_samples(reference_audio)
    candidate = read_float32_samples(candidate_audio)
    return compare_audio_samples(reference, candidate, config, thresholds)


def compare_audio_samples(
    reference: list[float],
    candidate: list[float],
    config: MediaConfig,
    thresholds: Thresholds,
) -> AudioComparison:
    compared_samples = min(len(reference), len(candidate))
    comparison = AudioComparison(
        status="fail",
        reference_samples=len(reference),
        candidate_samples=len(candidate),
        compared_samples=compared_samples,
    )
    if compared_samples == 0:
        comparison.failures.append("no shared audio samples were extracted")
        return comparison

    reference = reference[:compared_samples]
    candidate = candidate[:compared_samples]
    comparison.reference_dc_offset = mean(reference)
    comparison.candidate_dc_offset = mean(candidate)
    reference = remove_dc_offset(reference, comparison.reference_dc_offset)
    candidate = remove_dc_offset(candidate, comparison.candidate_dc_offset)
    comparison.reference_rms = rms(reference)
    comparison.candidate_rms = rms(candidate)
    diff = [left - right for left, right in zip(reference, candidate)]
    comparison.diff_rms = rms(diff)
    comparison.normalized_diff_rms = comparison.diff_rms / max(comparison.reference_rms, 1e-9)
    comparison.reference_peak = peak(reference)
    comparison.candidate_peak = peak(candidate)
    comparison.correlation = correlation(reference, candidate)
    comparison.envelope_correlation = correlation(
        envelope(reference, config), envelope(candidate, config)
    )
    comparison.reference_zero_crossing_rate = zero_crossing_rate(reference)
    comparison.candidate_zero_crossing_rate = zero_crossing_rate(candidate)
    comparison.rms_ratio = comparison.candidate_rms / max(comparison.reference_rms, 1e-9)
    comparison.zero_crossing_ratio = comparison.candidate_zero_crossing_rate / max(
        comparison.reference_zero_crossing_rate, 1e-9
    )

    if (
        comparison.reference_rms <= thresholds.audio_silence_rms
        and comparison.candidate_rms <= thresholds.audio_silence_rms
    ):
        comparison.normalized_diff_rms = 0.0
        comparison.correlation = 1.0
        comparison.envelope_correlation = 1.0
        comparison.status = "pass"
        return comparison

    if comparison.normalized_diff_rms > thresholds.audio_nrms:
        comparison.failures.append(
            "audio normalized diff RMS "
            f"{comparison.normalized_diff_rms:.3f} exceeds threshold "
            f"{thresholds.audio_nrms:.3f}"
        )
    if comparison.correlation < thresholds.audio_correlation:
        comparison.failures.append(
            f"audio correlation {comparison.correlation:.3f} is below threshold "
            f"{thresholds.audio_correlation:.3f}"
        )
    if comparison.envelope_correlation < thresholds.audio_envelope_correlation:
        comparison.failures.append(
            "audio envelope correlation "
            f"{comparison.envelope_correlation:.3f} is below threshold "
            f"{thresholds.audio_envelope_correlation:.3f}"
        )

    if stochastic_noise_gate_passes(comparison, thresholds):
        comparison.noise_like_pass = True
        comparison.failures = [
            failure
            for failure in comparison.failures
            if not (
                failure.startswith("audio normalized diff RMS")
                or failure.startswith("audio correlation")
            )
        ]

    comparison.status = "pass" if not comparison.failures else "fail"
    return comparison


def mean(samples: list[float]) -> float:
    if not samples:
        return 0.0
    return sum(samples) / len(samples)


def remove_dc_offset(samples: list[float], dc_offset: float) -> list[float]:
    return [sample - dc_offset for sample in samples]


def rms(samples: list[float]) -> float:
    if not samples:
        return 0.0
    return math.sqrt(sum(sample * sample for sample in samples) / len(samples))


def peak(samples: list[float]) -> float:
    return max((abs(sample) for sample in samples), default=0.0)


def correlation(left: list[float], right: list[float]) -> float:
    count = min(len(left), len(right))
    if count == 0:
        return 0.0
    left = left[:count]
    right = right[:count]
    left_mean = sum(left) / count
    right_mean = sum(right) / count
    numerator = 0.0
    left_energy = 0.0
    right_energy = 0.0
    for left_sample, right_sample in zip(left, right):
        left_delta = left_sample - left_mean
        right_delta = right_sample - right_mean
        numerator += left_delta * right_delta
        left_energy += left_delta * left_delta
        right_energy += right_delta * right_delta
    denominator = math.sqrt(left_energy * right_energy)
    if denominator <= 1e-12:
        return 1.0 if left_energy <= 1e-12 and right_energy <= 1e-12 else 0.0
    return numerator / denominator


def envelope(samples: list[float], config: MediaConfig) -> list[float]:
    window = max(1, int(config.audio_sample_rate * config.audio_window_ms / 1000))
    return [
        sum(abs(sample) for sample in samples[index : index + window])
        / len(samples[index : index + window])
        for index in range(0, len(samples), window)
        if samples[index : index + window]
    ]


def zero_crossing_rate(samples: list[float]) -> float:
    if len(samples) < 2:
        return 0.0
    crossings = 0
    previous_positive = samples[0] >= 0.0
    for sample in samples[1:]:
        positive = sample >= 0.0
        if positive != previous_positive:
            crossings += 1
        previous_positive = positive
    return crossings / (len(samples) - 1)


def stochastic_noise_gate_passes(
    comparison: AudioComparison, thresholds: Thresholds
) -> bool:
    """Accept noise sounds by acoustic shape when exact random phase is unstable."""
    if comparison.envelope_correlation < thresholds.audio_envelope_correlation:
        return False
    if (
        comparison.reference_zero_crossing_rate < thresholds.audio_noise_zcr_min
        or comparison.candidate_zero_crossing_rate < thresholds.audio_noise_zcr_min
    ):
        return False
    if not (
        thresholds.audio_noise_rms_ratio_min
        <= comparison.rms_ratio
        <= thresholds.audio_noise_rms_ratio_max
    ):
        return False
    return (
        thresholds.audio_noise_zcr_ratio_min
        <= comparison.zero_crossing_ratio
        <= thresholds.audio_noise_zcr_ratio_max
    )


def write_reports(
    out_dir: Path,
    config: MediaConfig,
    reference_config: MediaConfig,
    candidate_config: MediaConfig,
    thresholds: Thresholds,
    reference_video: Path,
    reference_audio: Path | None,
    candidate_media: Path,
    candidate_audio: Path | None,
    visual: VisualComparison,
    audio: AudioComparison,
    acceptance_mode: str,
) -> str:
    out_dir.mkdir(parents=True, exist_ok=True)
    overall_status = overall_status_for_acceptance(visual, audio, acceptance_mode)
    report = {
        "status": overall_status,
        "acceptance_mode": acceptance_mode,
        "reference_video": str(reference_video),
        "reference_audio": str(reference_audio) if reference_audio else None,
        "candidate_media": str(candidate_media),
        "candidate_audio": str(candidate_audio) if candidate_audio else None,
        "config": config.__dict__,
        "reference_config": reference_config.__dict__,
        "candidate_config": candidate_config.__dict__,
        "thresholds": thresholds.__dict__,
        "visual": visual.to_report(),
        "audio": audio.to_report(),
    }
    (out_dir / "report.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_visual_tsv(out_dir / "visual_metrics.tsv", visual)
    write_audio_tsv(out_dir / "audio_metrics.tsv", audio)
    print(json.dumps(report, indent=2, sort_keys=True))
    return overall_status


def overall_status_for_acceptance(
    visual: VisualComparison, audio: AudioComparison, acceptance_mode: str
) -> str:
    if acceptance_mode == "visual":
        return "pass" if visual.status == "pass" else "fail"
    if acceptance_mode == "audio":
        return "pass" if audio.status == "pass" else "fail"
    return "pass" if visual.status == "pass" and audio.status == "pass" else "fail"


def write_visual_tsv(path: Path, visual: VisualComparison) -> None:
    rows = ["region\tsampled_channels\trms\tmae\tmax_abs"]
    rows.append(visual_row(visual.full))
    rows.extend(visual_row(region) for region in visual.regions)
    path.write_text("\n".join(rows) + "\n", encoding="utf-8")


def visual_row(aggregate: VisualAggregate) -> str:
    return (
        f"{aggregate.name}\t{aggregate.channel_count}\t{aggregate.rms:.6f}\t"
        f"{aggregate.mae:.6f}\t{aggregate.max_abs}"
    )


def write_audio_tsv(path: Path, audio: AudioComparison) -> None:
    rows = ["metric\tvalue"]
    for key, value in audio.to_report().items():
        if isinstance(value, list):
            value = "; ".join(value)
        rows.append(f"{key}\t{value}")
    path.write_text("\n".join(rows) + "\n", encoding="utf-8")


def stream_configs(
    config: MediaConfig,
    reference_start_ms: float | None,
    candidate_start_ms: float | None,
) -> tuple[MediaConfig, MediaConfig]:
    reference_start = (
        config.audio_start_ms if reference_start_ms is None else max(0.0, reference_start_ms)
    )
    candidate_start = (
        config.audio_start_ms if candidate_start_ms is None else max(0.0, candidate_start_ms)
    )
    return (
        replace(config, audio_start_ms=reference_start),
        replace(config, audio_start_ms=candidate_start),
    )


def main() -> int:
    args = parse_args()
    reference_video = Path(args.reference_video)
    reference_audio = Path(args.reference_audio) if args.reference_audio else None
    candidate_media = Path(args.candidate_media)
    candidate_audio = Path(args.candidate_audio) if args.candidate_audio else None
    if not reference_video.is_file():
        raise SystemExit(f"reference video does not exist: {reference_video}")
    if reference_audio is not None and not reference_audio.is_file():
        raise SystemExit(f"reference audio does not exist: {reference_audio}")
    if not candidate_media.is_file():
        raise SystemExit(f"candidate media does not exist: {candidate_media}")
    if candidate_audio is not None and not candidate_audio.is_file():
        raise SystemExit(f"candidate audio does not exist: {candidate_audio}")

    ffmpeg = resolve_tool(args.ffmpeg, "FFMPEG", "ffmpeg")
    out_dir = Path(args.out_dir)
    config = MediaConfig(
        width=max(1, args.width),
        height=max(1, args.height),
        visual_fps=max(0.001, args.visual_fps),
        max_frames=max(1, args.max_frames),
        pixel_stride=max(1, args.pixel_stride),
        audio_sample_rate=max(1, args.audio_sample_rate),
        audio_window_ms=max(1, args.audio_window_ms),
        audio_start_ms=max(0.0, args.audio_start_ms),
        duration_ms=args.duration_ms,
    )
    thresholds = Thresholds(
        visual_rms=args.visual_rms_threshold,
        visual_mae=args.visual_mae_threshold,
        audio_nrms=args.audio_nrms_threshold,
        audio_correlation=args.audio_correlation_threshold,
        audio_envelope_correlation=args.audio_envelope_correlation_threshold,
        audio_silence_rms=max(0.0, args.audio_silence_rms_threshold),
        audio_noise_rms_ratio_min=max(0.0, args.audio_noise_rms_ratio_min),
        audio_noise_rms_ratio_max=max(0.0, args.audio_noise_rms_ratio_max),
        audio_noise_zcr_min=max(0.0, args.audio_noise_zcr_min),
        audio_noise_zcr_ratio_min=max(0.0, args.audio_noise_zcr_ratio_min),
        audio_noise_zcr_ratio_max=max(0.0, args.audio_noise_zcr_ratio_max),
    )
    regions = [parse_region(region) for region in args.region] or list(DEFAULT_REGIONS)
    reference_config, candidate_config = stream_configs(
        config,
        args.reference_start_ms,
        args.candidate_start_ms,
    )

    reference_frame_dir = out_dir / "frames" / "reference"
    candidate_frame_dir = out_dir / "frames" / "candidate"
    extract_frames(ffmpeg, reference_video, reference_frame_dir, "reference", reference_config)
    extract_frames(ffmpeg, candidate_media, candidate_frame_dir, "candidate", candidate_config)
    visual = compare_visuals(reference_frame_dir, candidate_frame_dir, config, thresholds, regions)

    audio_dir = out_dir / "audio"
    reference_audio_path = audio_dir / "reference.f32"
    candidate_audio_path = audio_dir / "candidate.f32"
    reference_audio_source = reference_audio or reference_video
    reference_audio_ok, reference_audio_error = extract_audio(
        ffmpeg, reference_audio_source, reference_audio_path, reference_config
    )
    audio_source = candidate_audio or candidate_media
    candidate_audio_ok, candidate_audio_error = extract_audio(
        ffmpeg, audio_source, candidate_audio_path, candidate_config
    )
    if not reference_audio_ok:
        audio = AudioComparison(
            status="fail",
            failures=["reference video audio could not be extracted"],
            extraction_error=reference_audio_error,
        )
    elif not candidate_audio_ok:
        status = "not_comparable" if args.allow_missing_candidate_audio else "fail"
        failures = ["candidate audio could not be extracted"]
        if args.allow_missing_candidate_audio:
            failures = []
        audio = AudioComparison(
            status=status,
            failures=failures,
            extraction_error=candidate_audio_error,
        )
    else:
        audio = compare_audio_files(reference_audio_path, candidate_audio_path, config, thresholds)

    overall_status = write_reports(
        out_dir,
        config,
        reference_config,
        candidate_config,
        thresholds,
        reference_video,
        reference_audio,
        candidate_media,
        candidate_audio,
        visual,
        audio,
        args.acceptance_mode,
    )
    failed = overall_status != "pass"
    return 0 if args.report_only or not failed else 1


if __name__ == "__main__":
    sys.exit(main())
