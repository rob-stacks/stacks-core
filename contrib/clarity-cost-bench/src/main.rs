mod analysis;
mod callgrind;
mod cost_model;
mod counting_store;
mod coverage;
mod runner;
mod snippet;
mod suites;

use std::collections::HashMap;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "clarity-cost-bench",
    about = "Measure Clarity 6 function costs via Callgrind"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all (or selected) benchmarks under Callgrind and write results to CSV.
    Bench {
        #[arg(short, long, default_value = "results.csv")]
        output: String,
        /// Comma-separated function names to benchmark (default: all benchmarkable).
        #[arg(short, long)]
        functions: Option<String>,
        /// Number of snippet repetitions per Callgrind invocation.
        #[arg(short, long, default_value = "100")]
        iters: u32,
        /// Callgrind invocations per data point; median is reported.
        #[arg(short, long, default_value = "1")]
        samples: u32,
    },
    /// Internal: run one (function, variant) under Callgrind — called by `bench`.
    Run {
        #[arg(long)]
        function: String,
        #[arg(long)]
        variant: String,
        #[arg(long)]
        size: u64,
        #[arg(long, default_value = "100")]
        iters: u32,
    },
    /// Read a results CSV and print curve-fit + model-comparison analysis.
    Analyze {
        #[arg(default_value = "results.csv")]
        input: String,
    },
    /// List all available benchmarks (including those that require state).
    List {
        /// Show only benchmarkable (not RequiresState) entries.
        #[arg(short, long)]
        ready_only: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::List { ready_only } => cmd_list(ready_only),
        Commands::Run {
            function,
            variant,
            size,
            iters,
        } => cmd_run(&function, &variant, size, iters),
        Commands::Bench {
            output,
            functions,
            iters,
            samples,
        } => cmd_bench(&output, functions.as_deref(), iters, samples),
        Commands::Analyze { input } => cmd_analyze(&input),
    }
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

fn cmd_list(ready_only: bool) {
    println!(
        "{:<30}  {:<7}  {:<12}  {}",
        "FUNCTION", "SINCE", "STATUS", "DETAIL"
    );
    println!("{}", "-".repeat(80));
    for entry in coverage::ALL_FUNCTIONS {
        match &entry.status {
            coverage::Status::Benchmarked => {
                let suite = suites::all_suites().find(|s| s.function == entry.name);
                let variants: String = suite
                    .map(|s| {
                        s.cases
                            .iter()
                            .map(|c| c.variant)
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "MISSING SUITE".to_string());
                println!(
                    "{:<30}  {:<7}  {:<12}  variants: {}",
                    entry.name, entry.since, "benchmarked", variants
                );
            }
            coverage::Status::RequiresState(reason) => {
                if !ready_only {
                    println!(
                        "{:<30}  {:<7}  {:<12}  {}",
                        entry.name, entry.since, "state-req'd", reason
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// run  (inner mode)
// ---------------------------------------------------------------------------

fn cmd_run(function: &str, variant: &str, size: u64, iters: u32) {
    let (_, case) = suites::find(function, variant).unwrap_or_else(|| {
        eprintln!("unknown function/variant: {function} / {variant}");
        std::process::exit(1);
    });

    if let Err(e) = runner::run(&case.execution, function, size, iters) {
        eprintln!("error running '{function}/{variant}' at size {size}: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// bench  (orchestrator)
// ---------------------------------------------------------------------------

fn cmd_bench(output: &str, filter: Option<&str>, iters: u32, samples: u32) {
    let exe = std::env::current_exe()
        .expect("cannot determine own executable path")
        .to_string_lossy()
        .into_owned();

    let filter_set: Option<Vec<&str>> = filter.map(|s| s.split(',').map(str::trim).collect());

    let selected: Vec<&suites::Suite> = suites::all_suites()
        .filter(|s| {
            filter_set
                .as_ref()
                .map_or(true, |ids| ids.contains(&s.function))
        })
        .collect();

    if selected.is_empty() {
        eprintln!("no suites selected");
        std::process::exit(1);
    }

    let total_runs: usize = selected
        .iter()
        .map(|s| s.cases.iter().map(|c| c.sizes.len()).sum::<usize>())
        .sum();
    let mut done = 0usize;

    let mut wtr = csv::Writer::from_path(output).unwrap_or_else(|e| {
        eprintln!("cannot open {output}: {e}");
        std::process::exit(1)
    });
    wtr.write_record([
        "function",
        "variant",
        "n_unit",
        "size",
        "instrs_per_call",
        "store_bytes_read_per_call",
        "store_bytes_written_per_call",
        "iters",
    ])
    .unwrap();

    for suite in &selected {
        for case in suite.cases {
            for &size in case.sizes {
                done += 1;
                eprint!(
                    "[{done}/{total_runs}] {}/{} size={size} ... ",
                    suite.function, case.variant
                );

                let cg = callgrind::measure_median(
                    &exe,
                    suite.function,
                    case.variant,
                    size,
                    iters,
                    samples,
                );
                let store = runner::run(&case.execution, suite.function, size, iters);

                match (cg, store) {
                    (Ok(m), Ok(s)) => {
                        let n = iters as u64;
                        eprintln!(
                            "{} instrs  {} store-read  {} store-written",
                            m.instrs / n,
                            s.bytes_read / n,
                            s.bytes_written / n,
                        );
                        wtr.write_record(&[
                            suite.function,
                            case.variant,
                            case.n_unit,
                            &size.to_string(),
                            &(m.instrs / n).to_string(),
                            &(s.bytes_read / n).to_string(),
                            &(s.bytes_written / n).to_string(),
                            &iters.to_string(),
                        ])
                        .unwrap();
                        wtr.flush().unwrap();
                    }
                    (Err(e), _) | (_, Err(e)) => eprintln!("FAILED: {e}"),
                }
            }
        }
    }
    println!("\nResults written to {output}");
}

// ---------------------------------------------------------------------------
// analyze
// ---------------------------------------------------------------------------

fn cmd_analyze(input: &str) {
    let mut rdr = csv::Reader::from_path(input).unwrap_or_else(|e| {
        eprintln!("cannot open {input}: {e}");
        std::process::exit(1)
    });

    // Group by (function, variant).
    let mut groups: HashMap<(String, String), analysis::Group> = HashMap::new();

    for result in rdr.records() {
        let rec = result.unwrap();
        let function = rec[0].to_string();
        let variant = rec[1].to_string();
        let n_unit = rec[2].to_string();
        let size: u64 = rec[3].parse().unwrap();
        let instrs: u64 = rec[4].parse().unwrap();
        let bytes_read: u64 = rec.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
        let bytes_written: u64 = rec.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);

        let entry = groups
            .entry((function.clone(), variant.clone()))
            .or_insert_with(|| analysis::Group {
                function: function.clone(),
                variant: variant.clone(),
                n_unit,
                sizes: Vec::new(),
                instrs: Vec::new(),
                bytes_read: Vec::new(),
                bytes_written: Vec::new(),
            });
        entry.sizes.push(size);
        entry.instrs.push(instrs);
        entry.bytes_read.push(bytes_read);
        entry.bytes_written.push(bytes_written);
    }

    // Calibrate from the + / uint group (must happen before printing).
    let instrs_per_cost_unit = analysis::calibrate_from_plus(&groups);
    if let Some(ipu) = instrs_per_cost_unit {
        println!("Calibration: 1 cost unit ≈ {ipu:.0} instructions  (derived from +/uint)");
    } else {
        println!("Note: no +/uint data found — cost comparison columns will be omitted.");
    }

    let mut keys: Vec<_> = groups.keys().cloned().collect();
    keys.sort();
    for key in keys {
        analysis::print_report(&groups[&key], instrs_per_cost_unit);
    }
}
