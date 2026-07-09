pub struct Group {
    pub function: String,
    pub variant: String,
    pub n_unit: String,
    pub sizes: Vec<u64>,
    pub instrs: Vec<u64>,
    pub bytes_read: Vec<u64>,
    pub bytes_written: Vec<u64>,
}

// ── calibration ──────────────────────────────────────────────────────────────

/// Derive `instructions_per_cost_unit` from the `+/uint` benchmark.
///
/// `(+ u0 u1 … u{n-1})` with N arguments charges `cost_add(N) = linear(N, 11, 125) = 11N + 125`
/// in all Clarity epochs.  We use a weighted average: Σ instrs / Σ model_cost.
pub fn calibrate_from_plus(
    groups: &std::collections::HashMap<(String, String), Group>,
) -> Option<f64> {
    let group = groups
        .get(&("+".into(), "uint".into()))
        .or_else(|| groups.get(&("+".into(), "int".into())))?;

    let (sum_instrs, sum_cost): (f64, f64) = group
        .sizes
        .iter()
        .zip(group.instrs.iter())
        .map(|(&n, &i)| (i as f64, (11 * n + 125) as f64))
        .filter(|(_, c)| *c > 0.0)
        .fold((0.0, 0.0), |(si, sc), (i, c)| (si + i, sc + c));

    if sum_cost == 0.0 {
        return None;
    }
    Some(sum_instrs / sum_cost)
}

// ── model selection ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Complexity {
    Constant,
    Logarithmic,
    SquareRoot,
    Linear,
    NLogN,
    Quadratic,
    PowerLaw { k: f64 },
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::Constant => write!(f, "O(1)"),
            Complexity::Logarithmic => write!(f, "O(log n)"),
            Complexity::SquareRoot => write!(f, "O(√n)"),
            Complexity::Linear => write!(f, "O(n)"),
            Complexity::NLogN => write!(f, "O(n·log n)"),
            Complexity::Quadratic => write!(f, "O(n²)"),
            Complexity::PowerLaw { k } => write!(f, "O(n^{k:.2})"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fit {
    pub complexity: Complexity,
    pub a: f64,
    pub b: f64,
    pub aicc: f64,
    pub r2: f64,
}

impl Fit {
    /// Evaluate the fit at a given n, returning the predicted y value.
    pub fn eval(&self, n: f64) -> f64 {
        let x = match self.complexity {
            Complexity::Constant => 1.0,
            Complexity::Logarithmic => n.max(1.0).ln(),
            Complexity::SquareRoot => n.sqrt(),
            Complexity::Linear => n,
            Complexity::NLogN => n * n.max(1.0).ln(),
            Complexity::Quadratic => n * n,
            Complexity::PowerLaw { k } => n.powf(k),
        };
        self.a * x + self.b
    }
}

fn linreg(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxx: f64 = x.iter().map(|v| v * v).sum();
    let sxy: f64 = x.iter().zip(y).map(|(xi, yi)| xi * yi).sum();
    let d = n * sxx - sx * sx;
    if d.abs() < f64::EPSILON {
        return (0.0, sy / n);
    }
    let a = (n * sxy - sx * sy) / d;
    ((a), (sy - a * sx) / n)
}

fn aicc_score(n: usize, k: usize, rss: f64) -> f64 {
    let (n, k) = (n as f64, k as f64);
    let corr = if n - k - 1.0 > 0.0 {
        2.0 * k * (k + 1.0) / (n - k - 1.0)
    } else {
        f64::INFINITY
    };
    n * (rss / n).max(f64::EPSILON).ln() + 2.0 * k + corr
}

fn rss_r2(xf: &[f64], y: &[f64], a: f64, b: f64) -> (f64, f64) {
    let rss: f64 = xf
        .iter()
        .zip(y)
        .map(|(x, yi)| (yi - (a * x + b)).powi(2))
        .sum();
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let ss: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
    (
        rss,
        if ss < f64::EPSILON {
            1.0
        } else {
            1.0 - rss / ss
        },
    )
}

fn fit_one(c: Complexity, x: &[f64], y: &[f64]) -> Option<Fit> {
    if x.len() < 2 {
        return None;
    }
    let (xf, k): (Vec<f64>, usize) = match c {
        Complexity::Constant => (vec![1.0; x.len()], 1),
        Complexity::Logarithmic => (x.iter().map(|v| v.max(1.0).ln()).collect(), 2),
        Complexity::SquareRoot => (x.iter().map(|v| v.sqrt()).collect(), 2),
        Complexity::Linear => (x.to_vec(), 2),
        Complexity::NLogN => (x.iter().map(|v| v * v.max(1.0).ln()).collect(), 2),
        Complexity::Quadratic => (x.iter().map(|v| v * v).collect(), 2),
        Complexity::PowerLaw { k: pk } => (x.iter().map(|v| v.powf(pk)).collect(), 3),
    };
    let (a, b) = if c == Complexity::Constant {
        (0.0, y.iter().sum::<f64>() / y.len() as f64)
    } else {
        linreg(&xf, y)
    };
    let (rss, r2) = rss_r2(&xf, y, a, b);
    Some(Fit {
        complexity: c,
        a,
        b,
        aicc: aicc_score(x.len(), k, rss),
        r2,
    })
}

fn power_k(x: &[f64], y: &[f64]) -> f64 {
    let min_y = y.iter().cloned().fold(f64::INFINITY, f64::min);
    let lx: Vec<f64> = x.iter().map(|v| v.max(1.0).ln()).collect();
    let ly: Vec<f64> = y.iter().map(|v| (v - min_y + 1.0).max(1.0).ln()).collect();
    linreg(&lx, &ly).0.max(0.0)
}

pub fn select_model(sizes: &[u64], counts: &[u64]) -> Vec<Fit> {
    let x: Vec<f64> = sizes.iter().map(|&s| s as f64).collect();
    let y: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    let k = power_k(&x, &y);
    let mut fits: Vec<Fit> = [
        Complexity::Constant,
        Complexity::Logarithmic,
        Complexity::SquareRoot,
        Complexity::Linear,
        Complexity::NLogN,
        Complexity::Quadratic,
        Complexity::PowerLaw { k },
    ]
    .iter()
    .filter_map(|&c| fit_one(c, &x, &y))
    .collect();
    fits.sort_by(|a, b| a.aicc.partial_cmp(&b.aicc).unwrap());
    fits
}

// ── max supported n per dimension ────────────────────────────────────────────

/// Return the largest value `n` can legally take for a given dimension.
/// Used to extrapolate fits beyond the bench sample range.
pub fn max_n_for_unit(n_unit: &str) -> Option<u64> {
    match n_unit {
        // clarity-types/src/types/mod.rs: MAX_VALUE_SIZE = 1024 * 1024
        "buffer_bytes" | "value_bytes" => Some(1_048_576),
        // clarity-types/src/representations.rs: MAX_STRING_LEN = 128
        "string_chars" => Some(128),
        // MAX_VALUE_SIZE / 1 (bool element = 1 byte, smallest possible element)
        "list_length" => Some(1_048_576),
        _ => None,
    }
}

// ── report ────────────────────────────────────────────────────────────────────

fn metric_block(label: &str, sizes: &[u64], counts: &[u64], n_unit: &str, max_n: Option<u64>) {
    if sizes.len() < 2 {
        return;
    }
    let fits = select_model(sizes, counts);
    let w = &fits[0];
    println!("  ┌─ {label}");
    println!("  │  {:>10}  {:>14}  {:>12}", n_unit, label, "ratio vs n=1");
    let base = *counts.first().unwrap_or(&1) as f64;
    for (&s, &c) in sizes.iter().zip(counts) {
        println!(
            "  │  {:>10}  {:>14}  {:>12.2}",
            s,
            c,
            c as f64 / base.max(1.0)
        );
    }
    if w.complexity != Complexity::Constant {
        println!(
            "  │  fit: {}  R²={:.4}  ≈ {:.3}×{} + {:.0}",
            w.complexity, w.r2, w.a, w.complexity, w.b
        );
    } else {
        println!("  │  fit: {}  R²={:.4}  ≈ {:.1}", w.complexity, w.r2, w.b);
    }
    if let Some(n_max) = max_n {
        if n_max > *sizes.last().unwrap_or(&0) {
            let predicted = w.eval(n_max as f64).round() as u64;
            let ratio = predicted as f64 / base.max(1.0);
            println!(
                "  │  fit at n={n_max} (max): {predicted}  (ratio vs n=1: {ratio:.1}×)"
            );
        }
    }
    println!("  └─");
}

pub fn print_report(g: &Group, instrs_per_cost_unit: Option<f64>) {
    let max_n = max_n_for_unit(&g.n_unit);

    println!("\n══════════════════════════════════════════════════════════════════════");
    println!(" Function : {} / {}", g.function, g.variant);
    println!(
        " Dimension: {} ({})",
        g.n_unit,
        g.sizes
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(ipu) = instrs_per_cost_unit {
        println!(" Baseline : 1 cost unit ≈ {ipu:.0} instructions  (calibrated from +/uint)");
    }
    if let Some(n_max) = max_n {
        println!(" Max n    : {n_max} (largest value {}'s dimension can take)", g.n_unit);
    }

    // ── per-point table ─────────────────────────────────────────────────────
    if let Some(ipu) = instrs_per_cost_unit {
        println!("──────────────────────────────────────────────────────────────────────");
        println!(
            "  {:>10}  {:>14}  {:>11}",
            g.n_unit, "instrs/call", "suggested"
        );

        for (&n, &instrs) in g.sizes.iter().zip(g.instrs.iter()) {
            let suggested = (instrs as f64 / ipu).round() as u64;
            println!("  {:>10}  {:>14}  {:>11}", n, instrs, suggested);
        }

        // extrapolation row at max n (when max is beyond the measured range)
        if let Some(n_max) = max_n {
            if n_max > *g.sizes.last().unwrap_or(&0) {
                let fits = select_model(&g.sizes, &g.instrs);
                let best_fit = &fits[0];
                let predicted_instrs = best_fit.eval(n_max as f64).max(0.0);
                let suggested_at_max = (predicted_instrs / ipu).round() as u64;
                println!(
                    "──── extrapolation ({}) ──────────────────────────────────────────",
                    best_fit.complexity
                );
                println!(
                    "  {:>10}  {:>14}  {:>11}",
                    n_max,
                    format!("~{:.0}", predicted_instrs),
                    suggested_at_max,
                );
            }
        }
    }

    println!("──────────────────────────────────────────────────────────────────────");
    metric_block("instrs/call", &g.sizes, &g.instrs, &g.n_unit, max_n);
    metric_block("store-bytes-read/call", &g.sizes, &g.bytes_read, &g.n_unit, max_n);
    metric_block(
        "store-bytes-written/call",
        &g.sizes,
        &g.bytes_written,
        &g.n_unit,
        max_n,
    );
    println!("══════════════════════════════════════════════════════════════════════");
}
