//! Runtime-owned ROM report command facade.

use std::path::Path;

use anyhow::{Context, anyhow};

use crate::roms::{RomScanReport, VerifiedRomSummary};

pub(crate) fn run(path: Option<&Path>) -> anyhow::Result<()> {
    let Some(path) = path else {
        print!("{}", listing_text());
        return Ok(());
    };

    let report = crate::roms::scan_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?;
    print!("{}", report_text(&report));

    Ok(())
}

pub(crate) fn run_verify(path: &Path) -> anyhow::Result<()> {
    let summary = crate::roms::verify_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?
        .map_err(|report| anyhow!("{}", report_text(&report).trim_end()))?;

    print!("{}", verification_text(path, summary));

    Ok(())
}

fn listing_text() -> String {
    let descriptors = crate::roms::expected_roms();
    let mut text = format!(
        "Expected Williams Defender red-label ROM filenames ({} files):\n",
        descriptors.len()
    );
    for descriptor in descriptors {
        text.push_str(&format!(
            "  {:<24} {:>5} bytes crc {}\n",
            descriptor.name, descriptor.size, descriptor.crc32
        ));
    }
    text.push('\n');
    text.push_str("The runtime is self-contained; ROM files are only used for verification.\n");
    text.push_str("Pass a directory to compare against a local ROM set:\n");
    text.push_str("  defender --rom-report /path/to/roms\n");
    text.push_str("  defender --verify-roms /path/to/roms\n");
    text
}

fn verification_text(path: &Path, summary: VerifiedRomSummary) -> String {
    format!(
        "ROM set {} verified: {} files, {} bytes, {} mapped regions, {} mapped loads\n",
        path.display(),
        summary.file_count,
        summary.total_bytes,
        summary.region_count,
        summary.load_count
    )
}

fn report_text(report: &RomScanReport) -> String {
    let mut text = format!("{}\n", report.summary_line());
    if !report.missing.is_empty() {
        text.push_str(&format!("Missing: {}\n", report.missing.join(", ")));
    }
    if !report.wrong_size.is_empty() {
        text.push_str(&format!("Wrong size: {}\n", report.wrong_size.join(", ")));
    }
    if !report.wrong_crc.is_empty() {
        text.push_str(&format!("Wrong CRC: {}\n", report.wrong_crc.join(", ")));
    }
    if !report.unexpected.is_empty() {
        text.push_str(&format!("Unexpected: {}\n", report.unexpected.join(", ")));
    }
    text
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::roms::{RomScanReport, VerifiedRomSummary};

    use super::{listing_text, report_text, run, run_verify, verification_text};

    #[test]
    fn listing_text_preserves_current_contract() {
        let text = listing_text();

        assert!(text.starts_with("Expected Williams Defender red-label ROM filenames (14 files):"));
        assert!(text.contains("defend.1"));
        assert!(text.contains("decoder.3"));
        assert!(text.contains("crc c3e52d7e"));
        assert!(text.contains("ROM files are only used for verification"));
        assert!(text.contains("defender --rom-report /path/to/roms"));
        assert!(text.contains("defender --verify-roms /path/to/roms"));
    }

    #[test]
    fn report_text_preserves_current_sections() {
        let report = RomScanReport {
            directory: PathBuf::from("roms"),
            expected: 2,
            found: vec![String::from("defend.1")],
            missing: vec![String::from("defend.2")],
            unexpected: vec![String::from("extra.bin")],
            wrong_size: vec![String::from("defend.1 expected 2048 bytes got 1")],
            wrong_crc: vec![String::from("defend.1 expected c3e52d7e got 00000000")],
        };

        let text = report_text(&report);

        assert!(text.contains("ROM set roms: 1/2 expected files present, 0/1 CRCs verified"));
        assert!(text.contains("Missing: defend.2"));
        assert!(text.contains("Wrong size: defend.1 expected 2048 bytes got 1"));
        assert!(text.contains("Wrong CRC: defend.1 expected c3e52d7e got 00000000"));
        assert!(text.contains("Unexpected: extra.bin"));
    }

    #[test]
    fn verification_text_preserves_current_success_contract() {
        assert_eq!(
            verification_text(
                Path::new("roms"),
                VerifiedRomSummary {
                    file_count: 1,
                    total_bytes: 2,
                    region_count: 1,
                    load_count: 1,
                },
            ),
            "ROM set roms verified: 1 files, 2 bytes, 1 mapped regions, 1 mapped loads\n"
        );
    }

    #[test]
    fn run_accepts_complete_rom_fixture_report_path() {
        run(Some(Path::new("assets/roms/defender")))
            .expect("complete checked-in ROM fixture should report");
    }

    #[test]
    fn run_verify_accepts_complete_rom_fixture() {
        run_verify(Path::new("assets/roms/defender"))
            .expect("complete checked-in ROM fixture should verify");
    }
}
