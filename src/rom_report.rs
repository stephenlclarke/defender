//! Runtime-owned ROM report command facade.

use std::path::Path;

use anyhow::Context;

pub(crate) fn run(path: Option<&Path>) -> anyhow::Result<()> {
    let Some(path) = path else {
        print!("{}", listing_text());
        return Ok(());
    };

    let report = crate::rom::scan_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?;
    print!("{}", report_text(&report));

    Ok(())
}

fn listing_text() -> String {
    let descriptors = crate::rom::expected_roms();
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

fn report_text(report: &crate::rom::RomReport) -> String {
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
    use std::path::PathBuf;

    use super::{listing_text, report_text};

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
        let report = crate::rom::RomReport {
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
}
