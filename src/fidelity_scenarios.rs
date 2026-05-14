//! Runtime-owned fidelity scenario command facade.

use std::{fs, path::Path};

use anyhow::{Context, anyhow};

pub(crate) fn run_list() -> anyhow::Result<()> {
    print!("{}", listing_text()?);
    Ok(())
}

pub(crate) fn run_write_inputs(path: &Path) -> anyhow::Result<()> {
    print!("{}", write_inputs_text(path)?);
    Ok(())
}

fn listing_text() -> anyhow::Result<String> {
    let scenarios = scenarios()?;
    let mut text = format!("Red-label Phase 1 trace scenarios ({}):\n", scenarios.len());
    for scenario in scenarios {
        text.push_str(&format!(
            "  {:<20} {:>4} frames  {}\n",
            scenario.scenario, scenario.frames, scenario.description
        ));
    }

    Ok(text)
}

fn write_inputs_text(path: &Path) -> anyhow::Result<String> {
    let scenarios = scenarios()?;
    fs::create_dir_all(path).with_context(|| {
        format!(
            "failed to create scenario input directory {}",
            path.display()
        )
    })?;

    for scenario in &scenarios {
        let input_text = crate::legacy_fidelity::expanded_trace_input_text(&scenario.input_program)
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

fn scenarios() -> anyhow::Result<Vec<crate::legacy_fidelity::TraceScenario>> {
    crate::legacy_fidelity::trace_scenarios().map_err(|error| anyhow!(error))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{listing_text, write_inputs_text};

    #[test]
    fn listing_text_preserves_current_manifest_contract() {
        let text = listing_text().expect("scenario list");

        assert!(text.starts_with("Red-label Phase 1 trace scenarios (12):"));
        assert!(text.contains("attract_boot          900 frames"));
        assert!(text.contains("high_score_entry     3428 frames"));
        assert!(text.contains("Source CMOS default boot/attract readiness window"));
        assert!(text.ends_with("state when reached\n"));
    }

    #[test]
    fn write_inputs_text_writes_expanded_manifest_inputs() {
        let path = unique_temp_dir("defender-clean-scenario-inputs");
        let _ = fs::remove_dir_all(&path);

        let text = write_inputs_text(&path).expect("write scenario inputs");
        let attract =
            fs::read_to_string(path.join("attract_boot.inputs.txt")).expect("read attract inputs");
        let high_score_exists = path.join("high_score_entry.inputs.txt").is_file();
        let _ = fs::remove_dir_all(&path);

        assert!(text.contains("Wrote 12 Phase 1 trace scenario input script(s)"));
        assert_eq!(
            attract,
            crate::legacy_fidelity::expanded_trace_input_text("none*900")
                .expect("expanded attract")
        );
        assert!(high_score_exists);
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
