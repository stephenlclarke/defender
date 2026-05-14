//! Runtime-owned fidelity trace command facade.

use std::{fs, path::Path};

use anyhow::{Context, anyhow, bail};

pub(crate) fn run_trace(frame_count: usize) -> anyhow::Result<()> {
    print!("{}", trace_text(frame_count)?);
    Ok(())
}

pub(crate) fn run_trace_inputs(script: &str) -> anyhow::Result<()> {
    print!("{}", trace_input_text(script)?);
    Ok(())
}

pub(crate) fn run_trace_inputs_file(path: &Path) -> anyhow::Result<()> {
    print!("{}", trace_input_file_text(path)?);
    Ok(())
}

fn trace_text(frame_count: usize) -> anyhow::Result<String> {
    if frame_count == 0 {
        bail!("--fidelity-trace frame count must be greater than zero");
    }

    let input_program = format!("none*{frame_count}");
    let input_script = crate::legacy_fidelity::expanded_trace_input_text(&input_program)
        .map_err(|error| anyhow!(error))?;
    trace_input_text(&input_script)
}

fn trace_input_text(script: &str) -> anyhow::Result<String> {
    let inputs =
        crate::legacy_fidelity::parse_trace_input_script(script).map_err(|error| anyhow!(error))?;
    crate::legacy_fidelity::trace_text_for_inputs(&inputs).map_err(|error| anyhow!(error))
}

fn trace_input_file_text(path: &Path) -> anyhow::Result<String> {
    let script = fs::read_to_string(path)
        .with_context(|| format!("failed to read trace input script {}", path.display()))?;
    trace_input_text(&script)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        run_trace_inputs, run_trace_inputs_file, trace_input_file_text, trace_input_text,
        trace_text,
    };

    #[test]
    fn trace_text_preserves_default_idle_trace_contract() {
        let text = trace_text(2).expect("trace text");

        assert!(text.starts_with("frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2\tphase\t"));
        assert!(text.contains("\n1\t0x0000\t0x00\t0x00\t0x00\t"));
        assert!(text.contains("\n2\t0x0000\t0x00\t0x00\t0x00\t"));
        assert_eq!(text.lines().count(), 3);
    }

    #[test]
    fn trace_text_rejects_zero_frames() {
        let error = trace_text(0).expect_err("zero frames should fail");

        assert_eq!(
            error.to_string(),
            "--fidelity-trace frame count must be greater than zero"
        );
    }

    #[test]
    fn trace_input_text_applies_scripted_inputs() {
        let text = trace_input_text("coin,start_one;fire,thrust;none").expect("trace text");

        assert!(text.starts_with("frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2\tphase\t"));
        assert!(text.contains("\n1\t"));
        assert!(text.contains("\n2\t"));
        assert!(text.contains("\n3\t"));
        assert_eq!(text.lines().count(), 4);
    }

    #[test]
    fn trace_input_text_reports_invalid_script() {
        let error = trace_input_text("warp").expect_err("invalid script should fail");

        assert_eq!(
            error.to_string(),
            "unknown trace input action 'warp' in frame 1"
        );
    }

    #[test]
    fn trace_input_file_text_reads_scripted_inputs() {
        let path = unique_temp_dir("defender-clean-trace-input-file");
        fs::create_dir_all(&path).expect("create temp dir");
        let script_path = path.join("inputs.txt");
        fs::write(&script_path, "none;none\n").expect("write input script");

        let text = trace_input_file_text(&script_path).expect("trace text");
        let _ = fs::remove_dir_all(path);

        assert!(text.contains("\n2\t0x0000\t"));
        assert_eq!(text.lines().count(), 3);
    }

    #[test]
    fn trace_input_file_text_reports_read_failures() {
        let error = trace_input_file_text(Path::new("missing-trace-inputs.txt"))
            .expect_err("missing input script should fail");

        assert!(
            error
                .to_string()
                .contains("failed to read trace input script missing-trace-inputs.txt")
        );
    }

    #[test]
    fn run_trace_input_commands_accept_supported_inputs() {
        let path = unique_temp_dir("defender-clean-run-trace-input-file");
        fs::create_dir_all(&path).expect("create temp dir");
        let script_path = path.join("inputs.txt");
        fs::write(&script_path, "none\n").expect("write input script");

        run_trace_inputs("none").expect("inline trace inputs should run");
        run_trace_inputs_file(&script_path).expect("file trace inputs should run");
        let _ = fs::remove_dir_all(path);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
