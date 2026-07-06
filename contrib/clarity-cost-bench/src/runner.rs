use clarity::vm::contexts::OwnedEnvironment;
use clarity::vm::database::MemoryBackingStore;
use clarity::vm::execute_v6;
use clarity::vm::types::{PrincipalData, QualifiedContractIdentifier, StandardPrincipalData};
use clarity_types::{ClarityVersion, ContractName};
use stacks_common::consts::CHAIN_ID_TESTNET;
use stacks_common::types::StacksEpochId;

use crate::counting_store::{CountingStore, StoreByteCounts};
use crate::suites::Execution;

fn bench_principal() -> StandardPrincipalData {
    StandardPrincipalData::transient()
}

fn bench_contract_id() -> QualifiedContractIdentifier {
    QualifiedContractIdentifier::new(
        bench_principal(),
        ContractName::try_from("bench").expect("valid contract name"),
    )
}

/// Execute a case `iters` times and return the bytes transferred to/from the
/// backing store during the iteration phase (excludes contract setup).
pub fn run(
    execution: &Execution,
    function: &str,
    size: u64,
    iters: u32,
) -> Result<StoreByteCounts, String> {
    match execution {
        Execution::Snippet(s) => {
            let snippet = s.generate(function, size);
            run_snippet(&snippet, iters)
        }
        Execution::Contract {
            source,
            fn_name,
            initial_ustx,
        } => {
            let src = source(size);
            run_contract(&src, fn_name, *initial_ustx, iters)
        }
    }
}

/// Snippets have no backing store — byte counts are always zero.
fn run_snippet(snippet: &str, iters: u32) -> Result<StoreByteCounts, String> {
    // Warmup: execute once without counting so that all lazy-init work — regex
    // DFA compilation, HashMap RandomState seeding — completes before the
    // callgrind instrumentation window opens.
    let _ = execute_v6(snippet);

    crate::valgrind::start_instrumentation();
    for _ in 0..iters {
        execute_v6(snippet).map_err(|e| format!("{e:?}"))?;
    }
    crate::valgrind::stop_instrumentation();

    Ok(StoreByteCounts::default())
}

fn run_contract(
    source: &str,
    fn_name: &str,
    initial_ustx: u128,
    iters: u32,
) -> Result<StoreByteCounts, String> {
    let sender: PrincipalData = bench_principal().into();
    let contract_id = bench_contract_id();

    let mut marf = MemoryBackingStore::new();

    // Setup phase: deploy contract. Not counted — we only want per-call costs.
    {
        let db = marf.as_clarity_db();
        let mut env =
            OwnedEnvironment::new_free(false, CHAIN_ID_TESTNET, db, StacksEpochId::Epoch40);
        if initial_ustx > 0 {
            env.stx_faucet(&sender, initial_ustx);
        }
        env.initialize_versioned_contract(
            contract_id.clone(),
            ClarityVersion::Clarity6,
            source,
            None,
        )
        .map_err(|e| format!("contract deploy error: {e:?}"))?;
    }

    // Warmup: one un-instrumented call so lazy statics (regex DFAs, etc.) are
    // fully initialized before the callgrind counting window opens.
    {
        let db = marf.as_clarity_db();
        let mut env =
            OwnedEnvironment::new_free(false, CHAIN_ID_TESTNET, db, StacksEpochId::Epoch40);
        let _ = env.execute_transaction(sender.clone(), None, contract_id.clone(), fn_name, &[]);
    }

    // Iteration phase: run `fn_name` iters times under the counting store.
    let mut counting = CountingStore::new(&mut marf);
    {
        crate::valgrind::start_instrumentation();
        let db = counting.as_clarity_db();
        let mut env =
            OwnedEnvironment::new_free(false, CHAIN_ID_TESTNET, db, StacksEpochId::Epoch40);
        for _ in 0..iters {
            env.execute_transaction(sender.clone(), None, contract_id.clone(), fn_name, &[])
                .map_err(|e| format!("execute_transaction error: {e:?}"))?;
        }
        crate::valgrind::stop_instrumentation();
    }

    Ok(counting.counts())
}
