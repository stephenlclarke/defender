//! Runtime-facing fidelity trace engine facade.
//!
//! Trace commands are developer verification tools. This module keeps trace
//! generation and comparison behind clean result contracts while the temporary
//! oracle still supplies the trace engine.

use anyhow::anyhow;

pub(crate) fn trace_header() -> &'static str {
    crate::legacy_fidelity::trace_header()
}

pub(crate) fn trace_text_for_script(script: &str) -> anyhow::Result<String> {
    let inputs =
        crate::legacy_fidelity::parse_trace_input_script(script).map_err(|error| anyhow!(error))?;
    crate::legacy_fidelity::trace_text_for_inputs(&inputs).map_err(|error| anyhow!(error))
}

pub(crate) fn compare_trace_text(expected: &str, actual: &str) -> anyhow::Result<usize> {
    crate::legacy_fidelity::compare_trace_text(expected, actual)
        .map(|comparison| comparison.frames)
        .map_err(|mismatch| anyhow!(mismatch))
}

#[cfg(test)]
mod tests {
    use super::{compare_trace_text, trace_header, trace_text_for_script};

    #[test]
    fn trace_header_preserves_current_schema_contract() {
        assert!(trace_header().starts_with("frame\tinput_bits\tinput_in0\tinput_in1\tinput_in2"));
        assert!(trace_header().ends_with("sound_commands\tevents"));
    }

    #[test]
    fn trace_text_for_script_preserves_current_trace_generation() {
        let text = trace_text_for_script("none;none").expect("trace text");

        assert!(text.starts_with(trace_header()));
        assert_eq!(text.lines().count(), 3);
        assert!(text.contains("\n2\t0x0000\t"));
    }

    #[test]
    fn compare_trace_text_returns_matched_frame_count() {
        let text = trace_text_for_script("none;none").expect("trace text");

        assert_eq!(compare_trace_text(&text, &text).expect("matching trace"), 2);
    }

    #[test]
    fn compare_trace_text_preserves_mismatch_error_contract() {
        let expected = "frame\tinput_bits\n1\t0x0000\n";
        let actual = "frame\tinput_bits\n1\t0xFFFF\n";
        let error = compare_trace_text(expected, actual).expect_err("mismatched trace should fail");

        assert!(error.to_string().contains("trace mismatch at line 2"));
    }
}
