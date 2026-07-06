use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Metrics extracted from a single callgrind run.
#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    /// Instruction references (Ir) — total instructions executed.
    pub instrs: u64,
}

/// Run callgrind `samples` times and return the median `Ir` count.
///
/// `std::collections::HashMap` uses `RandomState` (seeded by `getrandom()`),
/// so each process produces slightly different probe-chain lengths.  Taking
/// the median of an odd number of samples cancels one-sided outliers while
/// keeping bench time proportional to `samples`.
pub fn measure_median(
    exe: &str,
    function: &str,
    variant: &str,
    size: u64,
    iters: u32,
    samples: u32,
) -> Result<Metrics, String> {
    if samples <= 1 {
        return measure(exe, function, variant, size, iters);
    }
    let mut counts: Vec<u64> = (0..samples)
        .map(|_| measure(exe, function, variant, size, iters).map(|m| m.instrs))
        .collect::<Result<Vec<_>, _>>()?;
    counts.sort_unstable();
    Ok(Metrics {
        instrs: counts[counts.len() / 2],
    })
}

/// Run the tool under callgrind for one (function, variant, size, iters) tuple.
pub fn measure(
    exe: &str,
    function: &str,
    variant: &str,
    size: u64,
    iters: u32,
) -> Result<Metrics, String> {
    let sanitise = |s: &str| -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    };
    let safe_name = format!("{}_{}_{}", sanitise(function), sanitise(variant), size);
    let out_file = PathBuf::from(format!("/tmp/cg_{safe_name}.out"));

    let out_file_arg = format!("--callgrind-out-file={}", out_file.display());

    let status = Command::new("valgrind")
        .args([
            "--tool=callgrind",
            &out_file_arg,
            // Cache simulation is intentionally disabled: valgrind ignores --LL
            // when it detects hardware L3, making Dr/Dw machine-dependent.
            // Ir (instruction count) is fully deterministic and sufficient for
            // cost-function fitting.
            "--quiet",
            // Instrumentation starts off; runner.rs calls CALLGRIND_START/STOP
            // around the measured iterations so that one-time lazy-init work
            // (regex DFA construction, HashMap RandomState seeding, etc.) that
            // happens in the warmup call is excluded from the counts.
            "--instr-atstart=no",
            "--",
            exe,
            "run",
            "--function",
            function,
            "--variant",
            variant,
            "--size",
            &size.to_string(),
            "--iters",
            &iters.to_string(),
        ])
        .status()
        .map_err(|e| format!("failed to spawn valgrind: {e}"))?;

    if !status.success() {
        return Err(format!("valgrind exited with {status}"));
    }

    let raw = fs::read_to_string(&out_file)
        .map_err(|e| format!("cannot read {}: {e}", out_file.display()))?;
    let _ = fs::remove_file(&out_file);

    parse_metrics(&raw)
}

/// Parse the callgrind output file.
///
/// Looks for the `events:` line to find the `Ir` column, then reads the
/// `totals:` (or `summary:`) line to extract the instruction count.
pub fn parse_metrics(content: &str) -> Result<Metrics, String> {
    let ir_col: usize = content
        .lines()
        .find(|l| l.trim_start().starts_with("events:"))
        .and_then(|l| {
            l.trim_start()
                .strip_prefix("events:")
                .unwrap_or("")
                .split_whitespace()
                .position(|name| name == "Ir")
        })
        .ok_or_else(|| "no Ir column in callgrind events line".to_string())?;

    let totals_line = content
        .lines()
        .rev()
        .find(|l| {
            let t = l.trim();
            t.starts_with("totals:") || t.starts_with("summary:")
        })
        .ok_or_else(|| "no totals/summary line in callgrind output".to_string())?;

    let instrs = totals_line
        .split_whitespace()
        .skip(1) // skip "totals:" / "summary:"
        .nth(ir_col)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Ok(Metrics { instrs })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ir_only() {
        let sample = "events: Ir\ntotals: 123456\n";
        assert_eq!(parse_metrics(sample).unwrap().instrs, 123_456);
    }

    #[test]
    fn parse_ir_among_other_events() {
        let sample = "events: Ir Dr Dw\ntotals: 987654321 111111111 22222222\n";
        assert_eq!(parse_metrics(sample).unwrap().instrs, 987_654_321);
    }

    #[test]
    fn parse_summary_fallback() {
        let sample = "events: Ir Dr Dw\nsummary: 111 222 333\n";
        assert_eq!(parse_metrics(sample).unwrap().instrs, 111);
    }
}
