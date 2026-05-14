//! Runtime-owned fidelity trace command facade.

use anyhow::{anyhow, bail};

pub(crate) fn run_trace(frame_count: usize) -> anyhow::Result<()> {
    print!("{}", trace_text(frame_count)?);
    Ok(())
}

fn trace_text(frame_count: usize) -> anyhow::Result<String> {
    if frame_count == 0 {
        bail!("--fidelity-trace frame count must be greater than zero");
    }

    let input_program = format!("none*{frame_count}");
    let input_script = crate::legacy_fidelity::expanded_trace_input_text(&input_program)
        .map_err(|error| anyhow!(error))?;
    let inputs = crate::legacy_fidelity::parse_trace_input_script(&input_script)
        .map_err(|error| anyhow!(error))?;
    crate::legacy_fidelity::trace_text_for_inputs(&inputs).map_err(|error| anyhow!(error))
}

#[cfg(test)]
mod tests {
    use super::trace_text;

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
}
