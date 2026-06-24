use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Metrics extracted from a single callgrind run.
#[derive(Debug, Clone, Copy, Default)]
pub struct Metrics {
    /// Instruction references (Ir) — total instructions executed.
    pub instrs: u64,
    /// Data reads (Dr) — memory read accesses.
    pub mem_reads: u64,
    /// Data writes (Dw) — memory write accesses.
    pub mem_writes: u64,
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
            "--simulate-cache=yes", // adds Dr + Dw alongside Ir
            "--quiet",
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
/// Looks for the `events:` line to determine column positions, then reads
/// the `totals:` (or `summary:`) line to extract `Ir`, `Dr`, `Dw` counts.
pub fn parse_metrics(content: &str) -> Result<Metrics, String> {
    // Build a column-index map from the `events:` line.
    let col_map: HashMap<&str, usize> = content
        .lines()
        .find(|l| l.trim_start().starts_with("events:"))
        .map(|l| {
            l.trim_start()
                .strip_prefix("events:")
                .unwrap_or("")
                .split_whitespace()
                .enumerate()
                .map(|(i, name)| (name, i))
                .collect()
        })
        .unwrap_or_default();

    // Find the totals/summary line (last occurrence wins).
    let totals_line = content
        .lines()
        .rev()
        .find(|l| {
            let t = l.trim();
            t.starts_with("totals:") || t.starts_with("summary:")
        })
        .ok_or_else(|| "no totals/summary line in callgrind output".to_string())?;

    let values: Vec<u64> = totals_line
        .split_whitespace()
        .skip(1) // skip "totals:" / "summary:"
        .map(|s| s.parse::<u64>().unwrap_or(0))
        .collect();

    let get = |name: &str| -> u64 {
        col_map
            .get(name)
            .and_then(|&i| values.get(i))
            .copied()
            .unwrap_or(0)
    };

    Ok(Metrics {
        instrs: get("Ir"),
        mem_reads: get("Dr"),
        mem_writes: get("Dw"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_cache_sim() {
        let sample = "\
events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw
fl=???
fn=(below main)
0 1 2 3 4 5 6 7 8 9
totals: 987654321 111111111 22222222 100 200 300 50 100 150
";
        let m = parse_metrics(sample).unwrap();
        assert_eq!(m.instrs, 987_654_321);
        assert_eq!(m.mem_reads, 111_111_111);
        assert_eq!(m.mem_writes, 22_222_222);
    }

    #[test]
    fn parse_ir_only() {
        // Without --simulate-cache, only Ir is present.
        let sample = "events: Ir\ntotals: 123456\n";
        let m = parse_metrics(sample).unwrap();
        assert_eq!(m.instrs, 123_456);
        assert_eq!(m.mem_reads, 0);
        assert_eq!(m.mem_writes, 0);
    }

    #[test]
    fn parse_summary_fallback() {
        let sample = "events: Ir Dr Dw\nsummary: 111 222 333\n";
        let m = parse_metrics(sample).unwrap();
        assert_eq!(m.instrs, 111);
        assert_eq!(m.mem_reads, 222);
        assert_eq!(m.mem_writes, 333);
    }
}
