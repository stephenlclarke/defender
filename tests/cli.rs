use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR_ID: AtomicUsize = AtomicUsize::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "defender-cli-test-{}-{}",
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
fn default_run_renders_the_logo_scene() {
    let output = Command::new(env!("CARGO_BIN_EXE_defender"))
        .output()
        .expect("run defender");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("NATIVE RUST PROTOTYPE"));
}

#[test]
fn gameplay_mode_renders_a_defender_frame() {
    let output = Command::new(env!("CARGO_BIN_EXE_defender"))
        .args(["--frames", "2"])
        .output()
        .expect("run defender");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("DEFENDER"));
    assert!(stdout.contains("THREAT"));
}

#[test]
fn scene_mode_renders_high_scores() {
    let output = Command::new(env!("CARGO_BIN_EXE_defender"))
        .args(["--scene", "high-score"])
        .output()
        .expect("run defender");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("HIGH SCORES"));
    assert!(stdout.contains("250000"));
}

#[test]
fn rom_report_summarises_canonical_files() {
    let temp_dir = TempDir::new();
    fs::write(temp_dir.path().join("defend.1"), []).expect("write rom");
    fs::write(temp_dir.path().join("decoder.2"), []).expect("write rom");

    let output = Command::new(env!("CARGO_BIN_EXE_defender"))
        .args(["--rom-report"])
        .arg(temp_dir.path())
        .output()
        .expect("run defender");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("2/14"));
    assert!(stdout.contains("Missing:"));
}
