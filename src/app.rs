//! CLI entrypoint for the clean-slate implementation.

use std::collections::{HashMap, HashSet};
#[cfg(not(any(test, coverage)))]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use anyhow::{Context, Result, anyhow, bail};

use crate::{
    audio::LiveAudioMode,
    fidelity::{
        compare_trace_text, expanded_trace_input_text, parse_trace_input_script, trace_header,
        trace_scenarios, trace_text_for_inputs,
    },
    input::{CabinetInput, InputProfile},
    live::run_live,
    presentation::PresentationBackend,
    rom::{RedLabelRomImages, load_verified_dir},
    wgpu_presenter::run_wgpu_live_smoke,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    PlayLive {
        input_profile: InputProfile,
        presentation_backend: PresentationBackend,
        audio_mode: LiveAudioMode,
        live_smoke: bool,
        cmos_path: Option<PathBuf>,
    },
    RomReport {
        path: Option<PathBuf>,
    },
    VerifyRoms {
        path: PathBuf,
    },
    FidelityTrace {
        frame_count: usize,
    },
    FidelityTraceInputs {
        script: String,
    },
    FidelityTraceInputsFile {
        path: PathBuf,
    },
    FidelityCheckTrace {
        inputs_path: PathBuf,
        expected_path: PathBuf,
    },
    FidelityCheckTraceDir {
        path: PathBuf,
    },
    FidelityListScenarios,
    FidelityWriteScenarioInputs {
        path: PathBuf,
    },
    FidelityCheckReferenceTraceDir {
        path: PathBuf,
    },
    Help,
}

#[cfg(not(any(test, coverage)))]
pub fn run() -> Result<()> {
    run_command(parse_args(env::args().skip(1))?)
}

#[cfg(any(test, coverage))]
pub fn run() -> Result<()> {
    run_command(Command::Help)
}

fn run_command(command: Command) -> Result<()> {
    match command {
        Command::PlayLive {
            input_profile,
            presentation_backend,
            audio_mode,
            live_smoke,
            cmos_path,
        } => {
            if live_smoke {
                if presentation_backend != PresentationBackend::Wgpu {
                    bail!("--live-smoke currently requires --renderer wgpu");
                }
                let report = run_wgpu_live_smoke(input_profile, cmos_path.as_deref())?;
                print!("{}", report.to_text());
                Ok(())
            } else {
                run_live(
                    input_profile,
                    presentation_backend,
                    audio_mode,
                    cmos_path.as_deref(),
                )
            }
        }
        Command::RomReport { path } => run_rom_report(path.as_deref()),
        Command::VerifyRoms { path } => run_verify_roms(&path),
        Command::FidelityTrace { frame_count } => {
            print!("{}", fidelity_trace_text(frame_count)?);
            Ok(())
        }
        Command::FidelityTraceInputs { script } => {
            print!("{}", fidelity_trace_input_text(&script)?);
            Ok(())
        }
        Command::FidelityTraceInputsFile { path } => {
            print!("{}", fidelity_trace_input_file_text(&path)?);
            Ok(())
        }
        Command::FidelityCheckTrace {
            inputs_path,
            expected_path,
        } => {
            print!(
                "{}",
                fidelity_check_trace_text(&inputs_path, &expected_path)?
            );
            Ok(())
        }
        Command::FidelityCheckTraceDir { path } => {
            print!("{}", fidelity_check_trace_dir_text(&path)?);
            Ok(())
        }
        Command::FidelityListScenarios => {
            print!("{}", fidelity_list_scenarios_text()?);
            Ok(())
        }
        Command::FidelityWriteScenarioInputs { path } => {
            print!("{}", fidelity_write_scenario_inputs_text(&path)?);
            Ok(())
        }
        Command::FidelityCheckReferenceTraceDir { path } => {
            print!("{}", fidelity_check_reference_trace_dir_text(&path)?);
            Ok(())
        }
        Command::Help => {
            print_help();
            Ok(())
        }
    }
}

fn run_rom_report(path: Option<&Path>) -> Result<()> {
    let Some(path) = path else {
        print!("{}", rom_listing_text());
        return Ok(());
    };

    let report = crate::rom::scan_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?;

    print!("{}", rom_report_text(&report));

    Ok(())
}

fn run_verify_roms(path: &Path) -> Result<()> {
    let verified = load_verified_dir(path)
        .with_context(|| format!("failed to inspect ROM directory {}", path.display()))?
        .map_err(|report| anyhow!("{}", rom_report_text(&report).trim_end()))?;
    let images = RedLabelRomImages::from_verified_rom_set(&verified)
        .map_err(|error| anyhow!("verified ROM set could not be mapped: {error}"))?;

    print!("{}", verify_roms_text(path, &verified, &images));

    Ok(())
}

fn rom_listing_text() -> String {
    let mut text = format!(
        "Expected Williams Defender red-label ROM filenames ({} files):\n",
        crate::rom::red_label_roms().len()
    );
    for descriptor in crate::rom::red_label_roms() {
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

fn verify_roms_text(
    path: &Path,
    verified: &crate::rom::VerifiedRomSet,
    images: &RedLabelRomImages,
) -> String {
    format!(
        "ROM set {} verified: {} files, {} bytes, {} mapped regions, {} mapped loads\n",
        path.display(),
        verified.files().len(),
        verified.total_bytes(),
        images.regions().len(),
        images.loads().len()
    )
}

fn fidelity_trace_text(frame_count: usize) -> Result<String> {
    if frame_count == 0 {
        bail!("--fidelity-trace frame count must be greater than zero");
    }

    let inputs = vec![CabinetInput::NONE; frame_count];
    trace_text_for_inputs(&inputs).map_err(|error| anyhow!(error))
}

fn fidelity_trace_input_text(script: &str) -> Result<String> {
    let inputs = parse_trace_input_script(script).map_err(|error| anyhow!(error))?;
    trace_text_for_inputs(&inputs).map_err(|error| anyhow!(error))
}

fn fidelity_trace_input_file_text(path: &Path) -> Result<String> {
    let script = fs::read_to_string(path)
        .with_context(|| format!("failed to read trace input script {}", path.display()))?;
    fidelity_trace_input_text(&script)
}

fn fidelity_check_trace_text(inputs_path: &Path, expected_path: &Path) -> Result<String> {
    let actual = fidelity_trace_input_file_text(inputs_path)?;
    let expected = fs::read_to_string(expected_path)
        .with_context(|| format!("failed to read expected trace {}", expected_path.display()))?;
    let comparison = compare_trace_text(&expected, &actual)
        .map_err(|mismatch| anyhow!("{}: {mismatch}", expected_path.display()))?;

    Ok(format!(
        "Fidelity trace {} matched {} frame(s)\n",
        expected_path.display(),
        comparison.frames
    ))
}

fn fidelity_check_trace_dir_text(path: &Path) -> Result<String> {
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

    let frames = check_trace_fixtures(&fixtures, RustTraceFixtureChecker)?;

    Ok(format!(
        "Fidelity trace fixture directory {} matched {} fixture(s), {} frame(s)\n",
        path.display(),
        fixtures.len(),
        frames
    ))
}

fn fidelity_list_scenarios_text() -> Result<String> {
    let scenarios = trace_scenarios().map_err(|error| anyhow!(error))?;
    let mut text = format!("Red-label Phase 1 trace scenarios ({}):\n", scenarios.len());
    for scenario in scenarios {
        text.push_str(&format!(
            "  {:<20} {:>4} frames  {}\n",
            scenario.scenario, scenario.frames, scenario.description
        ));
    }

    Ok(text)
}

fn fidelity_write_scenario_inputs_text(path: &Path) -> Result<String> {
    let scenarios = trace_scenarios().map_err(|error| anyhow!(error))?;
    fs::create_dir_all(path).with_context(|| {
        format!(
            "failed to create scenario input directory {}",
            path.display()
        )
    })?;

    for scenario in &scenarios {
        let input_text = expanded_trace_input_text(&scenario.input_program)
            .map_err(|error| anyhow!(error))
            .with_context(|| format!("failed to expand trace scenario {}", scenario.scenario))?;
        fs::write(
            path.join(format!("{}.inputs.txt", scenario.scenario)),
            input_text,
        )
        .with_context(|| {
            format!(
                "failed to write scenario input script for {}",
                scenario.scenario
            )
        })?;
    }

    Ok(format!(
        "Wrote {} Phase 1 trace scenario input script(s) to {}\n",
        scenarios.len(),
        path.display()
    ))
}

fn fidelity_check_reference_trace_dir_text(path: &Path) -> Result<String> {
    if !path.is_dir() {
        bail!(
            "reference trace fixture path {} is not a directory",
            path.display()
        );
    }

    let scenarios = trace_scenarios().map_err(|error| anyhow!(error))?;
    let scenario_names = scenarios
        .iter()
        .map(|scenario| scenario.scenario.as_str())
        .collect::<HashSet<_>>();
    let requirements = trace_requirements()?;
    validate_trace_requirements_reference_known_scenarios(&scenario_names, &requirements)?;
    let expected_header = trace_header();
    let mut frames = 0;
    for scenario in &scenarios {
        let inputs_path = path.join(format!("{}.inputs.txt", scenario.scenario));
        let expected_path = path.join(format!("{}.expected.tsv", scenario.scenario));
        let actual_inputs = fs::read_to_string(&inputs_path)
            .with_context(|| format!("failed to read {}", inputs_path.display()))?;
        let expected_inputs = expanded_trace_input_text(&scenario.input_program)
            .map_err(|error| anyhow!(error))
            .with_context(|| format!("failed to expand trace scenario {}", scenario.scenario))?;
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
        if trace_frames != scenario.frames {
            bail!(
                "reference trace fixture {} has {} frame(s), expected {}",
                expected_path.display(),
                trace_frames,
                scenario.frames
            );
        }
        check_reference_trace_required_cells(&expected_path, &lines)?;
        if let Some(requirement) = requirements.get(&scenario.scenario) {
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

fn trace_requirements() -> Result<HashMap<String, TraceRequirement>> {
    parse_trace_requirements(crate::assets::RED_LABEL_TRACE_REQUIREMENTS_TSV)
}

fn parse_trace_requirements(tsv: &str) -> Result<HashMap<String, TraceRequirement>> {
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
) -> Result<()> {
    for scenario in requirements.keys() {
        if !scenario_names.contains(scenario.as_str()) {
            bail!("trace requirement references unknown scenario {scenario}");
        }
    }
    Ok(())
}

fn check_reference_trace_required_cells(expected_path: &Path, lines: &[&str]) -> Result<()> {
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
        .collect::<Result<Vec<_>>>()?;

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
) -> Result<()> {
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

fn trace_column_index(columns: &[&str], column: &str) -> Result<usize> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceFixturePair {
    inputs_path: PathBuf,
    expected_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TraceFixtureCheck {
    frames: usize,
}

trait TraceFixtureChecker {
    fn check_fixture(&self, fixture: &TraceFixturePair) -> Result<TraceFixtureCheck>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RustTraceFixtureChecker;

impl TraceFixtureChecker for RustTraceFixtureChecker {
    fn check_fixture(&self, fixture: &TraceFixturePair) -> Result<TraceFixtureCheck> {
        let actual = fidelity_trace_input_file_text(&fixture.inputs_path)?;
        let expected = fs::read_to_string(&fixture.expected_path).with_context(|| {
            format!(
                "failed to read expected trace {}",
                fixture.expected_path.display()
            )
        })?;
        let comparison = compare_trace_text(&expected, &actual)
            .map_err(|mismatch| anyhow!("{}: {mismatch}", fixture.expected_path.display()))?;

        Ok(TraceFixtureCheck {
            frames: comparison.frames,
        })
    }
}

fn check_trace_fixtures<T>(fixtures: &[TraceFixturePair], checker: T) -> Result<usize>
where
    T: TraceFixtureChecker + Copy + Send + Sync,
{
    if fixtures.is_empty() {
        return Ok(0);
    }

    let workers = trace_fixture_worker_count(fixtures.len());
    let chunk_size = fixtures.len().div_ceil(workers);
    let mut checks = Vec::with_capacity(fixtures.len());

    thread::scope(|scope| -> Result<()> {
        let handles = fixtures
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|fixture| checker.check_fixture(fixture))
                        .collect::<Vec<_>>()
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
        frames += check?.frames;
    }

    Ok(frames)
}

fn trace_fixture_worker_count(fixture_count: usize) -> usize {
    let available = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    fixture_count.min(available).max(1)
}

fn trace_fixture_pairs(path: &Path) -> Result<Vec<TraceFixturePair>> {
    let mut fixtures = Vec::new();
    let mut unexpected_expected = Vec::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("failed to read fixture dir {}", path.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read fixture dir {}", path.display()))?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let Some(file_name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if let Some(stem) = file_name.strip_suffix(".inputs.txt") {
            let expected_path = path.join(format!("{stem}.expected.tsv"));
            if !expected_path.is_file() {
                bail!(
                    "fidelity trace fixture {} is missing expected trace {}",
                    entry_path.display(),
                    expected_path.display()
                );
            }
            fixtures.push(TraceFixturePair {
                inputs_path: entry_path,
                expected_path,
            });
        } else if file_name.ends_with(".expected.tsv") {
            unexpected_expected.push(entry_path);
        }
    }

    let scenario_order = trace_scenarios()
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
    for expected_path in unexpected_expected {
        let Some(file_name) = expected_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(stem) = file_name.strip_suffix(".expected.tsv") else {
            continue;
        };
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

fn rom_report_text(report: &crate::rom::RomReport) -> String {
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

fn parse_args<I>(args: I) -> Result<Command>
where
    I: IntoIterator<Item = String>,
{
    let mut input_profile = InputProfile::default();
    let mut presentation_backend = PresentationBackend::default();
    let mut audio_mode = LiveAudioMode::default();
    let mut live_smoke = false;
    let mut cmos_path = None;
    let mut args = args.into_iter().peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Command::Help),
            "--mute" => audio_mode = LiveAudioMode::Disabled,
            "--live-smoke" => live_smoke = true,
            "--input-profile" => {
                let Some(value) = args.next() else {
                    bail!("--input-profile requires one of: planetoid, cabinet, test");
                };
                input_profile = InputProfile::parse(&value)
                    .with_context(|| format!("unknown input profile: {value}"))?;
            }
            "--renderer" | "--presentation" => {
                let Some(value) = args.next() else {
                    bail!("--renderer requires one of: kitty, wgpu");
                };
                presentation_backend = PresentationBackend::parse(&value)
                    .with_context(|| format!("unknown renderer: {value}"))?;
            }
            "--cmos-path" => {
                let Some(value) = args.next() else {
                    bail!("--cmos-path requires a file path");
                };
                cmos_path = Some(PathBuf::from(value));
            }
            "--rom-report" => {
                let path = args.next().map(PathBuf::from);
                if args.next().is_some() {
                    bail!("--rom-report only accepts one optional path");
                }
                return Ok(Command::RomReport { path });
            }
            "--verify-roms" => {
                let Some(path) = args.next() else {
                    bail!("--verify-roms requires a ROM directory path");
                };
                if args.next().is_some() {
                    bail!("--verify-roms only accepts one directory path");
                }
                return Ok(Command::VerifyRoms {
                    path: PathBuf::from(path),
                });
            }
            "--fidelity-trace" => {
                let frame_count = match args.next() {
                    Some(value) => parse_frame_count(&value)?,
                    None => 1,
                };
                if args.next().is_some() {
                    bail!("--fidelity-trace only accepts one optional frame count");
                }
                return Ok(Command::FidelityTrace { frame_count });
            }
            "--fidelity-trace-inputs" => {
                let Some(script) = args.next() else {
                    bail!("--fidelity-trace-inputs requires a semicolon-separated input script");
                };
                if args.next().is_some() {
                    bail!("--fidelity-trace-inputs only accepts one input script");
                }
                return Ok(Command::FidelityTraceInputs { script });
            }
            "--fidelity-trace-inputs-file" => {
                let Some(path) = args.next() else {
                    bail!("--fidelity-trace-inputs-file requires a trace input script path");
                };
                if args.next().is_some() {
                    bail!("--fidelity-trace-inputs-file only accepts one path");
                }
                return Ok(Command::FidelityTraceInputsFile {
                    path: PathBuf::from(path),
                });
            }
            "--fidelity-check-trace" => {
                let Some(inputs_path) = args.next() else {
                    bail!(
                        "--fidelity-check-trace requires an input script path and expected trace path"
                    );
                };
                let Some(expected_path) = args.next() else {
                    bail!(
                        "--fidelity-check-trace requires an input script path and expected trace path"
                    );
                };
                if args.next().is_some() {
                    bail!(
                        "--fidelity-check-trace only accepts an input script path and expected trace path"
                    );
                }
                return Ok(Command::FidelityCheckTrace {
                    inputs_path: PathBuf::from(inputs_path),
                    expected_path: PathBuf::from(expected_path),
                });
            }
            "--fidelity-check-trace-dir" => {
                let Some(path) = args.next() else {
                    bail!("--fidelity-check-trace-dir requires a fixture directory path");
                };
                if args.next().is_some() {
                    bail!("--fidelity-check-trace-dir only accepts one fixture directory path");
                }
                return Ok(Command::FidelityCheckTraceDir {
                    path: PathBuf::from(path),
                });
            }
            "--fidelity-list-scenarios" => {
                if args.next().is_some() {
                    bail!("--fidelity-list-scenarios does not accept extra arguments");
                }
                return Ok(Command::FidelityListScenarios);
            }
            "--fidelity-write-scenario-inputs" => {
                let Some(path) = args.next() else {
                    bail!("--fidelity-write-scenario-inputs requires an output directory path");
                };
                if args.next().is_some() {
                    bail!(
                        "--fidelity-write-scenario-inputs only accepts one output directory path"
                    );
                }
                return Ok(Command::FidelityWriteScenarioInputs {
                    path: PathBuf::from(path),
                });
            }
            "--fidelity-check-reference-trace-dir" => {
                let Some(path) = args.next() else {
                    bail!(
                        "--fidelity-check-reference-trace-dir requires a reference fixture directory path"
                    );
                };
                if args.next().is_some() {
                    bail!(
                        "--fidelity-check-reference-trace-dir only accepts one reference fixture directory path"
                    );
                }
                return Ok(Command::FidelityCheckReferenceTraceDir {
                    path: PathBuf::from(path),
                });
            }
            other => bail!("unknown argument: {other}"),
        }
    }

    Ok(Command::PlayLive {
        input_profile,
        presentation_backend,
        audio_mode,
        live_smoke,
        cmos_path,
    })
}

fn parse_frame_count(value: &str) -> Result<usize> {
    let frame_count = value
        .parse::<usize>()
        .with_context(|| format!("invalid --fidelity-trace frame count: {value}"))?;
    if frame_count == 0 {
        bail!("--fidelity-trace frame count must be greater than zero");
    }

    Ok(frame_count)
}

fn print_help() {
    print!("{}", help_text());
}

fn help_text() -> &'static str {
    "defender\n  cargo run\n  cargo run -- --renderer wgpu\n  cargo run -- --renderer kitty\n  cargo run -- --live-smoke\n  cargo run -- --mute\n  cargo run -- --input-profile planetoid\n  cargo run -- --input-profile cabinet\n  cargo run -- --cmos-path ~/.local/state/defender/red-label-cmos.bin\n  cargo run -- --rom-report\n  cargo run -- --rom-report /path/to/roms\n  cargo run -- --verify-roms /path/to/roms\n  cargo run -- --fidelity-trace 300\n  cargo run -- --fidelity-trace-inputs 'coin,start_one;fire,thrust;none'\n  cargo run -- --fidelity-trace-inputs-file /path/to/inputs.txt\n  cargo run -- --fidelity-check-trace /path/to/inputs.txt /path/to/expected.tsv\n  cargo run -- --fidelity-check-trace-dir docs/fidelity/fixtures/local/rust-current\n  cargo run -- --fidelity-list-scenarios\n  cargo run -- --fidelity-write-scenario-inputs docs/fidelity/fixtures/local/reference\n  cargo run -- --fidelity-check-reference-trace-dir docs/fidelity/fixtures/local/reference\n\nRuntime assets are embedded in the binary for copy-only deployment.\nLive play defaults to the windowed wgpu backend; Kitty graphics remains available with --renderer kitty.\nLive audio routes accepted sound commands through a non-blocking null backend; --mute disables that runtime path.\n"
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};
    use std::{env, fs};

    use super::{
        Command, TraceFixtureCheck, TraceFixtureChecker, TraceFixturePair, TraceRequirement,
        check_reference_trace_evidence, check_reference_trace_required_cells, check_trace_fixtures,
        fidelity_check_reference_trace_dir_text, fidelity_check_trace_dir_text,
        fidelity_check_trace_text, fidelity_list_scenarios_text, fidelity_trace_input_file_text,
        fidelity_trace_input_text, fidelity_trace_text, fidelity_write_scenario_inputs_text,
        help_text, parse_args, parse_requirement_values, parse_trace_requirements,
        rom_listing_text, rom_report_text, run_command, trace_cell_values,
        trace_fixture_worker_count, validate_trace_requirements_reference_known_scenarios,
    };
    use crate::audio::LiveAudioMode;
    use crate::fidelity::{expanded_trace_input_text, trace_header};
    use crate::input::InputProfile;
    use crate::presentation::PresentationBackend;
    use crate::rom::RomReport;
    use anyhow::{Result, anyhow};

    #[derive(Debug, Clone, Copy)]
    struct FakeTraceFixtureChecker;

    impl TraceFixtureChecker for FakeTraceFixtureChecker {
        fn check_fixture(&self, fixture: &TraceFixturePair) -> Result<TraceFixtureCheck> {
            let file_name = fixture
                .inputs_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if file_name.contains("bad") {
                return Err(anyhow!("{file_name} failed"));
            }

            Ok(TraceFixtureCheck { frames: 2 })
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct PanickingTraceFixtureChecker;

    impl TraceFixtureChecker for PanickingTraceFixtureChecker {
        fn check_fixture(&self, _fixture: &TraceFixturePair) -> Result<TraceFixtureCheck> {
            panic!("intentional fixture worker panic")
        }
    }

    fn write_minimal_expected_fixture(
        path: &Path,
        stem: &str,
        frame_count: usize,
        evidence_frame: Option<usize>,
    ) {
        let mut expected = String::from(trace_header());
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

    #[test]
    fn parse_args_defaults_to_live_planetoid() {
        let command = parse_args(Vec::<String>::new()).expect("parse args");
        assert_eq!(
            command,
            Command::PlayLive {
                input_profile: InputProfile::Planetoid,
                presentation_backend: PresentationBackend::Wgpu,
                audio_mode: LiveAudioMode::Null,
                live_smoke: false,
                cmos_path: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_mute_audio_flag() {
        let command = parse_args(vec![String::from("--mute")]).expect("parse args");

        assert_eq!(
            command,
            Command::PlayLive {
                input_profile: InputProfile::Planetoid,
                presentation_backend: PresentationBackend::Wgpu,
                audio_mode: LiveAudioMode::Disabled,
                live_smoke: false,
                cmos_path: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_live_renderer_backend() {
        let command = parse_args(vec![
            String::from("--renderer"),
            String::from("wgpu"),
            String::from("--input-profile"),
            String::from("cabinet"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::PlayLive {
                input_profile: InputProfile::Cabinet,
                presentation_backend: PresentationBackend::Wgpu,
                audio_mode: LiveAudioMode::Null,
                live_smoke: false,
                cmos_path: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_wgpu_live_smoke() {
        let command = parse_args(vec![String::from("--live-smoke")]).expect("parse args");

        assert_eq!(
            command,
            Command::PlayLive {
                input_profile: InputProfile::Planetoid,
                presentation_backend: PresentationBackend::Wgpu,
                audio_mode: LiveAudioMode::Null,
                live_smoke: true,
                cmos_path: None,
            }
        );
    }

    #[test]
    fn parse_args_accepts_live_cmos_path() {
        let command = parse_args(vec![
            String::from("--cmos-path"),
            String::from("/tmp/defender-cmos.bin"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::PlayLive {
                input_profile: InputProfile::Planetoid,
                presentation_backend: PresentationBackend::Wgpu,
                audio_mode: LiveAudioMode::Null,
                live_smoke: false,
                cmos_path: Some(PathBuf::from("/tmp/defender-cmos.bin")),
            }
        );
    }

    #[test]
    fn run_command_executes_wgpu_live_smoke_stub() {
        run_command(Command::PlayLive {
            input_profile: InputProfile::Planetoid,
            presentation_backend: PresentationBackend::Wgpu,
            audio_mode: LiveAudioMode::Null,
            live_smoke: true,
            cmos_path: None,
        })
        .expect("run live smoke stub");
    }

    #[test]
    fn run_command_rejects_live_smoke_with_non_wgpu_backend() {
        let error = run_command(Command::PlayLive {
            input_profile: InputProfile::Planetoid,
            presentation_backend: PresentationBackend::Kitty,
            audio_mode: LiveAudioMode::Null,
            live_smoke: true,
            cmos_path: None,
        })
        .expect_err("kitty live smoke should be rejected");

        assert!(error.to_string().contains("--renderer wgpu"));
    }

    #[test]
    fn run_command_dispatches_normal_live_backend() {
        run_command(Command::PlayLive {
            input_profile: InputProfile::Cabinet,
            presentation_backend: PresentationBackend::Kitty,
            audio_mode: LiveAudioMode::Disabled,
            live_smoke: false,
            cmos_path: None,
        })
        .expect("run live backend stub");
    }

    #[test]
    fn run_test_entrypoint_executes_help_stub() {
        super::run().expect("run test entrypoint");
    }

    #[test]
    fn parse_args_supports_rom_report_without_directory() {
        let command = parse_args(vec![String::from("--rom-report")]).expect("parse args");
        assert_eq!(command, Command::RomReport { path: None });
    }

    #[test]
    fn parse_args_uses_rom_report_directory() {
        let command = parse_args(vec![
            String::from("--rom-report"),
            String::from("/tmp/defender"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::RomReport {
                path: Some(PathBuf::from("/tmp/defender")),
            }
        );
    }

    #[test]
    fn parse_args_supports_verify_roms_directory() {
        let command = parse_args(vec![
            String::from("--verify-roms"),
            String::from("/tmp/defender"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::VerifyRoms {
                path: PathBuf::from("/tmp/defender"),
            }
        );
    }

    #[test]
    fn parse_args_supports_fidelity_trace_default_and_frame_count() {
        let default_command =
            parse_args(vec![String::from("--fidelity-trace")]).expect("parse args");
        assert_eq!(default_command, Command::FidelityTrace { frame_count: 1 });

        let counted_command =
            parse_args(vec![String::from("--fidelity-trace"), String::from("300")])
                .expect("parse args");
        assert_eq!(counted_command, Command::FidelityTrace { frame_count: 300 });
    }

    #[test]
    fn parse_args_supports_fidelity_trace_input_script() {
        let command = parse_args(vec![
            String::from("--fidelity-trace-inputs"),
            String::from("coin,start_one;fire"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::FidelityTraceInputs {
                script: String::from("coin,start_one;fire"),
            }
        );
    }

    #[test]
    fn parse_args_supports_fidelity_trace_input_file() {
        let command = parse_args(vec![
            String::from("--fidelity-trace-inputs-file"),
            String::from("/tmp/defender-inputs.txt"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::FidelityTraceInputsFile {
                path: PathBuf::from("/tmp/defender-inputs.txt"),
            }
        );
    }

    #[test]
    fn parse_args_supports_fidelity_trace_check() {
        let command = parse_args(vec![
            String::from("--fidelity-check-trace"),
            String::from("/tmp/defender-inputs.txt"),
            String::from("/tmp/defender-expected.tsv"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::FidelityCheckTrace {
                inputs_path: PathBuf::from("/tmp/defender-inputs.txt"),
                expected_path: PathBuf::from("/tmp/defender-expected.tsv"),
            }
        );
    }

    #[test]
    fn parse_args_supports_fidelity_trace_check_dir() {
        let command = parse_args(vec![
            String::from("--fidelity-check-trace-dir"),
            String::from("/tmp/defender-fixtures"),
        ])
        .expect("parse args");

        assert_eq!(
            command,
            Command::FidelityCheckTraceDir {
                path: PathBuf::from("/tmp/defender-fixtures"),
            }
        );
    }

    #[test]
    fn parse_args_supports_phase_one_reference_commands() {
        let command =
            parse_args(vec![String::from("--fidelity-list-scenarios")]).expect("parse args");
        assert_eq!(command, Command::FidelityListScenarios);

        let command = parse_args(vec![
            String::from("--fidelity-write-scenario-inputs"),
            String::from("/tmp/defender-fixtures"),
        ])
        .expect("parse args");
        assert_eq!(
            command,
            Command::FidelityWriteScenarioInputs {
                path: PathBuf::from("/tmp/defender-fixtures"),
            }
        );

        let command = parse_args(vec![
            String::from("--fidelity-check-reference-trace-dir"),
            String::from("/tmp/defender-fixtures"),
        ])
        .expect("parse args");
        assert_eq!(
            command,
            Command::FidelityCheckReferenceTraceDir {
                path: PathBuf::from("/tmp/defender-fixtures"),
            }
        );
    }

    #[test]
    fn parse_args_rejects_unknown_flags() {
        let error = parse_args(vec![String::from("--wat")]).expect_err("parse args");
        assert!(error.to_string().contains("unknown argument"));
    }

    #[test]
    fn parse_args_requires_profile_value() {
        let error = parse_args(vec![String::from("--input-profile")]).expect_err("parse args");
        assert!(error.to_string().contains("--input-profile requires"));
    }

    #[test]
    fn parse_args_requires_verify_roms_directory() {
        let error = parse_args(vec![String::from("--verify-roms")]).expect_err("parse args");
        assert!(error.to_string().contains("--verify-roms requires"));
    }

    #[test]
    fn parse_args_requires_fidelity_trace_input_script() {
        let error =
            parse_args(vec![String::from("--fidelity-trace-inputs")]).expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-trace-inputs requires")
        );
    }

    #[test]
    fn parse_args_requires_fidelity_trace_input_file_path() {
        let error =
            parse_args(vec![String::from("--fidelity-trace-inputs-file")]).expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-trace-inputs-file requires")
        );
    }

    #[test]
    fn parse_args_requires_fidelity_check_trace_paths() {
        let error =
            parse_args(vec![String::from("--fidelity-check-trace")]).expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-check-trace requires")
        );

        let error = parse_args(vec![
            String::from("--fidelity-check-trace"),
            String::from("/tmp/defender-inputs.txt"),
        ])
        .expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-check-trace requires")
        );
    }

    #[test]
    fn parse_args_requires_fidelity_check_trace_dir_path() {
        let error =
            parse_args(vec![String::from("--fidelity-check-trace-dir")]).expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-check-trace-dir requires")
        );
    }

    #[test]
    fn parse_args_requires_phase_one_reference_paths() {
        let error = parse_args(vec![String::from("--fidelity-write-scenario-inputs")])
            .expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-write-scenario-inputs requires")
        );

        let error = parse_args(vec![String::from("--fidelity-check-reference-trace-dir")])
            .expect_err("parse args");
        assert!(
            error
                .to_string()
                .contains("--fidelity-check-reference-trace-dir requires")
        );
    }

    #[test]
    fn parse_args_rejects_zero_or_bad_fidelity_trace_counts() {
        let error = parse_args(vec![String::from("--fidelity-trace"), String::from("0")])
            .expect_err("parse args");
        assert!(error.to_string().contains("greater than zero"));

        let error = parse_args(vec![String::from("--fidelity-trace"), String::from("wat")])
            .expect_err("parse args");
        assert!(error.to_string().contains("invalid --fidelity-trace"));
    }

    #[test]
    fn help_text_documents_self_contained_runtime_and_profiles() {
        let text = help_text();

        assert!(text.contains("--input-profile planetoid"));
        assert!(text.contains("--renderer kitty"));
        assert!(text.contains("--renderer wgpu"));
        assert!(text.contains("--live-smoke"));
        assert!(text.contains("--mute"));
        assert!(text.contains("--verify-roms"));
        assert!(text.contains("--fidelity-trace 300"));
        assert!(text.contains("--fidelity-trace-inputs"));
        assert!(text.contains("--fidelity-trace-inputs-file"));
        assert!(text.contains("--fidelity-check-trace"));
        assert!(text.contains("--fidelity-check-trace-dir"));
        assert!(text.contains("--fidelity-list-scenarios"));
        assert!(text.contains("--fidelity-write-scenario-inputs"));
        assert!(text.contains("--fidelity-check-reference-trace-dir"));
        assert!(text.contains("docs/fidelity/fixtures/local/rust-current"));
        assert!(text.contains("docs/fidelity/fixtures/local/reference"));
        assert!(text.contains("copy-only deployment"));
        assert!(text.contains("defaults to the windowed wgpu"));
        assert!(text.contains("Kitty graphics"));
        assert!(text.contains("wgpu"));
    }

    #[test]
    fn rom_listing_text_includes_crc_metadata_and_verification_note() {
        let text = rom_listing_text();

        assert!(text.contains("defend.1"));
        assert!(text.contains("crc c3e52d7e"));
        assert!(text.contains("self-contained"));
    }

    #[test]
    fn rom_report_text_includes_all_non_empty_sections() {
        let report = RomReport {
            directory: PathBuf::from("/tmp/roms"),
            expected: 14,
            found: vec![String::from("defend.1")],
            missing: vec![String::from("defend.2")],
            unexpected: vec![String::from("notes.txt")],
            wrong_size: vec![String::from("defend.1 expected 2048 bytes got 0")],
            wrong_crc: vec![String::from("defend.1 expected c3e52d7e got 00000000")],
        };

        let text = rom_report_text(&report);

        assert!(text.contains("1/14"));
        assert!(text.contains("Missing: defend.2"));
        assert!(text.contains("Wrong size: defend.1"));
        assert!(text.contains("Wrong CRC: defend.1"));
        assert!(text.contains("Unexpected: notes.txt"));
    }

    #[test]
    fn rom_report_text_omits_empty_sections() {
        let report = RomReport {
            directory: PathBuf::from("/tmp/roms"),
            expected: 14,
            found: Vec::new(),
            missing: Vec::new(),
            unexpected: Vec::new(),
            wrong_size: Vec::new(),
            wrong_crc: Vec::new(),
        };

        let text = rom_report_text(&report);

        assert!(!text.contains("Missing:"));
        assert!(!text.contains("Wrong size:"));
        assert!(!text.contains("Wrong CRC:"));
        assert!(!text.contains("Unexpected:"));
    }

    #[test]
    fn fidelity_trace_text_emits_header_and_requested_frame_count() {
        let text = fidelity_trace_text(2).expect("trace text");
        let lines = text.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("video_crc32"));
        assert!(lines[1].starts_with("1\t"));
        assert!(lines[2].starts_with("2\t"));
    }

    #[test]
    fn fidelity_trace_input_text_applies_scripted_cabinet_inputs() {
        let text = fidelity_trace_input_text("coin,start_one;fire").expect("trace text");
        let lines = text.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("\t0x0003\t0x20\t0x00\t0x10\t"));
        assert!(lines[2].contains("\t0x0080\t0x01\t0x00\t0x00\t"));
    }

    #[test]
    fn fidelity_trace_input_file_text_reads_scripted_cabinet_inputs() {
        let path =
            env::temp_dir().join(format!("defender-trace-inputs-{}.txt", std::process::id()));
        fs::write(&path, "coin,start_one;fire").expect("write trace input file");

        let text = fidelity_trace_input_file_text(&path).expect("trace text");
        let _ = fs::remove_file(&path);
        let lines = text.lines().collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("\t0x0003\t0x20\t0x00\t0x10\t"));
        assert!(lines[2].contains("\t0x0080\t0x01\t0x00\t0x00\t"));
    }

    #[test]
    fn fidelity_list_scenarios_text_reports_phase_one_manifest() {
        let text = fidelity_list_scenarios_text().expect("scenario list");

        assert!(text.contains("Phase 1 trace scenarios"));
        assert!(text.contains("attract_boot"));
        assert!(text.contains("high_score_entry"));
    }

    #[test]
    fn fidelity_write_scenario_inputs_text_writes_expanded_inputs() {
        let path = env::temp_dir().join(format!("defender-scenario-inputs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);

        let text = fidelity_write_scenario_inputs_text(&path).expect("write scenario inputs");
        let attract =
            fs::read_to_string(path.join("attract_boot.inputs.txt")).expect("read attract inputs");
        let _ = fs::remove_dir_all(&path);

        assert!(text.contains("12 Phase 1 trace scenario input script(s)"));
        assert_eq!(
            attract,
            expanded_trace_input_text("none*900").expect("expanded attract")
        );
    }

    #[test]
    fn fidelity_check_reference_trace_dir_text_validates_required_phase_one_fixtures() {
        let path = env::temp_dir().join(format!(
            "defender-reference-fixtures-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fidelity_write_scenario_inputs_text(&path).expect("write scenario inputs");
        write_minimal_reference_fixtures(&path, true);

        let text =
            fidelity_check_reference_trace_dir_text(&path).expect("reference fixtures complete");
        let _ = fs::remove_dir_all(&path);

        assert!(text.contains("12 complete Phase 1 fixture(s)"));
    }

    #[test]
    fn fidelity_check_reference_trace_dir_text_rejects_missing_required_start_evidence() {
        let path = env::temp_dir().join(format!(
            "defender-reference-fixtures-missing-evidence-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fidelity_write_scenario_inputs_text(&path).expect("write scenario inputs");
        write_minimal_reference_fixtures(&path, false);

        let error =
            fidelity_check_reference_trace_dir_text(&path).expect_err("missing start evidence");
        let _ = fs::remove_dir_all(&path);

        assert!(
            error
                .to_string()
                .contains("missing required sound command 0xE6")
        );
    }

    #[test]
    fn fidelity_check_reference_trace_dir_text_rejects_late_start_evidence() {
        let path = env::temp_dir().join(format!(
            "defender-reference-fixtures-late-evidence-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fidelity_write_scenario_inputs_text(&path).expect("write scenario inputs");
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

        let error =
            fidelity_check_reference_trace_dir_text(&path).expect_err("late start evidence");
        let _ = fs::remove_dir_all(&path);

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
            trace_header(),
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
    fn check_reference_trace_evidence_rejects_missing_required_event() {
        let lines = [
            trace_header(),
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
            trace_header(),
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
            trace_header(),
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
    fn fidelity_check_reference_trace_dir_text_rejects_header_drift() {
        let path = env::temp_dir().join(format!(
            "defender-reference-fixtures-bad-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fidelity_write_scenario_inputs_text(&path).expect("write scenario inputs");
        fs::write(path.join("attract_boot.expected.tsv"), "bad\theader\n")
            .expect("write bad expected fixture");

        let error = fidelity_check_reference_trace_dir_text(&path).expect_err("header drift");
        let _ = fs::remove_dir_all(&path);

        assert!(error.to_string().contains("header does not match"));
    }

    #[test]
    fn fidelity_check_trace_text_compares_expected_trace_file() {
        let inputs_path = env::temp_dir().join(format!(
            "defender-trace-check-inputs-{}.txt",
            std::process::id()
        ));
        let expected_path = env::temp_dir().join(format!(
            "defender-trace-check-expected-{}.tsv",
            std::process::id()
        ));
        fs::write(&inputs_path, "coin,start_one;fire").expect("write trace input file");
        fs::write(
            &expected_path,
            fidelity_trace_input_text("coin,start_one;fire").expect("trace text"),
        )
        .expect("write expected trace");

        let text =
            fidelity_check_trace_text(&inputs_path, &expected_path).expect("trace should match");
        let _ = fs::remove_file(&inputs_path);
        let _ = fs::remove_file(&expected_path);

        assert!(text.contains("matched 2 frame(s)"));
    }

    #[test]
    fn fidelity_check_trace_text_reports_mismatch() {
        let inputs_path = env::temp_dir().join(format!(
            "defender-trace-check-bad-inputs-{}.txt",
            std::process::id()
        ));
        let expected_path = env::temp_dir().join(format!(
            "defender-trace-check-bad-expected-{}.tsv",
            std::process::id()
        ));
        fs::write(&inputs_path, "coin,start_one").expect("write trace input file");
        fs::write(&expected_path, "not\ta\ttrace\n").expect("write expected trace");

        let error =
            fidelity_check_trace_text(&inputs_path, &expected_path).expect_err("trace mismatch");
        let _ = fs::remove_file(&inputs_path);
        let _ = fs::remove_file(&expected_path);

        assert!(error.to_string().contains("trace mismatch at line 1"));
    }

    fn assert_local_reference_trace_matches(scenario: &str) {
        let fixture_dir = Path::new("docs/fidelity/fixtures/local/reference");
        let inputs_path = fixture_dir.join(format!("{scenario}.inputs.txt"));
        let expected_path = fixture_dir.join(format!("{scenario}.expected.tsv"));
        if !inputs_path.exists() || !expected_path.exists() {
            eprintln!(
                "local reference fixture pair for {scenario} is missing under {}; skipped",
                fixture_dir.display()
            );
            return;
        }

        fidelity_check_trace_text(&inputs_path, &expected_path)
            .unwrap_or_else(|error| panic!("{scenario} local reference mismatch: {error:#}"));
    }

    #[test]
    fn local_reference_attract_boot_matches_red_label() {
        assert_local_reference_trace_matches("attract_boot");
    }

    #[test]
    fn local_reference_start_game_matches_red_label() {
        assert_local_reference_trace_matches("start_game");
    }

    #[test]
    fn local_reference_first_300_frames_matches_red_label() {
        assert_local_reference_trace_matches("first_300_frames");
    }

    #[test]
    fn local_reference_firing_matches_red_label() {
        assert_local_reference_trace_matches("firing");
    }

    #[test]
    fn local_reference_thrust_reverse_matches_red_label() {
        assert_local_reference_trace_matches("thrust_reverse");
    }

    #[test]
    fn local_reference_smart_bomb_matches_red_label() {
        assert_local_reference_trace_matches("smart_bomb");
    }

    #[test]
    fn local_reference_hyperspace_matches_red_label() {
        assert_local_reference_trace_matches("hyperspace");
    }

    #[test]
    fn local_reference_abduction_matches_red_label() {
        assert_local_reference_trace_matches("abduction");
    }

    #[test]
    fn local_reference_death_matches_red_label() {
        assert_local_reference_trace_matches("death");
    }

    #[test]
    fn local_reference_wave_advance_matches_red_label() {
        assert_local_reference_trace_matches("wave_advance");
    }

    #[test]
    fn local_reference_planet_destruction_matches_red_label() {
        assert_local_reference_trace_matches("planet_destruction");
    }

    #[test]
    fn local_reference_high_score_entry_matches_red_label() {
        assert_local_reference_trace_matches("high_score_entry");
    }

    #[test]
    fn fidelity_check_trace_dir_text_skips_missing_directory() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-missing-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);

        let text = fidelity_check_trace_dir_text(&path).expect("missing dir skips");

        assert!(text.contains("not found; skipped"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_skips_empty_directory() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-empty-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");

        let text = fidelity_check_trace_dir_text(&path).expect("empty dir skips");
        let _ = fs::remove_dir_all(&path);

        assert!(text.contains("has no *.inputs.txt fixtures; skipped"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_compares_fixture_pairs() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-match-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("attract.inputs.txt"), "coin,start_one;fire")
            .expect("write fixture input");
        fs::write(
            path.join("attract.expected.tsv"),
            fidelity_trace_input_text("coin,start_one;fire").expect("trace text"),
        )
        .expect("write expected trace");

        let text = fidelity_check_trace_dir_text(&path).expect("fixture dir should match");
        let _ = fs::remove_dir_all(&path);

        assert!(text.contains("matched 1 fixture(s), 2 frame(s)"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_names_mismatched_fixture() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-mismatch-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("start.inputs.txt"), "coin,start_one").expect("write fixture input");
        fs::write(path.join("start.expected.tsv"), "not\ta\ttrace\n")
            .expect("write expected trace");

        let error = fidelity_check_trace_dir_text(&path).expect_err("trace mismatch");
        let _ = fs::remove_dir_all(&path);
        let message = error.to_string();

        assert!(message.contains("start.expected.tsv"));
        assert!(message.contains("trace mismatch at line 1"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_checks_phase_one_fixtures_in_manifest_order() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-order-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("abduction.inputs.txt"), "coin,start_one")
            .expect("write abduction input");
        fs::write(path.join("abduction.expected.tsv"), "bad\tabduction\n")
            .expect("write abduction expected trace");
        fs::write(path.join("attract_boot.inputs.txt"), "none").expect("write attract input");
        fs::write(path.join("attract_boot.expected.tsv"), "bad\tattract\n")
            .expect("write attract expected trace");

        let error = fidelity_check_trace_dir_text(&path).expect_err("trace mismatch");
        let _ = fs::remove_dir_all(&path);
        let message = error.to_string();

        assert!(message.contains("attract_boot.expected.tsv"));
        assert!(!message.contains("abduction.expected.tsv"));
    }

    #[test]
    fn check_trace_fixtures_sums_parallel_results_in_fixture_order() {
        let fixtures = vec![
            TraceFixturePair {
                inputs_path: PathBuf::from("first.inputs.txt"),
                expected_path: PathBuf::from("first.expected.tsv"),
            },
            TraceFixturePair {
                inputs_path: PathBuf::from("second.inputs.txt"),
                expected_path: PathBuf::from("second.expected.tsv"),
            },
        ];

        let frames = check_trace_fixtures(&fixtures, FakeTraceFixtureChecker)
            .expect("parallel fixture checks should pass");

        assert_eq!(frames, 4);
        assert_eq!(
            check_trace_fixtures(&[], FakeTraceFixtureChecker).expect("empty fixture set"),
            0
        );
        assert_eq!(trace_fixture_worker_count(0), 1);
    }

    #[test]
    fn check_trace_fixtures_preserves_first_error_in_fixture_order() {
        let fixtures = vec![
            TraceFixturePair {
                inputs_path: PathBuf::from("first-good.inputs.txt"),
                expected_path: PathBuf::from("first-good.expected.tsv"),
            },
            TraceFixturePair {
                inputs_path: PathBuf::from("second-bad.inputs.txt"),
                expected_path: PathBuf::from("second-bad.expected.tsv"),
            },
            TraceFixturePair {
                inputs_path: PathBuf::from("third-bad.inputs.txt"),
                expected_path: PathBuf::from("third-bad.expected.tsv"),
            },
        ];

        let error =
            check_trace_fixtures(&fixtures, FakeTraceFixtureChecker).expect_err("ordered error");
        let message = error.to_string();

        assert!(message.contains("second-bad.inputs.txt"));
        assert!(!message.contains("third-bad.inputs.txt"));
    }

    #[test]
    fn check_trace_fixtures_reports_worker_panic() {
        let fixtures = vec![TraceFixturePair {
            inputs_path: PathBuf::from("panic.inputs.txt"),
            expected_path: PathBuf::from("panic.expected.tsv"),
        }];

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let result = check_trace_fixtures(&fixtures, PanickingTraceFixtureChecker);
        std::panic::set_hook(previous_hook);
        let error = result.expect_err("worker panic should be reported");

        assert!(error.to_string().contains("worker panicked"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_rejects_input_without_expected_trace() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-missing-expected-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.inputs.txt"), "none").expect("write fixture input");

        let error = fidelity_check_trace_dir_text(&path).expect_err("missing expected trace");
        let _ = fs::remove_dir_all(&path);

        assert!(error.to_string().contains("missing expected trace"));
    }

    #[test]
    fn fidelity_check_trace_dir_text_rejects_expected_without_input_script() {
        let path = env::temp_dir().join(format!(
            "defender-trace-fixtures-missing-input-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create fixture dir");
        fs::write(path.join("boot.expected.tsv"), "frame\tinputs_bits\n")
            .expect("write expected trace");

        let error = fidelity_check_trace_dir_text(&path).expect_err("missing input script");
        let _ = fs::remove_dir_all(&path);

        assert!(error.to_string().contains("missing input script"));
    }

    #[test]
    fn fidelity_trace_text_rejects_zero_frames() {
        let error = fidelity_trace_text(0).expect_err("trace text");
        assert!(error.to_string().contains("greater than zero"));
    }
}
