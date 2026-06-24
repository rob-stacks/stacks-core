/// Current Clarity 6 runtime cost formulas (from costs_5.rs / costs_4.rs).
///
/// `eval(n)` takes the *same* `n` as the suite's `n_unit` (raw bytes,
/// element count, arg count, etc.) and returns the current model's
/// `ExecutionCost::runtime` value.  None is returned for SpecialFunctions
/// whose runtime cost is not separately exported.
pub struct CostSpec {
    /// Human-readable formula.
    pub formula: &'static str,
    /// Compute the current model's runtime cost for a given n.
    pub eval: fn(u64) -> u64,
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn linear(n: u64, a: u64, b: u64) -> u64 {
    a.saturating_mul(n).saturating_add(b)
}

fn logn(n: u64, a: u64, b: u64) -> u64 {
    if n <= 1 {
        return b;
    }
    a.saturating_mul((n as f64).log2().ceil() as u64)
        .saturating_add(b)
}

// ── per-function specs ────────────────────────────────────────────────────────

// arithmetic (n = arg count)
static COST_ARITH: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_CONST1: CostSpec = CostSpec {
    formula: "constant(1)",
    eval: |_| 1,
};

// hash (n = buffer bytes; model divides by 1 KiB with >>10)
static COST_HASH160: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 1)",
    eval: |n| linear(n >> 10, 125, 1),
};
static COST_SHA256: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 1)",
    eval: |n| linear(n >> 10, 125, 1),
};
static COST_SHA512: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 1)",
    eval: |n| linear(n >> 10, 125, 1),
};
static COST_SHA512T: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 1)",
    eval: |n| linear(n >> 10, 125, 1),
};
static COST_KECCAK: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 1)",
    eval: |n| linear(n >> 10, 125, 1),
};

// Clarity 6 crypto
static COST_MERKLE: CostSpec = CostSpec {
    formula: "linear(n, 125, 502)",
    eval: |n| linear(n, 125, 502),
};
static COST_BTCOUT: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 291)",
    eval: |n| linear(n >> 10, 125, 291),
};
static COST_ED25519: CostSpec = CostSpec {
    formula: "linear(n>>10, 125, 7880)",
    eval: |n| linear(n >> 10, 125, 7880),
};
static COST_SECP_DC: CostSpec = CostSpec {
    formula: "constant(1035)",
    eval: |_| 1035,
};

// sequences (n = element/byte count)
static COST_LEN: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_CONCAT: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_APPEND: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_INDEX_OF: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_SLICE: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_ELEM_AT: CostSpec = CostSpec {
    formula: "constant(1)",
    eval: |_| 1,
};

// tuple (n = field count)
static COST_TUPLE_CONS: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_TUPLE_GET: CostSpec = CostSpec {
    formula: "logn(n, 1, 1)",
    eval: |n| logn(n, 1, 1),
};
static COST_TUPLE_MERGE: CostSpec = CostSpec {
    formula: "logn(n, 1, 1)",
    eval: |n| logn(n, 1, 1),
};

// conversions (constant)
static COST_INT_CAST: CostSpec = CostSpec {
    formula: "constant(1)",
    eval: |_| 1,
};
static COST_PRINT: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};
static COST_CONS_BUF: CostSpec = CostSpec {
    formula: "linear(n, 1, 1)",
    eval: |n| linear(n, 1, 1),
};

// ── lookup ────────────────────────────────────────────────────────────────────

/// Return the current cost spec for a Clarity 6 native function, or `None`
/// for SpecialFunctions without a separately exported runtime cost.
pub fn for_function(function: &str) -> Option<&'static CostSpec> {
    match function {
        // arithmetic / bitwise variadic
        "+" | "-" | "*" | "/" | "begin" | "bit-and" | "bit-or" | "bit-xor" => Some(&COST_ARITH),

        // constant-cost scalars
        "mod"
        | "pow"
        | "sqrti"
        | "log2"
        | "to-int"
        | "to-uint"
        | "xor"
        | "bit-not"
        | "bit-shift-left"
        | "bit-shift-right"
        | ">="
        | "<="
        | "<"
        | ">"
        | "not"
        | "some"
        | "ok"
        | "err"
        | "is-ok"
        | "is-none"
        | "is-err"
        | "is-some"
        | "default-to"
        | "unwrap!"
        | "unwrap-err!"
        | "unwrap-panic"
        | "unwrap-err-panic"
        | "try!"
        | "is-standard"
        | "principal-destruct?"
        | "principal-construct?"
        | "string-to-int?"
        | "string-to-uint?"
        | "int-to-ascii"
        | "int-to-utf8"
        | "to-ascii?"
        | "buff-to-int-le"
        | "buff-to-uint-le"
        | "buff-to-int-be"
        | "buff-to-uint-be"
        | "element-at"
        | "element-at?" => Some(&COST_ELEM_AT),

        // hash
        "hash160" => Some(&COST_HASH160),
        "sha256" => Some(&COST_SHA256),
        "sha512" => Some(&COST_SHA512),
        "sha512/256" => Some(&COST_SHA512T),
        "keccak256" => Some(&COST_KECCAK),

        // Clarity 6
        "verify-merkle-proof" => Some(&COST_MERKLE),
        "get-bitcoin-tx-output?" => Some(&COST_BTCOUT),
        "ed25519-verify" => Some(&COST_ED25519),
        "secp256k1-decompress?" => Some(&COST_SECP_DC),

        // sequences
        "len" => Some(&COST_LEN),
        "concat" => Some(&COST_CONCAT),
        "append" => Some(&COST_APPEND),
        "index-of" | "index-of?" => Some(&COST_INDEX_OF),
        "slice?" => Some(&COST_SLICE),
        "as-max-len?" => Some(&COST_LEN), // same model
        "replace-at?" => Some(&COST_LEN),

        // tuple
        "tuple" => Some(&COST_TUPLE_CONS),
        "get" => Some(&COST_TUPLE_GET),
        "merge" => Some(&COST_TUPLE_MERGE),

        // misc
        "print" => Some(&COST_PRINT),
        "let" => Some(&COST_ARITH), // binding count
        "to-consensus-buff?" => Some(&COST_CONS_BUF),
        "from-consensus-buff?" => Some(&COST_CONS_BUF),
        "is-eq" => Some(&COST_LEN), // scales with serialized size

        // integer cast aliases, secp256k1
        "secp256k1-recover?" | "secp256k1-verify" | "secp256r1-verify" => Some(&COST_CONST1),

        // SpecialFunctions without a separately exported runtime cost:
        // and, or, if, match, map, filter, fold, list,
        // var-get, var-set, map-get?, map-set, map-insert, map-delete,
        // contract-call?, stx-*, ft-*, nft-*, get-block-info?, …
        _ => None,
    }
}
