# Playing Defender

Download the archive for your computer from [GitHub Releases](https://github.com/stephenlclarke/defender/releases). Rust, ROMs and external assets are not needed.

| Computer | Archive suffix | Executable |
| --- | --- | --- |
| Apple Silicon Mac (ARM64), macOS 11 or later | `aarch64-apple-darwin.tar.gz` | `defender` |
| Intel/AMD 64-bit Windows 10 or later | `x86_64-pc-windows-msvc.zip` | `defender.exe` |

Extract the archive first. On macOS, open Terminal in the extracted folder and run `./defender`. On Windows, run `defender.exe`, or open PowerShell in that folder and run `.\defender.exe`. Run with `--mute` to disable audio.

The binaries are unsigned and the macOS binary is not notarized. If macOS blocks it, use System Settings > Privacy & Security > Open Anyway after verifying the download. Windows may show a SmartScreen warning for an unsigned download.

## Controls

Press `1` for one player or `2` for two players. No coin input is required.

| Key | Action |
| --- | --- |
| `A` / `Z` | Move up / down |
| `SHIFT` | Reverse direction |
| `SPACE` | Thrust |
| `ENTER` | Fire |
| `TAB` | Smart bomb |
| `H` | Hyperspace |
| `BACKSPACE` | Delete a high-score initial |
| `ESC` | Quit |

Type `XYZZY` to toggle extra mode. In this mode, `F` toggles automatic fire, `G` toggles god mode, smart bombs are unlimited, and the four-shot laser cap is removed.

## Graphics And Audio

The game uses WGPU for rendering, Winit for its window, and CPAL for audio. It needs a working graphics driver and a compatible GPU. macOS uses Metal; Windows normally uses Direct3D 12, with Vulkan available where supported. For launch problems, try `--mute` and include your OS, GPU, driver and release filename in a [bug report](https://github.com/stephenlclarke/defender/issues).

## Verify A Download

Each archive has a matching `.sha256` file. On macOS run `shasum -a 256 -c <archive>.sha256` from the download folder. On Windows run `Get-FileHash .\<archive>.zip -Algorithm SHA256` and compare the hash with the text in the matching `.sha256` file.
