#!/usr/bin/env python3
"""Generate ROM-derived Defender branding assets.

This script reconstructs the first attract page from the Williams red-label
source release:

- `LGOTAB` / `LOGO` in `amode1.src` for the Williams script mark
- `DEFDAT` / `DEFNNN` in `amode1.src` for the Defender wordmark
- `CPRTAB` in `amode1.src` for the copyright line
- `ELECTR` / `PRESEN` in `mess0.src` for the text content

The runtime only embeds the generated PNGs from `assets/arcade/`, so compile
and runtime stay self-contained after this script has been run.
"""

from __future__ import annotations

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
AMODE_PATH = Path("/tmp/defender-src/src/amode1.src")
FONT_SHEET_PATH = ROOT / "assets/arcade/font-sheet.png"
OUTPUT_DIR = ROOT / "assets/arcade"

DISPLAY_WIDTH = 320
DISPLAY_HEIGHT = 256
NATIVE_WIDTH = 256
NATIVE_HEIGHT = 256

WILLIAMS_RED = (237, 42, 47, 255)
TITLE_GOLD = (241, 182, 57, 255)
DEFENDER_FACE = (112, 255, 52, 255)
DEFENDER_SHADOW = (255, 48, 48, 255)

FONT_LAYOUT: list[tuple[str, int]] = [
    (" ", 2),
    ("!", 2),
    (",", 2),
    (".", 2),
    ("0", 6),
    ("1", 6),
    ("2", 6),
    ("3", 6),
    ("4", 6),
    ("5", 6),
    ("6", 6),
    ("7", 6),
    ("8", 6),
    ("9", 6),
    (":", 2),
    ("?", 6),
    ("A", 6),
    ("B", 6),
    ("C", 6),
    ("D", 6),
    ("E", 6),
    ("F", 6),
    ("G", 6),
    ("H", 6),
    ("I", 4),
    ("J", 6),
    ("K", 6),
    ("L", 6),
    ("M", 8),
    ("N", 6),
    ("O", 6),
    ("P", 6),
    ("Q", 6),
    ("R", 6),
    ("S", 6),
    ("T", 6),
    ("U", 6),
    ("V", 6),
    ("W", 8),
    ("X", 6),
    ("Y", 6),
    ("Z", 6),
    ("-", 6),
    ("/", 6),
]
FONT_MAP = {glyph: index for index, (glyph, _) in enumerate(FONT_LAYOUT)}
FONT_COLUMNS = 8
GLYPH_CELL_WIDTH = 8
GLYPH_CELL_HEIGHT = 8

FCB_SYMBOLS = {
    "NULL": 0xFF,
    "HYPERC": 0xFE,
    "QUIT": 0xFD,
}


def main() -> None:
    amode_lines = AMODE_PATH.read_text().splitlines()
    font_sheet = Image.open(FONT_SHEET_PATH).convert("RGBA")

    williams_logo = decode_williams_logo(amode_lines)
    defender_logo = decode_defender_logo(amode_lines)
    copyright_line = decode_copyright(amode_lines)
    logo_page = compose_logo_page(font_sheet, williams_logo, defender_logo, copyright_line)

    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    scale_display_aspect(williams_logo).save(OUTPUT_DIR / "williams-logo.png")
    scale_display_aspect(defender_logo).save(OUTPUT_DIR / "defender-logo.png")
    scale_display_aspect(copyright_line).save(OUTPUT_DIR / "copyright-1980.png")
    logo_page.save(OUTPUT_DIR / "logo-page.png")


def compose_logo_page(
    font_sheet: Image.Image,
    williams_logo: Image.Image,
    defender_logo: Image.Image,
    copyright_line: Image.Image,
) -> Image.Image:
    native = Image.new("RGBA", (NATIVE_WIDTH, NATIVE_HEIGHT), (0, 0, 0, 255))
    native.alpha_composite(williams_logo)
    render_text(font_sheet, native, 0x32 * 2, 0x58, "ELECTRONICS INC.", TITLE_GOLD)
    render_text(font_sheet, native, (0x32 + 12) * 2, 0x58 + 20, "PRESENTS", TITLE_GOLD)
    native.alpha_composite(defender_logo, (0x30 * 2, 0x90))
    native.alpha_composite(copyright_line, (0x3B * 2, 0xD0))
    return native.resize((DISPLAY_WIDTH, DISPLAY_HEIGHT), Image.Resampling.NEAREST)


def decode_williams_logo(amode_lines: list[str]) -> Image.Image:
    data = parse_fcb_table(amode_lines, "LGOTAB")
    image = Image.new("RGBA", (NATIVE_WIDTH, NATIVE_HEIGHT), (0, 0, 0, 0))
    index = 0
    cursor_x = 0
    cursor_y = 0
    while index < len(data):
        value = data[index]
        index += 1
        if value <= 0xAA:
            accumulator = value
            while True:
                carry = accumulator & 0x80
                accumulator = (accumulator << 1) & 0xFF
                if carry:
                    cursor_x -= 1

                carry = accumulator & 0x80
                accumulator = (accumulator << 1) & 0xFF
                if carry:
                    cursor_x += 1

                carry = accumulator & 0x80
                accumulator = (accumulator << 1) & 0xFF
                if carry:
                    cursor_y -= 1

                carry = accumulator & 0x80
                accumulator = (accumulator << 1) & 0xFF
                if carry:
                    cursor_y += 1

                if 0 <= cursor_x < NATIVE_WIDTH and 0 <= cursor_y < NATIVE_HEIGHT:
                    image.putpixel((cursor_x, cursor_y), WILLIAMS_RED)

                if accumulator == 0:
                    break
        else:
            instruction = ((~value) & 0xFF) - 1
            if instruction < 0:
                continue
            if instruction != 0:
                break
            cursor_x = data[index]
            cursor_y = data[index + 1]
            index += 2

    return image.crop(image.getbbox())


def decode_defender_logo(amode_lines: list[str]) -> Image.Image:
    data = parse_fcb_table(amode_lines, "DEFDAT", "DEFDEN")
    columns = 60
    rows = 24
    buffer = [0] * (columns * rows)
    color_table = [None, 0x22, 0xCC, 0x00]
    cursor = 0
    odd_nibble = 0
    run_length = 0

    for byte in data:
        for nibble in (byte >> 4, byte & 0xF):
            if nibble & 0xC:
                run_length = (nibble & 0x3) + run_length
                color = color_table[(nibble & 0xC) >> 2]
                if cursor >= columns * rows:
                    cursor = cursor + 1 - columns * rows

                if odd_nibble:
                    if cursor < len(buffer):
                        buffer[cursor] = (buffer[cursor] & 0xF0) | (color & 0x0F)
                    cursor += rows
                    run_length -= 1
                    if run_length < 0:
                        odd_nibble = 0
                        run_length = 0
                        continue
                else:
                    odd_nibble = 0xFF

                while run_length >= 0:
                    if cursor < len(buffer):
                        buffer[cursor] = color
                    cursor += rows
                    run_length -= 1

                odd_nibble = 0
                run_length = 0
            else:
                run_length = (nibble + run_length) * 4

    image = Image.new("RGBA", (columns * 2, rows), (0, 0, 0, 0))
    for x_byte in range(columns):
        for y in range(rows):
            packed = buffer[x_byte * rows + y]
            left = (packed >> 4) & 0x0F
            right = packed & 0x0F
            if left:
                image.putpixel(
                    (x_byte * 2, y),
                    DEFENDER_SHADOW if left == 0x2 else DEFENDER_FACE,
                )
            if right:
                image.putpixel(
                    (x_byte * 2 + 1, y),
                    DEFENDER_SHADOW if right == 0x2 else DEFENDER_FACE,
                )
    return image


def decode_copyright(amode_lines: list[str]) -> Image.Image:
    data = parse_fcb_table(amode_lines, "CPRTAB", "CPREND")
    image = Image.new("RGBA", ((len(data) // 2) * 2, 8), (0, 0, 0, 0))
    for column in range(len(data) // 2):
        left = data[column * 2]
        right = data[column * 2 + 1]
        mask = 1
        for y in range(8):
            if left & mask:
                image.putpixel((column * 2, y), TITLE_GOLD)
            if right & mask:
                image.putpixel((column * 2 + 1, y), TITLE_GOLD)
            mask <<= 1
    return image


def render_text(
    font_sheet: Image.Image,
    destination: Image.Image,
    x: int,
    y: int,
    text: str,
    color: tuple[int, int, int, int],
) -> None:
    pen_x = x
    for character in text:
        glyph_image, advance = glyph(font_sheet, character)
        for glyph_y in range(glyph_image.height):
            for glyph_x in range(glyph_image.width):
                if glyph_image.getpixel((glyph_x, glyph_y))[3]:
                    destination.putpixel((pen_x + glyph_x, y + glyph_y), color)
        pen_x += advance + 1


def glyph(font_sheet: Image.Image, character: str) -> tuple[Image.Image, int]:
    character = character.upper()
    index = FONT_MAP.get(character, FONT_MAP["?"])
    column = index % FONT_COLUMNS
    row = index // FONT_COLUMNS
    image = font_sheet.crop(
        (
            column * GLYPH_CELL_WIDTH,
            row * GLYPH_CELL_HEIGHT,
            column * GLYPH_CELL_WIDTH + GLYPH_CELL_WIDTH,
            row * GLYPH_CELL_HEIGHT + GLYPH_CELL_HEIGHT,
        )
    )
    return image, FONT_LAYOUT[index][1]


def parse_fcb_table(lines: list[str], start_label: str, end_label: str | None = None) -> list[int]:
    start = next(index for index, line in enumerate(lines) if line.startswith(start_label))
    end = len(lines)
    if end_label is not None:
        end = next(index for index, line in enumerate(lines[start + 1 :], start + 1) if line.startswith(end_label))

    values: list[int] = []
    for line in lines[start:end]:
        stripped = line.split(";", 1)[0].strip()
        if "FCB" not in stripped:
            continue
        rhs = stripped.split("FCB", 1)[1]
        for token in (piece.strip() for piece in rhs.split(",")):
            if token:
                values.append(parse_token(token))
    return values


def parse_token(token: str) -> int:
    if token in FCB_SYMBOLS:
        return FCB_SYMBOLS[token]
    if token.startswith("$"):
        return int(token[1:], 16)
    return int(token)


def scale_display_aspect(image: Image.Image) -> Image.Image:
    return image.resize((image.width * 5 // 4, image.height), Image.Resampling.NEAREST)


if __name__ == "__main__":
    main()
