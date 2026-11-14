# Desktop Binary Downloads

## Request Details

Verify that the production game renders through WGPU and provide GitHub Actions builds and GitHub Release downloads for Apple Silicon macOS and Intel/AMD 64-bit Windows. The owner confirmed Windows x86-64 only. Existing workflows check Linux and leaks but do not package native executable downloads.

## Alternatives Considered

Cargo installation remains available. Prebuilt archives let players run the embedded-asset game without installing Rust or downloading ROMs. Native runners exercise each platform's executable.

## Contribution Checks

This task follows the owner's request. No GitHub issue number has been assigned. Implementation and validation are recorded in [PR-10.md](PR-10.md).
