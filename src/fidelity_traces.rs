//! Runtime-owned fidelity trace command facade.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    thread,
};

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

pub(crate) fn run_check_trace(inputs_path: &Path, expected_path: &Path) -> anyhow::Result<()> {
    print!("{}", check_trace_text(inputs_path, expected_path)?);
    Ok(())
}

pub(crate) fn run_check_trace_dir(path: &Path) -> anyhow::Result<()> {
    print!("{}", check_trace_dir_text(path)?);
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

fn check_trace_text(inputs_path: &Path, expected_path: &Path) -> anyhow::Result<String> {
    let frames = check_trace_frames(inputs_path, expected_path)?;

    Ok(format!(
        "Fidelity trace {} matched {} frame(s)\n",
        expected_path.display(),
        frames
    ))
}

fn check_trace_frames(inputs_path: &Path, expected_path: &Path) -> anyhow::Result<usize> {
    let actual = trace_input_file_text(inputs_path)?;
    let expected = fs::read_to_string(expected_path)
        .with_context(|| format!("failed to read expected trace {}", expected_path.display()))?;
    let comparison = crate::legacy_fidelity::compare_trace_text(&expected, &actual)
        .map_err(|mismatch| anyhow!("{}: {mismatch}", expected_path.display()))?;

    Ok(comparison.frames)
}

fn check_trace_dir_text(path: &Path) -> anyhow::Result<String> {
    if !path.exists() {
        return Ok(format!(
            "Fidelity trace fixture directory {} not found; skipped\n",
            path.display()
        ));
    }
    if !path.is_dir() {
        bail!(
            "fidelity trace fixture path {} is not a directory",
            path.display()
        );
    }

    let fixtures = trace_fixture_pairs(path)?;
    if fixtures.is_empty() {
        return Ok(format!(
            "Fidelity trace fixture directory {} has no *.inputs.txt fixtures; skipped\n",
            path.display()
        ));
    }

    let frames = check_trace_fixtures(&fixtures)?;

    Ok(format!(
        "Fidelity trace fixture directory {} matched {} fixture(s), {} frame(s)\n",
        path.display(),
        fixtures.len(),
        frames
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceFixture {
    inputs_path: PathBuf,
    expected_path: PathBuf,
}

fn check_trace_fixtures(fixtures: &[TraceFixture]) -> anyhow::Result<usize> {
    if fixtures.is_empty() {
        return Ok(0);
    }

    let workers = trace_fixture_worker_count(fixtures.len());
    let chunk_size = fixtures.len().div_ceil(workers);
    let mut checks = Vec::with_capacity(fixtures.len());

    thread::scope(|scope| -> anyhow::Result<()> {
        let handles = fixtures
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(check_trace_fixture)
                        .collect::<Vec<anyhow::Result<usize>>>()
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            checks.extend(
                handle
                    .join()
                    .map_err(|_| anyhow!("fidelity trace fixture worker panicked"))?,
            );
        }

        Ok(())
    })?;

    let mut frames = 0;
    for check in checks {
        frames += check?;
    }

    Ok(frames)
}

fn check_trace_fixture(fixture: &TraceFixture) -> anyhow::Result<usize> {
    check_trace_frames(&fixture.inputs_path, &fixture.expected_path)
}

fn trace_fixture_worker_count(fixture_count: usize) -> usize {
    let available = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    fixture_count.min(available).max(1)
}

fn trace_fixture_pairs(path: &Path) -> anyhow::Result<Vec<TraceFixture>> {
    let mut fixtures = Vec::new();
    let mut expected_traces = Vec::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("failed to read fixture dir {}", path.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read fixture dir {}", path.display()))?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if let Some(stem) = file_name.strip_suffix(".inputs.txt") {
            let expected_path = path.join(format!("{stem}.expected.tsv"));
            if !expected_path.is_file() {
                bail!(
                    "fidelity trace fixture {} is missing expected trace {}",
                    entry_path.display(),
                    expected_path.display()
                );
            }
            fixtures.push(TraceFixture {
                inputs_path: entry_path,
                expected_path,
            });
        } else if let Some(stem) = file_name.strip_suffix(".expected.tsv") {
            expected_traces.push((entry_path, String::from(stem)));
        }
    }

    let scenario_order = crate::legacy_fidelity::trace_scenarios()
        .map(|scenarios| {
            scenarios
                .into_iter()
                .enumerate()
                .map(|(index, scenario)| (scenario.scenario, index))
                .collect::<HashMap<_, _>>()
        })
        .map_err(|error| anyhow!(error))?;
    fixtures.sort_by(|left, right| {
        let left_stem = trace_fixture_stem(&left.inputs_path);
        let right_stem = trace_fixture_stem(&right.inputs_path);
        let left_order = left_stem
            .as_deref()
            .and_then(|stem| scenario_order.get(stem))
            .copied()
            .unwrap_or(usize::MAX);
        let right_order = right_stem
            .as_deref()
            .and_then(|stem| scenario_order.get(stem))
            .copied()
            .unwrap_or(usize::MAX);

        left_order
            .cmp(&right_order)
            .then_with(|| left.inputs_path.cmp(&right.inputs_path))
    });

    expected_traces.sort_by(|left, right| left.0.cmp(&right.0));
    for (expected_path, stem) in expected_traces {
        let inputs_path = path.join(format!("{stem}.inputs.txt"));
        if !inputs_path.is_file() {
            bail!(
                "fidelity trace fixture {} is missing input script {}",
                expected_path.display(),
                inputs_path.display()
            );
        }
    }

    Ok(fixtures)
}

fn trace_fixture_stem(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".inputs.txt"))
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        TraceFixture, check_trace_dir_text, check_trace_fixtures, check_trace_text,
        run_check_trace, run_check_trace_dir, run_trace_inputs, run_trace_inputs_file,
        trace_fixture_pairs, trace_fixture_worker_count, trace_input_file_text, trace_input_text,
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

    #[test]
    fn check_trace_text_compares_expected_trace_file() {
        let path = unique_temp_dir("defender-clean-check-trace");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("expected.tsv");
        fs::write(&inputs_path, "none\n").expect("write input script");
        fs::write(
            &expected_path,
            trace_input_text("none").expect("expected trace text"),
        )
        .expect("write expected trace");

        let text = check_trace_text(&inputs_path, &expected_path).expect("trace should match");
        let _ = fs::remove_dir_all(path);

        assert!(text.ends_with("matched 1 frame(s)\n"));
    }

    #[test]
    fn check_trace_text_reports_mismatch_with_expected_path() {
        let path = unique_temp_dir("defender-clean-check-trace-mismatch");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("expected.tsv");
        fs::write(&inputs_path, "none\n").expect("write input script");
        fs::write(&expected_path, "frame\tinput_bits\n1\t0xFFFF\n").expect("write bad trace");

        let error = check_trace_text(&inputs_path, &expected_path)
            .expect_err("mismatched expected trace should fail");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("expected.tsv: trace mismatch"));
    }

    #[test]
    fn check_trace_text_reports_expected_trace_read_failures() {
        let path = unique_temp_dir("defender-clean-check-trace-missing-expected");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("missing.tsv");
        fs::write(&inputs_path, "none\n").expect("write input script");

        let error = check_trace_text(&inputs_path, &expected_path)
            .expect_err("missing expected trace should fail");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("failed to read expected trace"));
    }

    #[test]
    fn run_check_trace_accepts_supported_inputs() {
        let path = unique_temp_dir("defender-clean-run-check-trace");
        fs::create_dir_all(&path).expect("create temp dir");
        let inputs_path = path.join("inputs.txt");
        let expected_path = path.join("expected.tsv");
        fs::write(&inputs_path, "none\n").expect("write input script");
        fs::write(
            &expected_path,
            trace_input_text("none").expect("expected trace text"),
        )
        .expect("write expected trace");

        run_check_trace(&inputs_path, &expected_path).expect("trace check should run");
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn check_trace_dir_text_skips_missing_directory() {
        let path = unique_temp_dir("defender-clean-trace-dir-missing");
        let _ = fs::remove_dir_all(&path);

        let text = check_trace_dir_text(&path).expect("missing dir skips");

        assert!(text.contains("not found; skipped"));
    }

    #[test]
    fn check_trace_dir_text_skips_empty_directory() {
        let path = unique_temp_dir("defender-clean-trace-dir-empty");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::create_dir(path.join("nested")).expect("create nested non-fixture dir");

        let text = check_trace_dir_text(&path).expect("empty dir skips");
        let _ = fs::remove_dir_all(path);

        assert!(text.contains("has no *.inputs.txt fixtures; skipped"));
    }

    #[test]
    fn check_trace_dir_text_rejects_non_directory_path() {
        let path = unique_temp_dir("defender-clean-trace-dir-file");
        fs::write(&path, "not a directory").expect("write file fixture path");

        let error = check_trace_dir_text(&path).expect_err("file path should fail");
        let _ = fs::remove_file(path);

        assert!(error.to_string().contains("is not a directory"));
    }

    #[test]
    fn check_trace_dir_text_compares_fixture_pairs() {
        let path = unique_temp_dir("defender-clean-trace-dir-match");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("attract.inputs.txt"), "coin,start_one;fire")
            .expect("write fixture input");
        fs::write(
            path.join("attract.expected.tsv"),
            trace_input_text("coin,start_one;fire").expect("trace text"),
        )
        .expect("write expected trace");

        let text = check_trace_dir_text(&path).expect("fixture dir should match");
        let _ = fs::remove_dir_all(path);

        assert!(text.contains("matched 1 fixture(s), 2 frame(s)"));
    }

    #[test]
    fn check_trace_dir_text_names_mismatched_fixture() {
        let path = unique_temp_dir("defender-clean-trace-dir-mismatch");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("start.inputs.txt"), "coin,start_one").expect("write fixture input");
        fs::write(path.join("start.expected.tsv"), "not\ta\ttrace\n")
            .expect("write expected trace");

        let error = check_trace_dir_text(&path).expect_err("trace mismatch");
        let _ = fs::remove_dir_all(path);
        let message = error.to_string();

        assert!(message.contains("start.expected.tsv"));
        assert!(message.contains("trace mismatch at line 1"));
    }

    #[test]
    fn check_trace_dir_text_checks_manifest_fixtures_in_order() {
        let path = unique_temp_dir("defender-clean-trace-dir-order");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("abduction.inputs.txt"), "coin,start_one")
            .expect("write abduction input");
        fs::write(path.join("abduction.expected.tsv"), "bad\tabduction\n")
            .expect("write abduction expected trace");
        fs::write(path.join("attract_boot.inputs.txt"), "none").expect("write attract input");
        fs::write(path.join("attract_boot.expected.tsv"), "bad\tattract\n")
            .expect("write attract expected trace");

        let error = check_trace_dir_text(&path).expect_err("trace mismatch");
        let _ = fs::remove_dir_all(path);
        let message = error.to_string();

        assert!(message.contains("attract_boot.expected.tsv"));
        assert!(!message.contains("abduction.expected.tsv"));
    }

    #[test]
    fn trace_fixture_pairs_reject_missing_expected_trace() {
        let path = unique_temp_dir("defender-clean-trace-dir-missing-expected");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.inputs.txt"), "none").expect("write fixture input");

        let error = trace_fixture_pairs(&path).expect_err("missing expected trace should fail");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("missing expected trace"));
    }

    #[test]
    fn trace_fixture_pairs_reject_expected_without_input_script() {
        let path = unique_temp_dir("defender-clean-trace-dir-missing-input");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.expected.tsv"), "frame\tinputs_bits\n")
            .expect("write expected trace");

        let error = trace_fixture_pairs(&path).expect_err("missing input script should fail");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("missing input script"));
    }

    #[test]
    fn check_trace_fixtures_sums_frames_in_fixture_order() {
        let path = unique_temp_dir("defender-clean-check-fixtures");
        fs::create_dir_all(&path).expect("create fixture dir");
        let first_inputs = path.join("first.inputs.txt");
        let first_expected = path.join("first.expected.tsv");
        let second_inputs = path.join("second.inputs.txt");
        let second_expected = path.join("second.expected.tsv");
        fs::write(&first_inputs, "none\n").expect("write first input");
        fs::write(
            &first_expected,
            trace_input_text("none").expect("first expected trace"),
        )
        .expect("write first expected trace");
        fs::write(&second_inputs, "none;none\n").expect("write second input");
        fs::write(
            &second_expected,
            trace_input_text("none;none").expect("second expected trace"),
        )
        .expect("write second expected trace");

        let frames = check_trace_fixtures(&[
            TraceFixture {
                inputs_path: first_inputs,
                expected_path: first_expected,
            },
            TraceFixture {
                inputs_path: second_inputs,
                expected_path: second_expected,
            },
        ])
        .expect("fixture checks should pass");
        let _ = fs::remove_dir_all(path);

        assert_eq!(frames, 3);
        assert_eq!(check_trace_fixtures(&[]).expect("empty fixture list"), 0);
        assert_eq!(trace_fixture_worker_count(0), 1);
    }

    #[test]
    fn run_check_trace_dir_accepts_supported_inputs() {
        let path = unique_temp_dir("defender-clean-run-check-trace-dir");
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.inputs.txt"), "none\n").expect("write fixture input");
        fs::write(
            path.join("boot.expected.tsv"),
            trace_input_text("none").expect("expected trace text"),
        )
        .expect("write expected trace");

        run_check_trace_dir(&path).expect("trace fixture directory should run");
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
