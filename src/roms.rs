//! Runtime-facing optional ROM verification facade.
//!
//! ROM files are not needed for normal play. This module keeps optional
//! verification commands on clean report contracts while the metadata and
//! loader still live in the legacy evidence tree.

use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RomDescriptor {
    pub(crate) name: &'static str,
    pub(crate) size: u64,
    pub(crate) crc32: &'static str,
}

impl From<crate::rom::RomDescriptor> for RomDescriptor {
    fn from(descriptor: crate::rom::RomDescriptor) -> Self {
        Self {
            name: descriptor.name,
            size: descriptor.size,
            crc32: descriptor.crc32,
        }
    }
}

pub(crate) fn expected_roms() -> Vec<RomDescriptor> {
    crate::rom::expected_roms()
        .iter()
        .copied()
        .map(RomDescriptor::from)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RomScanReport {
    pub(crate) directory: PathBuf,
    pub(crate) expected: usize,
    pub(crate) found: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) unexpected: Vec<String>,
    pub(crate) wrong_size: Vec<String>,
    pub(crate) wrong_crc: Vec<String>,
}

impl RomScanReport {
    pub(crate) fn expected_count(&self) -> usize {
        self.expected
    }

    pub(crate) fn found_count(&self) -> usize {
        self.found.len()
    }

    pub(crate) fn verified_count(&self) -> usize {
        self.found_count().saturating_sub(self.wrong_crc.len())
    }

    pub(crate) fn summary_line(&self) -> String {
        format!(
            "ROM set {}: {}/{} expected files present, {}/{} CRCs verified",
            self.directory.display(),
            self.found_count(),
            self.expected_count(),
            self.verified_count(),
            self.found_count()
        )
    }
}

impl From<crate::rom::RomReport> for RomScanReport {
    fn from(report: crate::rom::RomReport) -> Self {
        Self {
            directory: report.directory,
            expected: report.expected,
            found: report.found,
            missing: report.missing,
            unexpected: report.unexpected,
            wrong_size: report.wrong_size,
            wrong_crc: report.wrong_crc,
        }
    }
}

pub(crate) fn scan_dir(path: &Path) -> io::Result<RomScanReport> {
    crate::rom::scan_dir(path).map(RomScanReport::from)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerifiedRomSummary {
    pub(crate) file_count: usize,
    pub(crate) total_bytes: usize,
    pub(crate) region_count: usize,
    pub(crate) load_count: usize,
}

pub(crate) fn verify_dir(path: &Path) -> anyhow::Result<Result<VerifiedRomSummary, RomScanReport>> {
    let verified = match crate::rom::load_verified_dir(path)? {
        Ok(verified) => verified,
        Err(report) => return Ok(Err(RomScanReport::from(report))),
    };
    let images = crate::rom::RedLabelRomImages::from_verified_rom_set(&verified)
        .map_err(|error| anyhow!("verified ROM set could not be mapped: {error}"))?;

    Ok(Ok(VerifiedRomSummary {
        file_count: verified.files().len(),
        total_bytes: verified.total_bytes(),
        region_count: images.regions().len(),
        load_count: images.loads().len(),
    }))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{RomScanReport, VerifiedRomSummary, expected_roms, verify_dir};

    #[test]
    fn expected_roms_adapt_legacy_metadata() {
        let descriptors = expected_roms();

        assert_eq!(descriptors.len(), 14);
        assert_eq!(descriptors[0].name, "defend.1");
        assert_eq!(descriptors[0].size, 2048);
        assert_eq!(descriptors[0].crc32, "c3e52d7e");
        assert_eq!(descriptors[13].name, "decoder.3");
    }

    #[test]
    fn scan_report_summary_matches_current_cli_contract() {
        let report = RomScanReport {
            directory: "roms".into(),
            expected: 2,
            found: vec![String::from("defend.1"), String::from("defend.2")],
            missing: Vec::new(),
            unexpected: Vec::new(),
            wrong_size: Vec::new(),
            wrong_crc: vec![String::from("defend.2 expected ffffffff got 00000000")],
        };

        assert_eq!(
            report.summary_line(),
            "ROM set roms: 2/2 expected files present, 1/2 CRCs verified"
        );
    }

    #[test]
    fn verify_dir_summarizes_complete_rom_fixture() {
        let summary = verify_dir(Path::new("assets/roms/defender"))
            .expect("fixture verification should run")
            .expect("fixture should be complete");

        assert_eq!(
            summary,
            VerifiedRomSummary {
                file_count: 14,
                total_bytes: 29_696,
                region_count: 4,
                load_count: 14,
            }
        );
    }
}
