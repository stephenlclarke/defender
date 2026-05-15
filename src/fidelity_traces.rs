//! Runtime-owned fidelity trace command facade.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    thread,
};

use anyhow::{Context, anyhow, bail};

const TRACE_REQUIREMENTS_TSV: &str = include_str!("../assets/red-label/trace-requirements.tsv");

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

pub(crate) fn run_check_reference_trace_dir(path: &Path) -> anyhow::Result<()> {
    print!("{}", check_reference_trace_dir_text(path)?);
    Ok(())
}

fn trace_text(frame_count: usize) -> anyhow::Result<String> {
    if frame_count == 0 {
        bail!("--fidelity-trace frame count must be greater than zero");
    }

    let input_program = format!("none*{frame_count}");
    let input_script = crate::fidelity_manifest::expanded_input_text(&input_program)?;
    trace_input_text(&input_script)
}

fn trace_input_text(script: &str) -> anyhow::Result<String> {
    crate::fidelity_trace_engine::trace_text_for_script(script)
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
    let frames = crate::fidelity_trace_engine::compare_trace_text(&expected, &actual)
        .map_err(|error| anyhow!("{}: {error}", expected_path.display()))?;

    Ok(frames)
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

    let scenario_order = crate::fidelity_manifest::scenarios()?
        .into_iter()
        .enumerate()
        .map(|(index, scenario)| (scenario.name, index))
        .collect::<HashMap<_, _>>();
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

fn check_reference_trace_dir_text(path: &Path) -> anyhow::Result<String> {
    if !path.is_dir() {
        bail!(
            "reference trace fixture path {} is not a directory",
            path.display()
        );
    }

    let scenarios = crate::fidelity_manifest::scenarios()?;
    let scenario_names = scenarios
        .iter()
        .map(|scenario| scenario.name.as_str())
        .collect::<HashSet<_>>();
    let requirements = trace_requirements()?;
    validate_trace_requirements_reference_known_scenarios(&scenario_names, &requirements)?;
    let expected_header = crate::fidelity_trace_engine::trace_header();
    let mut frames = 0;
    for scenario in &scenarios {
        let inputs_path = path.join(format!("{}.inputs.txt", scenario.name));
        let expected_path = path.join(format!("{}.expected.tsv", scenario.name));
        let actual_inputs = fs::read_to_string(&inputs_path)
            .with_context(|| format!("failed to read {}", inputs_path.display()))?;
        let expected_inputs =
            crate::fidelity_manifest::expanded_input_text(&scenario.input_program)?;
        if actual_inputs.trim_end() != expected_inputs.trim_end() {
            bail!(
                "reference trace fixture {} does not match embedded scenario input program",
                inputs_path.display()
            );
        }

        let expected_trace = fs::read_to_string(&expected_path)
            .with_context(|| format!("failed to read {}", expected_path.display()))?;
        let lines = expected_trace.lines().collect::<Vec<_>>();
        if lines.first().copied() != Some(expected_header) {
            bail!(
                "reference trace fixture {} header does not match assets/red-label/trace-schema.tsv",
                expected_path.display()
            );
        }
        let trace_frames = lines.len().saturating_sub(1);
        if trace_frames != scenario.frame_count {
            bail!(
                "reference trace fixture {} has {} frame(s), expected {}",
                expected_path.display(),
                trace_frames,
                scenario.frame_count
            );
        }
        check_reference_trace_required_cells(&expected_path, &lines)?;
        if let Some(requirement) = requirements.get(&scenario.name) {
            let input_frames = actual_inputs.trim_end().split(';').collect::<Vec<_>>();
            check_reference_trace_evidence(&expected_path, &lines, requirement, &input_frames)?;
        }
        frames += trace_frames;
    }

    Ok(format!(
        "Reference trace fixture directory {} has {} complete Phase 1 fixture(s), {} frame(s)\n",
        path.display(),
        scenarios.len(),
        frames
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceRequirement {
    required_sound_commands: Vec<String>,
    required_events: Vec<String>,
}

fn trace_requirements() -> anyhow::Result<HashMap<String, TraceRequirement>> {
    parse_trace_requirements(TRACE_REQUIREMENTS_TSV)
}

fn parse_trace_requirements(tsv: &str) -> anyhow::Result<HashMap<String, TraceRequirement>> {
    let mut lines = tsv.lines();
    let Some(header) = lines.next() else {
        bail!("trace requirement TSV is empty");
    };
    if header != "scenario\trequired_sound_commands\trequired_events\tdescription\tsource" {
        bail!("unexpected trace requirement header: {header}");
    }

    let mut requirements = HashMap::new();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            bail!(
                "trace requirement line {} has {} fields, expected 5",
                index + 2,
                fields.len()
            );
        }
        let scenario = fields[0].trim();
        if scenario.is_empty() {
            bail!("trace requirement line {} has empty scenario", index + 2);
        }
        if requirements
            .insert(
                String::from(scenario),
                TraceRequirement {
                    required_sound_commands: parse_requirement_values(fields[1]),
                    required_events: parse_requirement_values(fields[2]),
                },
            )
            .is_some()
        {
            bail!("trace requirement line {} duplicates {scenario}", index + 2);
        }
    }
    Ok(requirements)
}

fn parse_requirement_values(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "-")
        .map(String::from)
        .collect()
}

fn validate_trace_requirements_reference_known_scenarios(
    scenario_names: &HashSet<&str>,
    requirements: &HashMap<String, TraceRequirement>,
) -> anyhow::Result<()> {
    for scenario in requirements.keys() {
        if !scenario_names.contains(scenario.as_str()) {
            bail!("trace requirement references unknown scenario {scenario}");
        }
    }
    Ok(())
}

fn check_reference_trace_required_cells(
    expected_path: &Path,
    lines: &[&str],
) -> anyhow::Result<()> {
    let header = lines.first().copied().unwrap_or_default();
    let columns = header.split('\t').collect::<Vec<_>>();
    let required_columns = [
        "frame",
        "input_bits",
        "input_in0",
        "input_in1",
        "input_in2",
        "phase",
        "p1_score",
        "p2_score",
        "wave",
        "lives",
        "smart_bombs",
        "seed",
        "hseed",
        "lseed",
        "object_table_crc32",
        "process_table_crc32",
        "super_process_table_crc32",
        "shell_table_crc32",
        "video_crc32",
    ];
    let required_indexes = required_columns
        .iter()
        .map(|column| trace_column_index(&columns, column).map(|index| (column, index)))
        .collect::<anyhow::Result<Vec<_>>>()?;

    for (index, line) in lines.iter().copied().enumerate().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != columns.len() {
            bail!(
                "reference trace fixture {} line {} has {} columns, expected {}",
                expected_path.display(),
                index + 1,
                fields.len(),
                columns.len()
            );
        }
        for (column, field_index) in &required_indexes {
            let value = fields[*field_index].trim();
            if value.is_empty() || value == "-" {
                bail!(
                    "reference trace fixture {} line {} column {} is missing required value",
                    expected_path.display(),
                    index + 1,
                    column
                );
            }
        }
    }

    Ok(())
}

fn check_reference_trace_evidence(
    expected_path: &Path,
    lines: &[&str],
    requirement: &TraceRequirement,
    input_frames: &[&str],
) -> anyhow::Result<()> {
    let header = lines.first().copied().unwrap_or_default();
    let columns = header.split('\t').collect::<Vec<_>>();
    let sound_index = trace_column_index(&columns, "sound_commands")?;
    let events_index = trace_column_index(&columns, "events")?;
    let mut sound_commands = HashSet::new();
    let mut events = HashSet::new();
    let evidence_deadline = reference_evidence_deadline_frame(input_frames, requirement);

    for (index, line) in lines.iter().copied().enumerate().skip(1) {
        let frame_number = index;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != columns.len() {
            bail!(
                "reference trace fixture {} line {} has {} columns, expected {}",
                expected_path.display(),
                index + 1,
                fields.len(),
                columns.len()
            );
        }
        if evidence_deadline.is_none_or(|deadline| frame_number <= deadline) {
            sound_commands.extend(trace_cell_values(fields[sound_index]));
            events.extend(trace_cell_values(fields[events_index]));
        }
    }

    for command in &requirement.required_sound_commands {
        if !sound_commands.contains(command) {
            let timing = evidence_timing_message(evidence_deadline);
            bail!(
                "reference trace fixture {} is missing required sound command {}{}",
                expected_path.display(),
                command,
                timing
            );
        }
    }
    for event in &requirement.required_events {
        if !events.contains(event) {
            let timing = evidence_timing_message(evidence_deadline);
            bail!(
                "reference trace fixture {} is missing required event {}{}",
                expected_path.display(),
                event,
                timing
            );
        }
    }

    Ok(())
}

fn reference_evidence_deadline_frame(
    input_frames: &[&str],
    requirement: &TraceRequirement,
) -> Option<usize> {
    if requirement.required_sound_commands.is_empty() && requirement.required_events.is_empty() {
        return None;
    }
    input_frames
        .iter()
        .rposition(|frame| trace_input_frame_contains_start(frame))
        .map(|index| index + 1)
}

fn trace_input_frame_contains_start(frame: &str) -> bool {
    frame
        .split(',')
        .map(str::trim)
        .any(|action| matches!(action, "start_one" | "start1" | "start_two" | "start2"))
}

fn evidence_timing_message(deadline: Option<usize>) -> String {
    deadline
        .map(|frame| format!(" by frame {frame}"))
        .unwrap_or_default()
}

fn trace_column_index(columns: &[&str], column: &str) -> anyhow::Result<usize> {
    columns
        .iter()
        .position(|candidate| *candidate == column)
        .ok_or_else(|| anyhow!("trace schema is missing {column} column"))
}

fn trace_cell_values(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "-")
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        TraceFixture, TraceRequirement, check_reference_trace_dir_text,
        check_reference_trace_evidence, check_reference_trace_required_cells, check_trace_dir_text,
        check_trace_fixtures, check_trace_text, parse_requirement_values, parse_trace_requirements,
        run_check_reference_trace_dir, run_check_trace, run_check_trace_dir, run_trace_inputs,
        run_trace_inputs_file, trace_cell_values, trace_fixture_pairs, trace_fixture_worker_count,
        trace_input_file_text, trace_input_text, trace_text,
        validate_trace_requirements_reference_known_scenarios,
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

    #[test]
    fn check_reference_trace_dir_text_validates_required_phase_one_fixtures() {
        let path = unique_temp_dir("defender-clean-reference-fixtures");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_reference_fixtures(&path, true);

        let text =
            check_reference_trace_dir_text(&path).expect("reference fixtures should validate");
        let _ = fs::remove_dir_all(path);

        assert!(text.contains("12 complete Phase 1 fixture(s), 22308 frame(s)"));
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_non_directory_path() {
        let path = unique_temp_dir("defender-clean-reference-dir-file");
        fs::write(&path, "not a directory").expect("write file fixture path");

        let error =
            check_reference_trace_dir_text(&path).expect_err("file path should fail validation");
        let _ = fs::remove_file(path);

        assert!(error.to_string().contains("is not a directory"));
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_input_program_drift() {
        let path = unique_temp_dir("defender-clean-reference-fixtures-input-drift");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_reference_fixtures(&path, true);
        fs::write(path.join("attract_boot.inputs.txt"), "none\n").expect("drift inputs");

        let error =
            check_reference_trace_dir_text(&path).expect_err("input drift should fail validation");
        let _ = fs::remove_dir_all(path);

        assert!(
            error
                .to_string()
                .contains("does not match embedded scenario input program")
        );
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_header_drift() {
        let path = unique_temp_dir("defender-clean-reference-fixtures-header-drift");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        fs::write(path.join("attract_boot.expected.tsv"), "bad\theader\n")
            .expect("write bad expected fixture");

        let error =
            check_reference_trace_dir_text(&path).expect_err("header drift should fail validation");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("header does not match"));
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_frame_count_drift() {
        let path = unique_temp_dir("defender-clean-reference-fixtures-frame-drift");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_expected_fixture(&path, "attract_boot", 1, None);

        let error = check_reference_trace_dir_text(&path)
            .expect_err("frame count drift should fail validation");
        let _ = fs::remove_dir_all(path);

        assert!(error.to_string().contains("has 1 frame(s), expected 900"));
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_missing_required_start_evidence() {
        let path = unique_temp_dir("defender-clean-reference-fixtures-missing-evidence");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_reference_fixtures(&path, false);

        let error = check_reference_trace_dir_text(&path)
            .expect_err("missing start evidence should fail validation");
        let _ = fs::remove_dir_all(path);

        assert!(
            error
                .to_string()
                .contains("missing required sound command 0xE6")
        );
    }

    #[test]
    fn check_reference_trace_dir_text_rejects_late_start_evidence() {
        let path = unique_temp_dir("defender-clean-reference-fixtures-late-evidence");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_reference_fixtures(&path, false);
        let start_frame_count = fs::read_to_string(path.join("start_game.inputs.txt"))
            .expect("read start_game inputs")
            .trim_end()
            .split(';')
            .count();
        write_minimal_expected_fixture(
            &path,
            "start_game",
            start_frame_count,
            Some(start_frame_count),
        );

        let error = check_reference_trace_dir_text(&path)
            .expect_err("late start evidence should fail validation");
        let _ = fs::remove_dir_all(path);

        assert!(
            error
                .to_string()
                .contains("missing required sound command 0xE6 by frame")
        );
    }

    #[test]
    fn trace_requirement_parser_handles_lists_and_rejects_duplicate_scenarios() {
        let requirements = parse_trace_requirements(
            "scenario\trequired_sound_commands\trequired_events\tdescription\tsource\n\
             start_game\t0xE6,0xF5\tcredit_added,game_started\tdescription\tsource\n\
             \n",
        )
        .expect("requirements");

        assert_eq!(
            requirements["start_game"],
            TraceRequirement {
                required_sound_commands: vec![String::from("0xE6"), String::from("0xF5")],
                required_events: vec![String::from("credit_added"), String::from("game_started")],
            }
        );
        assert_eq!(parse_requirement_values("-"), Vec::<String>::new());
        assert_eq!(
            trace_cell_values("credit_added, game_started"),
            vec![String::from("credit_added"), String::from("game_started")]
        );

        let error = parse_trace_requirements(
            "scenario\trequired_sound_commands\trequired_events\tdescription\tsource\n\
             start_game\t-\t-\tdescription\tsource\n\
             start_game\t-\t-\tdescription\tsource\n",
        )
        .expect_err("duplicate scenario");
        assert!(error.to_string().contains("duplicates start_game"));
    }

    #[test]
    fn trace_requirement_parser_rejects_malformed_rows() {
        let error = parse_trace_requirements("").expect_err("empty requirements");
        assert!(error.to_string().contains("trace requirement TSV is empty"));

        let error = parse_trace_requirements("bad\theader\n").expect_err("bad header");
        assert!(
            error
                .to_string()
                .contains("unexpected trace requirement header")
        );

        let error = parse_trace_requirements(
            "scenario\trequired_sound_commands\trequired_events\tdescription\tsource\n\
             start_game\t-\t-\n",
        )
        .expect_err("short row");
        assert!(error.to_string().contains("has 3 fields, expected 5"));

        let error = parse_trace_requirements(
            "scenario\trequired_sound_commands\trequired_events\tdescription\tsource\n\
             \t-\t-\tdescription\tsource\n",
        )
        .expect_err("empty scenario");
        assert!(error.to_string().contains("empty scenario"));
    }

    #[test]
    fn trace_requirements_must_reference_known_scenarios() {
        let scenario_names = HashSet::from(["attract_boot"]);
        let requirements = HashMap::from([(
            String::from("missing"),
            TraceRequirement {
                required_sound_commands: Vec::new(),
                required_events: Vec::new(),
            },
        )]);

        let error =
            validate_trace_requirements_reference_known_scenarios(&scenario_names, &requirements)
                .expect_err("unknown scenario");

        assert!(
            error
                .to_string()
                .contains("trace requirement references unknown scenario missing")
        );
    }

    #[test]
    fn check_reference_trace_required_cells_rejects_stale_video_crc() {
        let lines = [
            crate::fidelity_trace_engine::trace_header(),
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t0xE15D8394\t0xC4C53DA1\t0x05B7E865\t0x41D912FF\t-\t-\t-",
        ];

        let error = check_reference_trace_required_cells(
            &PathBuf::from("/tmp/attract_boot.expected.tsv"),
            &lines,
        )
        .expect_err("stale video CRC");

        assert!(
            error
                .to_string()
                .contains("column video_crc32 is missing required value")
        );
    }

    #[test]
    fn check_reference_trace_required_cells_rejects_bad_trace_columns() {
        let lines = [
            crate::fidelity_trace_engine::trace_header(),
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t0xE15D8394\t0xC4C53DA1\t0x05B7E865\t0x41D912FF",
        ];

        let error = check_reference_trace_required_cells(
            &PathBuf::from("/tmp/attract_boot.expected.tsv"),
            &lines,
        )
        .expect_err("bad trace columns");

        assert!(error.to_string().contains("has 18 columns, expected 21"));
    }

    #[test]
    fn check_reference_trace_evidence_rejects_missing_required_event() {
        let lines = [
            crate::fidelity_trace_engine::trace_header(),
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t-\t-\t-\t-\t-\t0xE6\t-",
        ];
        let requirement = TraceRequirement {
            required_sound_commands: vec![String::from("0xE6")],
            required_events: vec![String::from("credit_added")],
        };

        let error = check_reference_trace_evidence(
            &PathBuf::from("/tmp/start_game.expected.tsv"),
            &lines,
            &requirement,
            &["coin,start_one"],
        )
        .expect_err("missing event");

        assert!(
            error
                .to_string()
                .contains("missing required event credit_added")
        );
    }

    #[test]
    fn check_reference_trace_evidence_rejects_bad_trace_columns() {
        let lines = [
            crate::fidelity_trace_engine::trace_header(),
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t-\t-\t-\t-\t-\t0xE6",
        ];
        let requirement = TraceRequirement {
            required_sound_commands: Vec::new(),
            required_events: vec![String::from("credit_added")],
        };

        let error = check_reference_trace_evidence(
            &PathBuf::from("/tmp/start_game.expected.tsv"),
            &lines,
            &requirement,
            &["coin,start_one"],
        )
        .expect_err("bad trace columns");

        assert!(error.to_string().contains("has 20 columns, expected 21"));
    }

    #[test]
    fn check_reference_trace_evidence_rejects_late_bad_trace_columns() {
        let lines = [
            crate::fidelity_trace_engine::trace_header(),
            "1\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t-\t-\t-\t-\t-\t0xE6\tcredit_added",
            "2\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t-\t-\t-\t-\t-\tlate",
        ];
        let requirement = TraceRequirement {
            required_sound_commands: vec![String::from("0xE6")],
            required_events: vec![String::from("credit_added")],
        };

        let error = check_reference_trace_evidence(
            &PathBuf::from("/tmp/start_game.expected.tsv"),
            &lines,
            &requirement,
            &["start_one", "none"],
        )
        .expect_err("late bad trace columns");

        assert!(
            error
                .to_string()
                .contains("line 3 has 20 columns, expected 21")
        );
    }

    #[test]
    fn run_check_reference_trace_dir_accepts_supported_inputs() {
        let path = unique_temp_dir("defender-clean-run-reference-trace-dir");
        let _ = fs::remove_dir_all(&path);
        write_reference_scenario_inputs(&path);
        write_minimal_reference_fixtures(&path, true);

        run_check_reference_trace_dir(&path).expect("reference trace directory should run");
        let _ = fs::remove_dir_all(path);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }

    fn write_reference_scenario_inputs(path: &Path) {
        fs::create_dir_all(path).expect("create scenario input dir");
        for scenario in crate::fidelity_manifest::scenarios().expect("trace scenarios") {
            let input_text = crate::fidelity_manifest::expanded_input_text(&scenario.input_program)
                .expect("expanded inputs");
            fs::write(
                path.join(format!("{}.inputs.txt", scenario.name)),
                input_text,
            )
            .expect("write scenario inputs");
        }
    }

    fn write_minimal_expected_fixture(
        path: &Path,
        stem: &str,
        frame_count: usize,
        evidence_frame: Option<usize>,
    ) {
        let mut expected = String::from(crate::fidelity_trace_engine::trace_header());
        expected.push('\n');
        for frame in 1..=frame_count {
            let (sound_commands, events) = if evidence_frame == Some(frame) {
                ("0xE6,0xF5", "credit_added,game_started")
            } else {
                ("-", "-")
            };
            expected.push_str(&format!(
                "{frame}\t0x0000\t0x00\t0x00\t0x00\tattract\t0\t0\t1\t3\t3\t0x00\t0x00\t0x00\t0xE15D8394\t0xC4C53DA1\t0x05B7E865\t0x41D912FF\t0x157E98C7\t{sound_commands}\t{events}\n"
            ));
        }
        fs::write(path.join(format!("{stem}.expected.tsv")), expected)
            .expect("write expected fixture");
    }

    fn write_minimal_reference_fixtures(path: &Path, include_start_evidence: bool) {
        for entry in fs::read_dir(path).expect("read scenario input dir") {
            let input_path = entry.expect("dir entry").path();
            let Some(name) = input_path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some(stem) = name.strip_suffix(".inputs.txt") else {
                continue;
            };
            let frame_count = fs::read_to_string(&input_path)
                .expect("read inputs")
                .trim_end()
                .split(';')
                .count();
            let evidence_frame = if include_start_evidence && stem != "attract_boot" {
                Some(1)
            } else {
                None
            };
            write_minimal_expected_fixture(path, stem, frame_count, evidence_frame);
        }
    }
}
