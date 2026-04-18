use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CANONICAL_ROM_SET: [&str; 14] = [
    "decoder.2",
    "decoder.3",
    "defend.1",
    "defend.2",
    "defend.3",
    "defend.4",
    "defend.6",
    "defend.7",
    "defend.8",
    "defend.9",
    "defend.10",
    "defend.11",
    "defend.12",
    "video_sound_rom_1.ic12",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomReport {
    pub directory: PathBuf,
    pub found: Vec<String>,
    pub missing: Vec<String>,
    pub unexpected: Vec<String>,
}

impl RomReport {
    pub fn expected_count(&self) -> usize {
        CANONICAL_ROM_SET.len()
    }

    pub fn found_count(&self) -> usize {
        self.found.len()
    }

    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "ROM set {}: {}/{} expected files present",
            self.directory.display(),
            self.found_count(),
            self.expected_count()
        )
    }
}

pub fn scan_dir(path: &Path) -> io::Result<RomReport> {
    let mut found = BTreeSet::new();
    let mut unexpected = BTreeSet::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if CANONICAL_ROM_SET.contains(&name.as_str()) {
            found.insert(name);
        } else {
            unexpected.insert(name);
        }
    }

    let found: Vec<String> = CANONICAL_ROM_SET
        .iter()
        .filter(|name| found.contains(**name))
        .map(|name| (*name).to_string())
        .collect();

    let missing: Vec<String> = CANONICAL_ROM_SET
        .iter()
        .filter(|name| !found.iter().any(|entry| entry == **name))
        .map(|name| (*name).to_string())
        .collect();

    Ok(RomReport {
        directory: path.to_path_buf(),
        found,
        missing,
        unexpected: unexpected.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{CANONICAL_ROM_SET, scan_dir};

    static NEXT_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "defender-rom-test-{}-{}",
                std::process::id(),
                NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn canonical_rom_set_matches_expected_size() {
        assert_eq!(CANONICAL_ROM_SET.len(), 14);
    }

    #[test]
    fn scan_dir_reports_missing_and_unexpected_files() {
        let temp_dir = TempDir::new();
        fs::write(temp_dir.path().join("defend.1"), []).expect("write file");
        fs::write(temp_dir.path().join("decoder.2"), []).expect("write file");
        fs::write(temp_dir.path().join("notes.txt"), []).expect("write file");

        let report = scan_dir(temp_dir.path()).expect("scan rom dir");

        assert_eq!(report.found_count(), 2);
        assert!(report.missing.contains(&String::from("defend.12")));
        assert_eq!(report.unexpected, vec![String::from("notes.txt")]);
    }

    #[test]
    fn summary_line_uses_found_and_expected_counts() {
        let temp_dir = TempDir::new();
        for file_name in CANONICAL_ROM_SET {
            fs::write(temp_dir.path().join(file_name), []).expect("write rom");
        }

        let report = scan_dir(temp_dir.path()).expect("scan rom dir");

        assert!(report.is_complete());
        assert!(report.summary_line().contains("14/14"));
    }
}
