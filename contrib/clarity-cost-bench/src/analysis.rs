/// AICc-based complexity analysis, applied independently to each metric.

pub struct Group {
    pub function: String,
    pub variant: String,
    pub n_unit: String,
    pub sizes: Vec<u64>,
    pub instrs: Vec<u64>,
    pub mem_reads: Vec<u64>,
    pub mem_writes: Vec<u64>,
}

// ---------------------------------------------------------------------------
// Candidate models
// ---------------------------------------------------------------------------

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
            Complexity::Constant        => write!(f, "O(1)"),
            Complexity::Logarithmic     => write!(f, "O(log n)"),
            Complexity::SquareRoot      => write!(f, "O(√n)"),
            Complexity::Linear          => write!(f, "O(n)"),
            Complexity::NLogN           => write!(f, "O(n·log n)"),
            Complexity::Quadratic       => write!(f, "O(n²)"),
            Complexity::PowerLaw { k }  => write!(f, "O(n^{k:.2})"),
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

// ---------------------------------------------------------------------------
// Curve fitting
// ---------------------------------------------------------------------------

fn linear_regression(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let sx: f64 = x.iter().sum();
    let sy: f64 = y.iter().sum();
    let sxx: f64 = x.iter().map(|v| v * v).sum();
    let sxy: f64 = x.iter().zip(y).map(|(xi, yi)| xi * yi).sum();
    let d = n * sxx - sx * sx;
    if d.abs() < f64::EPSILON { return (0.0, sy / n); }
    let a = (n * sxy - sx * sy) / d;
    let b = (sy - a * sx) / n;
    (a, b)
}

fn aicc(n: usize, k_params: usize, rss: f64) -> f64 {
    let n = n as f64;
    let k = k_params as f64;
    if rss <= 0.0 { return f64::NEG_INFINITY; }
    let correction = if n - k - 1.0 > 0.0 { 2.0 * k * (k + 1.0) / (n - k - 1.0) } else { f64::INFINITY };
    n * (rss / n).ln() + 2.0 * k + correction
}

fn rss_r2(x_feat: &[f64], y: &[f64], a: f64, b: f64) -> (f64, f64) {
    let rss: f64 = x_feat.iter().zip(y).map(|(xi, yi)| (yi - (a * xi + b)).powi(2)).sum();
    let my = y.iter().sum::<f64>() / y.len() as f64;
    let ss: f64 = y.iter().map(|yi| (yi - my).powi(2)).sum();
    (rss, if ss < f64::EPSILON { 1.0 } else { 1.0 - rss / ss })
}

fn fit_one(c: Complexity, x: &[f64], y: &[f64]) -> Option<Fit> {
    if x.len() < 2 { return None; }
    let (x_feat, k): (Vec<f64>, usize) = match c {
        Complexity::Constant    => (vec![1.0; x.len()], 1),
        Complexity::Logarithmic => (x.iter().map(|v| v.max(1.0).ln()).collect(), 2),
        Complexity::SquareRoot  => (x.iter().map(|v| v.sqrt()).collect(), 2),
        Complexity::Linear      => (x.to_vec(), 2),
        Complexity::NLogN       => (x.iter().map(|v| v * v.max(1.0).ln()).collect(), 2),
        Complexity::Quadratic   => (x.iter().map(|v| v * v).collect(), 2),
        Complexity::PowerLaw { k: pk } => (x.iter().map(|v| v.powf(pk)).collect(), 3),
    };
    let (a, b) = if c == Complexity::Constant { (0.0, y.iter().sum::<f64>() / y.len() as f64) }
                 else { linear_regression(&x_feat, y) };
    let (rss, r2) = rss_r2(&x_feat, y, a, b);
    Some(Fit { complexity: c, a, b, aicc: aicc(x.len(), k, rss.max(f64::EPSILON)), r2 })
}

fn estimate_power_k(x: &[f64], y: &[f64]) -> f64 {
    let min_y = y.iter().cloned().fold(f64::INFINITY, f64::min);
    let lx: Vec<f64> = x.iter().map(|v| v.max(1.0).ln()).collect();
    let ly: Vec<f64> = y.iter().map(|v| (v - min_y + 1.0).max(1.0).ln()).collect();
    linear_regression(&lx, &ly).0.max(0.0)
}

pub fn best_fit(sizes: &[u64], counts: &[u64]) -> Vec<Fit> {
    let x: Vec<f64> = sizes.iter().map(|&s| s as f64).collect();
    let y: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    let k = estimate_power_k(&x, &y);
    let mut fits: Vec<Fit> = [
        Complexity::Constant, Complexity::Logarithmic, Complexity::SquareRoot,
        Complexity::Linear, Complexity::NLogN, Complexity::Quadratic,
        Complexity::PowerLaw { k },
    ]
    .iter()
    .filter_map(|&c| fit_one(c, &x, &y))
    .collect();
    fits.sort_by(|a, b| a.aicc.partial_cmp(&b.aicc).unwrap());
    fits
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

fn metric_section(label: &str, sizes: &[u64], counts: &[u64], n_unit: &str) {
    if sizes.len() < 2 { return; }
    let fits = best_fit(sizes, counts);
    let winner = &fits[0];
    println!("  ┌─ {label}");
    println!("  │  {:>10}  {:>14}  {:>12}", n_unit, label, "ratio vs n=1");
    let base = *counts.first().unwrap_or(&1) as f64;
    for (&s, &c) in sizes.iter().zip(counts.iter()) {
        println!("  │  {:>10}  {:>14}  {:>12.2}", s, c, c as f64 / base.max(1.0));
    }
    println!("  │  best fit: {}  (R²={:.4}, AICc={:.1})", winner.complexity, winner.r2, winner.aicc);
    if winner.complexity != Complexity::Constant {
        println!("  │  formula:  {label} ≈ {:.3} × {} + {:.0}", winner.a, winner.complexity, winner.b);
    }
    println!("  │  ranking:  {}", fits.iter().map(|f| format!("{}", f.complexity)).collect::<Vec<_>>().join(" > "));
    println!("  └─");
}

pub fn print_report(g: &Group) {
    println!("\n══════════════════════════════════════════════════════════════");
    println!(" Function : {} / {}", g.function, g.variant);
    println!(" Dimension: {} ({})", g.n_unit, g.sizes.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "));
    println!("──────────────────────────────────────────────────────────────");
    metric_section("instrs/call",      &g.sizes, &g.instrs,     &g.n_unit);
    metric_section("mem-reads/call",   &g.sizes, &g.mem_reads,  &g.n_unit);
    metric_section("mem-writes/call",  &g.sizes, &g.mem_writes, &g.n_unit);
    println!("══════════════════════════════════════════════════════════════");
}
