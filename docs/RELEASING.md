# Desktop Releases

The [Desktop binaries workflow](../.github/workflows/release.yml) builds the production WGPU executable using `Cargo.lock` and the stable Rust toolchain.

| Platform | Rust target | GitHub runner | Package |
| --- | --- | --- | --- |
| macOS ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `macos-15` | `.tar.gz` |
| Windows x86-64 (Intel/AMD) | `x86_64-pc-windows-msvc` | `windows-2022` | `.zip` |

The macOS deployment target is 11.0. Windows builds statically link the MSVC runtime. Binaries are unsigned; macOS notarization and installers are not configured. Intel Macs, Windows ARM64 and 32-bit Windows are not release targets. Linux remains a CI validation platform.

## Build Checks And Downloads

Pushes and pull requests to `main` and `develop`, version tag pushes, and manual workflow dispatches build both targets. Each job runs the Rust tests, builds the optimized executable, and runs that executable's help and actor/attract smoke commands outside the checkout. The macOS job also runs real offscreen WGPU readback; Linux CI retains its WGPU smoke checks. The Windows job verifies compilation and actor behavior; a hardware graphics/audio playtest remains necessary on Windows.

Every successful build uploads an artifact named `defender-<target>`, retained for 14 days. Download it from the workflow run's **Artifacts** section. Branch/manual builds use `dev-<commit>` in archive filenames. Tagged builds use the tag. Artifacts contain an archive and its SHA-256 sidecar.

## Publish A Release

Merge the reviewed changes and wait for CI to pass. Set the package version in `Cargo.toml` and update `Cargo.lock` before tagging a new version. For example, for package version `0.1.0`:

```sh
git tag -a v0.1.0 -m "Defender v0.1.0"
git push origin v0.1.0
```

The tag triggers fresh native builds. After both jobs pass, the publish job verifies both checksums, creates a draft GitHub Release with generated notes, uploads both packages and checksums, then publishes it. Only that tag-triggered job has `contents: write`; pull request builds cannot publish. Rerunning a tag workflow replaces that tag's assets, so published tags must remain immutable. Manual dispatch builds downloadable artifacts but does not publish a release.

Public release downloads live at [GitHub Releases](https://github.com/stephenlclarke/defender/releases). Each archive contains the executable, [player instructions](PLAYING.md) as `README.md`, and `SUPPORT.md`. Sprites, audio tables and default scripts are embedded; ROMs and external asset trees are not distributed.

The initial release is [v0.1.0](https://github.com/stephenlclarke/defender/releases/tag/v0.1.0), matching the package version. For subsequent releases, increment the package version and tag rather than moving `v0.1.0`. Update version-specific download links in `README.md` and `docs/PLAYING.md` with each release.

## Local Packaging

On an Apple Silicon Mac:

```sh
rustup target add aarch64-apple-darwin
make release-package RELEASE_TARGET=aarch64-apple-darwin RELEASE_VERSION=dev-local
```

On Windows, use an MSVC Rust installation and Visual Studio C++ Build Tools. In PowerShell:

```powershell
rustup target add x86_64-pc-windows-msvc
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --locked --release --bin defender --target x86_64-pc-windows-msvc
python scripts/package_release.py --target x86_64-pc-windows-msvc --version dev-local
```

Packages are written to ignored `dist/`. The packager checks the executable's architecture before archiving it. `make release-tools-test` runs its tests and `actionlint`; install Python 3 and actionlint to run this target. CI installs these tools before running the checks.
