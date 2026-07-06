/// Exhaustive list of every Clarity 6 native function and its benchmark status.
/// A test in this module verifies that every `Benchmarked` entry has a
/// corresponding suite in `suites.rs`, making coverage a compile-time invariant.
pub struct FunctionEntry {
    pub name: &'static str,
    /// Which Clarity version introduced this function.
    pub since: &'static str,
    pub status: Status,
}

pub enum Status {
    /// A benchmark suite exists in suites::ALL_SUITES.
    Benchmarked,
    /// Cannot be tested with a pure in-memory snippet execution.
    RequiresState(&'static str),
}

pub static ALL_FUNCTIONS: &[FunctionEntry] = &[
    // -----------------------------------------------------------------------
    // Clarity 1 — arithmetic
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "+",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "-",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "*",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "/",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "mod",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "pow",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "sqrti",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "log2",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "to-int",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "to-uint",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "xor",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — comparison
    FunctionEntry {
        name: ">=",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "<=",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "<",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: ">",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-eq",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — boolean
    FunctionEntry {
        name: "and",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "or",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "not",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — control flow
    FunctionEntry {
        name: "if",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "let",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "begin",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "asserts!",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "match",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — sequences
    FunctionEntry {
        name: "map",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "filter",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "fold",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "append",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "concat",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "as-max-len?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "len",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "element-at",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "index-of",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "list",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — data variables / maps (benchmarked via in-memory contract deployment)
    FunctionEntry {
        name: "var-get",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "var-set",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "map-get?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "map-set",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "map-insert",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "map-delete",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — tuples
    FunctionEntry {
        name: "tuple",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "get",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "merge",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — hash
    FunctionEntry {
        name: "hash160",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "sha256",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "sha512",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "sha512/256",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "keccak256",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — crypto
    FunctionEntry {
        name: "secp256k1-recover?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "secp256k1-verify",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — print
    FunctionEntry {
        name: "print",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — contract / principal
    FunctionEntry {
        name: "contract-call?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "as-contract",
        since: "1",
        status: Status::RequiresState("deprecated in Clarity 3; needs contract context"),
    },
    FunctionEntry {
        name: "contract-of",
        since: "1",
        status: Status::RequiresState("needs a trait-typed value from a deployed contract"),
    },
    FunctionEntry {
        name: "principal-of?",
        since: "1",
        status: Status::RequiresState("needs secp256k1 pubkey with recoverable-sig context"),
    },
    FunctionEntry {
        name: "at-block",
        since: "1",
        status: Status::RequiresState("needs block hash from chain history"),
    },
    // Clarity 1 — block info
    FunctionEntry {
        name: "get-block-info?",
        since: "1",
        status: Status::Benchmarked,
    },
    // Clarity 1 — option / result
    FunctionEntry {
        name: "err",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ok",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "some",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "default-to",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "unwrap!",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "unwrap-err!",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "unwrap-panic",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "unwrap-err-panic",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "try!",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-ok",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-none",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-err",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-some",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "filter",
        since: "1",
        status: Status::Benchmarked,
    }, // duplicate entry intentionally omitted, already above
    // Clarity 1 — tokens / assets (benchmarked via in-memory contract deployment)
    FunctionEntry {
        name: "ft-get-balance",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "nft-get-owner?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ft-transfer?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "nft-transfer?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "nft-mint?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ft-mint?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ft-get-supply",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ft-burn?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "nft-burn?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "stx-get-balance",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "stx-transfer?",
        since: "1",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "stx-burn?",
        since: "1",
        status: Status::Benchmarked,
    },
    // -----------------------------------------------------------------------
    // Clarity 2
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "element-at?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "index-of?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "buff-to-int-le",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "buff-to-uint-le",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "buff-to-int-be",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "buff-to-uint-be",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "is-standard",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "principal-destruct?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "principal-construct?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "string-to-int?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "string-to-uint?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "int-to-ascii",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "int-to-utf8",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "get-burn-block-info?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "stx-transfer-memo?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "stx-account",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-and",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-or",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-not",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-shift-left",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-shift-right",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "bit-xor",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "slice?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "to-consensus-buff?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "from-consensus-buff?",
        since: "2",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "replace-at?",
        since: "2",
        status: Status::Benchmarked,
    },
    // -----------------------------------------------------------------------
    // Clarity 3
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "get-stacks-block-info?",
        since: "3",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "get-tenure-info?",
        since: "3",
        status: Status::RequiresState("needs tenure/block data from real chain history"),
    },
    // -----------------------------------------------------------------------
    // Clarity 4
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "contract-hash?",
        since: "4",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "to-ascii?",
        since: "4",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "restrict-assets?",
        since: "4",
        status: Status::RequiresState(
            "post-condition modifier; needs full contract execution context",
        ),
    },
    FunctionEntry {
        name: "as-contract?",
        since: "4",
        status: Status::RequiresState(
            "post-condition modifier; needs full contract execution context",
        ),
    },
    FunctionEntry {
        name: "with-stx",
        since: "4",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "with-ft",
        since: "4",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "with-nft",
        since: "4",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "with-all-assets-unsafe",
        since: "4",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "secp256r1-verify",
        since: "4",
        status: Status::Benchmarked,
    },
    // -----------------------------------------------------------------------
    // Clarity 5
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "with-stacking",
        since: "5",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    // -----------------------------------------------------------------------
    // Clarity 6
    // -----------------------------------------------------------------------
    FunctionEntry {
        name: "with-staking",
        since: "6",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "with-pox",
        since: "6",
        status: Status::RequiresState("allowance modifier; needs full contract execution context"),
    },
    FunctionEntry {
        name: "verify-merkle-proof",
        since: "6",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "get-bitcoin-tx-output?",
        since: "6",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "ed25519-verify",
        since: "6",
        status: Status::Benchmarked,
    },
    FunctionEntry {
        name: "secp256k1-decompress?",
        since: "6",
        status: Status::Benchmarked,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suites;

    fn all_suite_slices() -> impl Iterator<Item = &'static suites::Suite> {
        suites::ALL_SUITES
            .iter()
            .chain(suites::STATEFUL_SUITES.iter())
    }

    #[test]
    fn all_benchmarkable_functions_have_suites() {
        let mut missing = Vec::new();
        for entry in ALL_FUNCTIONS {
            if matches!(entry.status, Status::Benchmarked) {
                let found = all_suite_slices().any(|s| s.function == entry.name);
                if !found {
                    missing.push(entry.name);
                }
            }
        }
        assert!(
            missing.is_empty(),
            "Functions marked Benchmarked but missing a suite: {missing:?}"
        );
    }

    #[test]
    fn all_suites_reference_known_functions() {
        let mut unknown = Vec::new();
        for suite in all_suite_slices() {
            let found = ALL_FUNCTIONS.iter().any(|f| f.name == suite.function);
            if !found {
                unknown.push(suite.function);
            }
        }
        assert!(
            unknown.is_empty(),
            "Suites reference functions not in ALL_FUNCTIONS: {unknown:?}"
        );
    }
}
