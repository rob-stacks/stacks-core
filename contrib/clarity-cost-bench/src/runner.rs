use clarity::vm::contexts::OwnedEnvironment;
use clarity::vm::database::MemoryBackingStore;
use clarity::vm::execute_v6;
use clarity::vm::types::{PrincipalData, QualifiedContractIdentifier, StandardPrincipalData};
use clarity_types::ContractName;
use clarity_types::ClarityVersion;
use stacks_common::consts::CHAIN_ID_TESTNET;
use stacks_common::types::StacksEpochId;

use crate::snippet::Snippet;
use crate::suites::Execution;

/// The principal under which benchmark contracts are deployed.
fn bench_principal() -> StandardPrincipalData {
    StandardPrincipalData::transient()
}

fn bench_contract_id() -> QualifiedContractIdentifier {
    QualifiedContractIdentifier::new(
        bench_principal(),
        ContractName::try_from("bench").expect("valid contract name"),
    )
}

/// Execute a case `iters` times, dispatching on its execution kind.
pub fn run(execution: &Execution, function: &str, size: u64, iters: u32) -> Result<(), String> {
    match execution {
        Execution::Snippet(s) => {
            let snippet = s.generate(function, size);
            run_snippet(&snippet, iters)
        }
        Execution::Contract { source, fn_name, initial_ustx } => {
            let src = source(size);
            run_contract(&src, fn_name, *initial_ustx, iters)
        }
    }
}

/// Evaluate a standalone Clarity 6 snippet `iters` times.
fn run_snippet(snippet: &str, iters: u32) -> Result<(), String> {
    for _ in 0..iters {
        execute_v6(snippet).map_err(|e| format!("{e:?}"))?;
    }
    Ok(())
}

/// Deploy a Clarity 6 contract in an in-memory environment, optionally fund
/// `tx-sender` via `stx_faucet`, then call `fn_name` `iters` times.
fn run_contract(
    source: &str,
    fn_name: &str,
    initial_ustx: u128,
    iters: u32,
) -> Result<(), String> {
    let mut marf = MemoryBackingStore::new();
    let db = marf.as_clarity_db();
    let mut env = OwnedEnvironment::new_free(false, CHAIN_ID_TESTNET, db, StacksEpochId::Epoch40);

    let sender: PrincipalData = bench_principal().into();
    let contract_id = bench_contract_id();

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

    for _ in 0..iters {
        env.execute_transaction(sender.clone(), None, contract_id.clone(), fn_name, &[])
            .map_err(|e| format!("execute_transaction error: {e:?}"))?;
    }

    Ok(())
}
