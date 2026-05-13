//! CMOS persistence for local live runs.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::board::{CMOS_RAM_SIZE, CmosRam};

pub trait CmosStorage {
    fn load_cmos(&self) -> io::Result<Option<CmosRam>>;
    fn save_cmos(&self, cmos: &CmosRam) -> io::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCmosStorage {
    path: PathBuf,
}

impl FileCmosStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn temp_path(&self) -> PathBuf {
        let mut temp_path = self.path.clone();
        let mut file_name = self
            .path
            .file_name()
            .map(OsString::from)
            .unwrap_or_else(|| OsString::from("red-label-cmos.bin"));
        file_name.push(format!(".{}.tmp", std::process::id()));
        temp_path.set_file_name(file_name);
        temp_path
    }
}

impl CmosStorage for FileCmosStorage {
    fn load_cmos(&self) -> io::Result<Option<CmosRam>> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let len = bytes.len();
        let cmos = CmosRam::try_from(bytes).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "CMOS image {} has {len} bytes, expected {CMOS_RAM_SIZE}",
                    self.path.display()
                ),
            )
        })?;
        validate_cmos_image(&cmos)?;
        Ok(Some(cmos))
    }

    fn save_cmos(&self, cmos: &CmosRam) -> io::Result<()> {
        validate_cmos_image(cmos)?;
        if let Some(parent) = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.temp_path();
        fs::write(&temp_path, cmos)?;
        if let Err(error) = fs::rename(&temp_path, &self.path) {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }
        Ok(())
    }
}

pub fn validate_cmos_image(cmos: &CmosRam) -> io::Result<()> {
    if let Some(index) = cmos.iter().position(|cell| cell & 0xF0 != 0xF0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("CMOS cell 0x{index:02X} is not a red-label 4-bit cell"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::{env, process};

    use crate::board::cmos_sram_write_byte;

    use super::{CMOS_RAM_SIZE, CmosStorage, FileCmosStorage};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_cmos_path(name: &str) -> std::path::PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir()
            .join(format!("defender-cmos-{name}-{}-{id}", process::id()))
            .join("red-label-cmos.bin")
    }

    #[test]
    fn file_cmos_storage_treats_missing_file_as_no_cmos() {
        let path = unique_cmos_path("missing");
        let storage = FileCmosStorage::new(&path);

        assert_eq!(storage.load_cmos().expect("load missing CMOS"), None);
    }

    #[test]
    fn file_cmos_storage_round_trips_cmos_cells() {
        let path = unique_cmos_path("round-trip");
        let storage = FileCmosStorage::new(&path);
        let mut cmos = [0xF0; CMOS_RAM_SIZE];
        cmos_sram_write_byte(&mut cmos, 0x7D, 0x09).expect("write credit backup");

        storage.save_cmos(&cmos).expect("save CMOS");
        assert_eq!(storage.load_cmos().expect("load CMOS"), Some(cmos));

        fs::remove_dir_all(path.parent().expect("test dir")).expect("remove test dir");
    }

    #[test]
    fn file_cmos_storage_rejects_wrong_size_and_bad_cells() {
        let wrong_size_path = unique_cmos_path("wrong-size");
        fs::create_dir_all(wrong_size_path.parent().expect("wrong-size dir"))
            .expect("create wrong-size dir");
        fs::write(&wrong_size_path, [0xF0; CMOS_RAM_SIZE - 1]).expect("write wrong-size CMOS");
        let wrong_size_error = FileCmosStorage::new(&wrong_size_path)
            .load_cmos()
            .expect_err("wrong-size CMOS should fail");
        assert_eq!(wrong_size_error.kind(), std::io::ErrorKind::InvalidData);

        let bad_cell_path = unique_cmos_path("bad-cell");
        fs::create_dir_all(bad_cell_path.parent().expect("bad-cell dir"))
            .expect("create bad-cell dir");
        let mut bad = [0xF0; CMOS_RAM_SIZE];
        bad[0x7F] = 0x5A;
        fs::write(&bad_cell_path, bad).expect("write bad CMOS");
        let bad_cell_error = FileCmosStorage::new(&bad_cell_path)
            .load_cmos()
            .expect_err("bad CMOS cell should fail");
        assert_eq!(bad_cell_error.kind(), std::io::ErrorKind::InvalidData);

        fs::remove_dir_all(wrong_size_path.parent().expect("wrong-size dir"))
            .expect("remove wrong-size dir");
        fs::remove_dir_all(bad_cell_path.parent().expect("bad-cell dir"))
            .expect("remove bad-cell dir");
    }
}
