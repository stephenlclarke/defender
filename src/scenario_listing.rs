//! Runtime-owned fidelity scenario list command facade.

use anyhow::anyhow;

pub(crate) fn run() -> anyhow::Result<()> {
    print!("{}", listing_text()?);
    Ok(())
}

fn listing_text() -> anyhow::Result<String> {
    let scenarios = crate::legacy_fidelity::trace_scenarios().map_err(|error| anyhow!(error))?;
    let mut text = format!("Red-label Phase 1 trace scenarios ({}):\n", scenarios.len());
    for scenario in scenarios {
        text.push_str(&format!(
            "  {:<20} {:>4} frames  {}\n",
            scenario.scenario, scenario.frames, scenario.description
        ));
    }

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::listing_text;

    #[test]
    fn listing_text_preserves_current_manifest_contract() {
        let text = listing_text().expect("scenario list");

        assert!(text.starts_with("Red-label Phase 1 trace scenarios (12):"));
        assert!(text.contains("attract_boot          900 frames"));
        assert!(text.contains("high_score_entry     3428 frames"));
        assert!(text.contains("Source CMOS default boot/attract readiness window"));
        assert!(text.ends_with("state when reached\n"));
    }
}
