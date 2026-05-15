//! Runtime-facing fidelity scenario manifest facade.
//!
//! Scenario commands are developer verification tools. This module keeps their
//! listing and input expansion on clean contracts while trace fixture data still
//! lives in the evidence tree.

use anyhow::anyhow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FidelityScenario {
    pub(crate) name: String,
    pub(crate) frame_count: usize,
    pub(crate) input_program: String,
    pub(crate) description: String,
}

impl From<crate::legacy_fidelity::TraceScenario> for FidelityScenario {
    fn from(scenario: crate::legacy_fidelity::TraceScenario) -> Self {
        Self {
            name: scenario.scenario,
            frame_count: scenario.frames,
            input_program: scenario.input_program,
            description: scenario.description,
        }
    }
}

pub(crate) fn scenarios() -> anyhow::Result<Vec<FidelityScenario>> {
    crate::legacy_fidelity::trace_scenarios()
        .map(|scenarios| scenarios.into_iter().map(FidelityScenario::from).collect())
        .map_err(|error| anyhow!(error))
}

pub(crate) fn expanded_input_text(input_program: &str) -> anyhow::Result<String> {
    crate::legacy_fidelity::expanded_trace_input_text(input_program).map_err(|error| anyhow!(error))
}

#[cfg(test)]
mod tests {
    use super::{expanded_input_text, scenarios};

    #[test]
    fn scenarios_adapt_trace_manifest_contract() {
        let scenarios = scenarios().expect("scenario manifest");

        assert_eq!(scenarios.len(), 12);
        assert_eq!(scenarios[0].name, "attract_boot");
        assert_eq!(scenarios[0].frame_count, 900);
        assert_eq!(scenarios[0].input_program, "none*900");
        assert_eq!(
            scenarios[0].description,
            "Source CMOS default boot/attract readiness window before credited play"
        );
        assert_eq!(scenarios[11].name, "high_score_entry");
    }

    #[test]
    fn expanded_input_text_preserves_current_program_expansion() {
        assert_eq!(
            expanded_input_text("none*2").expect("expanded inputs"),
            "none;none\n"
        );
    }
}
