use crate::cost_model;

pub struct Group {
    pub function: String,
    pub variant: String,
    pub n_unit: String,
    pub sizes: Vec<u64>,
    pub instrs: Vec<u64>,
    pub mem_reads: Vec<u64>,
    pub mem_writes: Vec<u64>,
}

// ── calibration ──────────────────────────────────────────────────────────────

/// Compute `instructions_per_cost_unit` from the `+` / uint benchmark.
/// Uses weighted average: Σ instrs / Σ model_cost across all data points.
pub fn calibrate_from_plus(
    groups: &std::collections::HashMap<(String, String), Group>,
) -> Option<f64> {
    let spec = cost_model::for_function("+")?;
    let group = groups
        .get(&("+".into(), "uint".into()))
        .or_else(|| groups.get(&("+".into(), "int".into())))?;

    let (sum_instrs, sum_cost): (f64, f64) = group
        .sizes
        .iter()
        .zip(group.instrs.iter())
        .map(|(&n, &i)| (i as f64, (spec.eval)(n) as f64))
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

// ── report ────────────────────────────────────────────────────────────────────

fn fmt_delta(d: i64) -> String {
    if d == 0 {
        "  0".into()
    } else if d > 0 {
        format!(" +{d}")
    } else {
        format!(" {d}")
    }
}

fn metric_block(label: &str, sizes: &[u64], counts: &[u64], n_unit: &str) {
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
    println!("  └─");
}

pub fn print_report(g: &Group, instrs_per_cost_unit: Option<f64>) {
    let spec = cost_model::for_function(&g.function);

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

    if let Some(s) = spec {
        println!(" Current  : runtime = {}", s.formula);
    } else {
        println!(" Current  : (SpecialFunction — no exported runtime cost)");
    }
    if let Some(ipu) = instrs_per_cost_unit {
        println!(" Baseline : 1 cost unit ≈ {ipu:.0} instructions  (calibrated from +/uint)");
    }

    // ── cost comparison table ───────────────────────────────────────────────
    if spec.is_some() && instrs_per_cost_unit.is_some() {
        let ipu = instrs_per_cost_unit.unwrap();
        let spec = spec.unwrap();

        println!("──────────────────────────────────────────────────────────────────────");
        println!(
            "  {:>10}  {:>14}  {:>12}  {:>11}  {:>8}",
            g.n_unit, "instrs/call", "current cost", "suggested", "Δ"
        );

        for (&n, &instrs) in g.sizes.iter().zip(g.instrs.iter()) {
            let current = (spec.eval)(n);
            let suggested = (instrs as f64 / ipu).round() as u64;
            let delta = suggested as i64 - current as i64;
            println!(
                "  {:>10}  {:>14}  {:>12}  {:>11}  {:>8}",
                n,
                instrs,
                current,
                suggested,
                fmt_delta(delta)
            );
        }
    }

    println!("──────────────────────────────────────────────────────────────────────");
    metric_block("instrs/call", &g.sizes, &g.instrs, &g.n_unit);
    metric_block("mem-reads/call", &g.sizes, &g.mem_reads, &g.n_unit);
    metric_block("mem-writes/call", &g.sizes, &g.mem_writes, &g.n_unit);
    println!("══════════════════════════════════════════════════════════════════════");
}
