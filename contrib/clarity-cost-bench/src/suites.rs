use crate::snippet::{ElemType, Snippet};

pub struct Suite {
    pub function: &'static str,
    pub cases: &'static [Case],
}

pub enum Execution {
    Snippet(Snippet),
    Contract {
        source: fn(u64) -> String,
        fn_name: &'static str,
        initial_ustx: u128,
    },
}

pub struct Case {
    pub variant: &'static str,
    pub n_unit: &'static str,
    pub sizes: &'static [u64],
    pub execution: Execution,
}

// ── macros ───────────────────────────────────────────────────────────────────

/// Stateless case — all fields derived from the Snippet.
macro_rules! snip {
    ($x:expr) => {
        Case {
            variant: $x.variant(),
            n_unit: $x.n_unit(),
            sizes: $x.default_sizes(),
            execution: Execution::Snippet($x),
        }
    };
}

/// Stateful case — deploys a Clarity contract in an in-memory environment.
macro_rules! ctr {
    ($src:expr, $fn:expr, $ustx:expr) => {
        Case {
            variant: "contract",
            n_unit: "ignored",
            sizes: &[1],
            execution: Execution::Contract {
                source: $src,
                fn_name: $fn,
                initial_ustx: $ustx,
            },
        }
    };
    ($v:expr, $u:expr, $sz:expr, $src:expr, $fn:expr, $ustx:expr) => {
        Case {
            variant: $v,
            n_unit: $u,
            sizes: $sz,
            execution: Execution::Contract {
                source: $src,
                fn_name: $fn,
                initial_ustx: $ustx,
            },
        }
    };
}

// ── contract source generators ───────────────────────────────────────────────

fn src_var_get(n: u64) -> String {
    format!(
        "(define-data-var x (buff {n}) 0x{})\n(define-public (bench) (ok (var-get x)))",
        "ab".repeat(n.max(1) as usize)
    )
}
fn src_var_set(n: u64) -> String {
    let b = format!("0x{}", "ab".repeat(n.max(1) as usize));
    format!("(define-data-var x (buff {n}) {b})\n(define-public (bench) (ok (var-set x {b})))")
}
fn src_map_get(n: u64) -> String {
    format!("(define-map m uint (buff {n}))\n(define-public (bench) (ok (map-get? m u1)))")
}
fn src_map_set(n: u64) -> String {
    let b = format!("0x{}", "ab".repeat(n.max(1) as usize));
    format!("(define-map m uint (buff {n}))\n(define-public (bench) (ok (map-set m u1 {b})))")
}
fn src_map_insert(n: u64) -> String {
    let b = format!("0x{}", "ab".repeat(n.max(1) as usize));
    format!(
        "(define-map m uint (buff {n}))\n\
         (define-data-var ctr uint u0)\n\
         (define-public (bench)\n\
           (let ((k (+ (var-get ctr) u1))) (var-set ctr k) (ok (map-insert m k {b}))))"
    )
}
fn src_map_delete(n: u64) -> String {
    let b = format!("0x{}", "ab".repeat(n.max(1) as usize));
    format!(
        "(define-map m uint (buff {n}))\n\
         (define-data-var ctr uint u0)\n\
         (define-public (bench)\n\
           (let ((k (+ (var-get ctr) u1))) (var-set ctr k) (map-set m k {b}) (ok (map-delete m k))))"
    )
}
fn src_ft_mint(_: u64) -> String {
    "(define-fungible-token tok)\n(define-public (bench) (ft-mint? tok u1000 tx-sender))".into()
}
fn src_ft_balance(_: u64) -> String {
    "(define-fungible-token tok)\n(define-public (bench) (ok (ft-get-balance tok tx-sender)))"
        .into()
}
fn src_ft_transfer(_: u64) -> String {
    "(define-fungible-token tok)\n\
     (define-constant dst 'SPAXYA5XS51713FDTQ8H94EJ4V579CXMTRNBZKSF)\n\
     (define-public (bench) (begin (try! (ft-mint? tok u1000 tx-sender)) (ft-transfer? tok u1 tx-sender dst)))".into()
}
fn src_ft_supply(_: u64) -> String {
    "(define-fungible-token tok)\n(define-public (bench) (ok (ft-get-supply tok)))".into()
}
fn src_ft_burn(_: u64) -> String {
    "(define-fungible-token tok)\n\
     (define-public (bench) (begin (try! (ft-mint? tok u1000 tx-sender)) (ft-burn? tok u1 tx-sender)))".into()
}
fn src_nft_mint(n: u64) -> String {
    format!(
        "(define-non-fungible-token tok uint)\n\
         (define-data-var ctr uint u0)\n\
         (define-public (bench) (let ((id (+ (var-get ctr) u{n}))) (var-set ctr id) (nft-mint? tok id tx-sender)))"
    )
}
fn src_nft_owner(_: u64) -> String {
    "(define-non-fungible-token tok uint)\n\
     (define-public (bench) (begin (try! (nft-mint? tok u1 tx-sender)) (ok (nft-get-owner? tok u1))))".into()
}
fn src_nft_transfer(_: u64) -> String {
    "(define-non-fungible-token tok uint)\n\
     (define-constant dst 'SPAXYA5XS51713FDTQ8H94EJ4V579CXMTRNBZKSF)\n\
     (define-data-var ctr uint u0)\n\
     (define-public (bench) (let ((id (+ (var-get ctr) u1))) (var-set ctr id) \
       (try! (nft-mint? tok id tx-sender)) (nft-transfer? tok id tx-sender dst)))"
        .into()
}
fn src_nft_burn(_: u64) -> String {
    "(define-non-fungible-token tok uint)\n\
     (define-data-var ctr uint u0)\n\
     (define-public (bench) (let ((id (+ (var-get ctr) u1))) (var-set ctr id) \
       (try! (nft-mint? tok id tx-sender)) (nft-burn? tok id tx-sender)))"
        .into()
}
fn src_stx_balance(_: u64) -> String {
    "(define-public (bench) (ok (stx-get-balance tx-sender)))".into()
}
fn src_stx_transfer(_: u64) -> String {
    "(define-constant dst 'SPAXYA5XS51713FDTQ8H94EJ4V579CXMTRNBZKSF)\n\
     (define-public (bench) (stx-transfer? u1 tx-sender dst))"
        .into()
}
fn src_stx_burn(_: u64) -> String {
    "(define-public (bench) (stx-burn? u1 tx-sender))".into()
}
fn src_stx_account(_: u64) -> String {
    "(define-public (bench) (ok (stx-account tx-sender)))".into()
}
fn src_stx_transfer_memo(_: u64) -> String {
    "(define-constant dst 'SPAXYA5XS51713FDTQ8H94EJ4V579CXMTRNBZKSF)\n\
     (define-public (bench) (stx-transfer-memo? u1 tx-sender dst 0x00))"
        .into()
}
fn src_stacks_block_info(_: u64) -> String {
    "(define-public (bench) (ok (get-stacks-block-info? id-header-hash u0)))".into()
}
fn src_tenure_info(_: u64) -> String {
    "(define-public (bench) (ok (get-tenure-info? time u0)))".into()
}

fn src_contract_hash(_: u64) -> String {
    "(define-public (bench) (ok (contract-hash? (as-contract tx-sender))))".into()
}

// ── helpers ──────────────────────────────────────────────────────────────────

// Wrap a cases array in a Suite.  Used only in the static tables below.
macro_rules! suite {
    ($fn:expr, $cases:expr) => {
        Suite {
            function: $fn,
            cases: $cases,
        }
    };
}

// ── ALL_SUITES ───────────────────────────────────────────────────────────────

pub static ALL_SUITES: &[Suite] = &[
    suite!(
        "+",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int))
        ]
    ),
    suite!(
        "-",
        &[snip!(Snippet::VarLeadUint), snip!(Snippet::VarLeadInt)]
    ),
    suite!(
        "*",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int))
        ]
    ),
    suite!(
        "/",
        &[snip!(Snippet::VarLeadUint), snip!(Snippet::VarLeadInt)]
    ),
    suite!(
        "mod",
        &[
            snip!(Snippet::Fixed("uint", "u100 u7")),
            snip!(Snippet::Fixed("int", "100 7"))
        ]
    ),
    suite!(
        "pow",
        &[
            snip!(Snippet::Fixed("uint", "u2 u10")),
            snip!(Snippet::Fixed("int", "2 10"))
        ]
    ),
    suite!("sqrti", &[snip!(Snippet::Fixed("uint", "u1000000"))]),
    suite!("log2", &[snip!(Snippet::Fixed("uint", "u1024"))]),
    suite!("to-int", &[snip!(Snippet::Fixed("uint", "u42"))]),
    suite!("to-uint", &[snip!(Snippet::Fixed("int", "42"))]),
    suite!(
        "xor",
        &[snip!(Snippet::XorPair), snip!(Snippet::XorPairInt)]
    ),
    suite!(
        ">=",
        &[
            snip!(Snippet::Fixed("uint", "u10 u5")),
            snip!(Snippet::Fixed("int", "10 5"))
        ]
    ),
    suite!(
        "<=",
        &[
            snip!(Snippet::Fixed("uint", "u5 u10")),
            snip!(Snippet::Fixed("int", "5 10"))
        ]
    ),
    suite!(
        "<",
        &[
            snip!(Snippet::Fixed("uint", "u5 u10")),
            snip!(Snippet::Fixed("int", "5 10"))
        ]
    ),
    suite!(
        ">",
        &[
            snip!(Snippet::Fixed("uint", "u10 u5")),
            snip!(Snippet::Fixed("int", "10 5"))
        ]
    ),
    suite!(
        "is-eq",
        &[
            snip!(Snippet::Fixed("uint", "u42 u42")),
            snip!(Snippet::Fixed("int", "42 42")),
            snip!(Snippet::Fixed("bool", "true true")),
            snip!(Snippet::TwoStr),
            snip!(Snippet::TwoBuf),
            snip!(Snippet::TwoList(ElemType::Uint)),
            snip!(Snippet::TwoList(ElemType::Int)),
            snip!(Snippet::TwoList(ElemType::Bool)),
            snip!(Snippet::TwoList(ElemType::AsciiStr)),
            snip!(Snippet::TwoList(ElemType::Buf)),
            snip!(Snippet::TwoList(ElemType::OptUint)),
        ]
    ),
    suite!("and", &[snip!(Snippet::VarOf(ElemType::Bool))]),
    suite!("or", &[snip!(Snippet::VarFalse)]),
    suite!(
        "not",
        &[
            snip!(Snippet::Fixed("true", "true")),
            snip!(Snippet::Fixed("false", "false"))
        ]
    ),
    suite!(
        "if",
        &[
            snip!(Snippet::Fixed("true-branch", "true u1 u2")),
            snip!(Snippet::Fixed("false-branch", "false u1 u2"))
        ]
    ),
    suite!("let", &[snip!(Snippet::LetUint)]),
    suite!("begin", &[snip!(Snippet::VarOf(ElemType::Uint))]),
    suite!("asserts!", &[snip!(Snippet::Fixed("true", "true u0"))]),
    suite!(
        "match",
        &[
            snip!(Snippet::Fixed("some", "(some u1) val val u0")),
            snip!(Snippet::Fixed("none", "none val u0 u1")),
            snip!(Snippet::Fixed("ok", "(ok u1) v v e u0")),
            snip!(Snippet::Fixed("err", "(err u1) v u0 e e")),
        ]
    ),
    // map/filter/fold: only function-compatible element types
    suite!(
        "map",
        &[
            snip!(Snippet::MapList {
                elem: ElemType::Bool,
                fn_name: "not"
            }),
            snip!(Snippet::MapList {
                elem: ElemType::Uint,
                fn_name: "to-int"
            }),
            snip!(Snippet::MapList {
                elem: ElemType::OptUint,
                fn_name: "is-some"
            }),
        ]
    ),
    suite!(
        "filter",
        &[
            snip!(Snippet::FilterList {
                elem: ElemType::Bool,
                fn_name: "not"
            }),
            snip!(Snippet::FilterList {
                elem: ElemType::OptUint,
                fn_name: "is-some"
            }),
        ]
    ),
    suite!(
        "fold",
        &[
            snip!(Snippet::FoldList {
                elem: ElemType::Uint,
                fn_name: "+",
                init: "u0"
            }),
            snip!(Snippet::FoldList {
                elem: ElemType::Bool,
                fn_name: "or",
                init: "false"
            }),
        ]
    ),
    // all element types for the generic sequence operations
    suite!(
        "append",
        &[
            snip!(Snippet::AppendList(ElemType::Uint)),
            snip!(Snippet::AppendList(ElemType::Int)),
            snip!(Snippet::AppendList(ElemType::Bool)),
            snip!(Snippet::AppendList(ElemType::AsciiStr)),
            snip!(Snippet::AppendList(ElemType::Buf)),
            snip!(Snippet::AppendList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "concat",
        &[
            snip!(Snippet::TwoHalfStr),
            snip!(Snippet::TwoHalfBuf),
            snip!(Snippet::TwoHalfList(ElemType::Uint)),
            snip!(Snippet::TwoHalfList(ElemType::Int)),
            snip!(Snippet::TwoHalfList(ElemType::Bool)),
            snip!(Snippet::TwoHalfList(ElemType::AsciiStr)),
            snip!(Snippet::TwoHalfList(ElemType::Buf)),
            snip!(Snippet::TwoHalfList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "as-max-len?",
        &[
            snip!(Snippet::AsMaxLenStr),
            snip!(Snippet::AsMaxLenBuf),
            snip!(Snippet::AsMaxLenList(ElemType::Uint)),
            snip!(Snippet::AsMaxLenList(ElemType::Int)),
            snip!(Snippet::AsMaxLenList(ElemType::Bool)),
            snip!(Snippet::AsMaxLenList(ElemType::AsciiStr)),
            snip!(Snippet::AsMaxLenList(ElemType::Buf)),
            snip!(Snippet::AsMaxLenList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "len",
        &[
            snip!(Snippet::ApplyStr),
            snip!(Snippet::ApplyBuf),
            snip!(Snippet::ApplyUtf8),
            snip!(Snippet::ApplyList(ElemType::Uint)),
            snip!(Snippet::ApplyList(ElemType::Int)),
            snip!(Snippet::ApplyList(ElemType::Bool)),
            snip!(Snippet::ApplyList(ElemType::AsciiStr)),
            snip!(Snippet::ApplyList(ElemType::Buf)),
            snip!(Snippet::ApplyList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "element-at",
        &[
            snip!(Snippet::ElementAtStr),
            snip!(Snippet::ElementAtBuf),
            snip!(Snippet::ElementAtList(ElemType::Uint)),
            snip!(Snippet::ElementAtList(ElemType::Int)),
            snip!(Snippet::ElementAtList(ElemType::Bool)),
            snip!(Snippet::ElementAtList(ElemType::AsciiStr)),
            snip!(Snippet::ElementAtList(ElemType::Buf)),
            snip!(Snippet::ElementAtList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "element-at?",
        &[
            snip!(Snippet::ElementAtList(ElemType::Uint)),
            snip!(Snippet::ElementAtList(ElemType::Int)),
            snip!(Snippet::ElementAtList(ElemType::Bool)),
            snip!(Snippet::ElementAtList(ElemType::AsciiStr)),
            snip!(Snippet::ElementAtList(ElemType::Buf)),
            snip!(Snippet::ElementAtList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "index-of",
        &[
            snip!(Snippet::IndexOfStr),
            snip!(Snippet::IndexOfBuf),
            snip!(Snippet::IndexOfList(ElemType::Uint)),
            snip!(Snippet::IndexOfList(ElemType::Int)),
            snip!(Snippet::IndexOfList(ElemType::Bool)),
            snip!(Snippet::IndexOfList(ElemType::AsciiStr)),
            snip!(Snippet::IndexOfList(ElemType::Buf)),
            snip!(Snippet::IndexOfList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "index-of?",
        &[
            snip!(Snippet::IndexOfList(ElemType::Uint)),
            snip!(Snippet::IndexOfList(ElemType::Int)),
            snip!(Snippet::IndexOfList(ElemType::Bool)),
            snip!(Snippet::IndexOfList(ElemType::AsciiStr)),
            snip!(Snippet::IndexOfList(ElemType::Buf)),
            snip!(Snippet::IndexOfList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "list",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int)),
            snip!(Snippet::VarOf(ElemType::Bool)),
            snip!(Snippet::VarOf(ElemType::AsciiStr)),
            snip!(Snippet::VarOf(ElemType::Buf)),
            snip!(Snippet::VarOf(ElemType::OptUint)),
        ]
    ),
    suite!(
        "tuple",
        &[
            snip!(Snippet::TupleOf(ElemType::Uint)),
            snip!(Snippet::TupleOf(ElemType::Int)),
            snip!(Snippet::TupleOf(ElemType::Bool)),
            snip!(Snippet::TupleOf(ElemType::AsciiStr)),
            snip!(Snippet::TupleOf(ElemType::Buf)),
            snip!(Snippet::TupleOf(ElemType::OptUint)),
        ]
    ),
    suite!("get", &[snip!(Snippet::TupleGet)]),
    suite!("merge", &[snip!(Snippet::TupleMerge)]),
    suite!("hash160", &[snip!(Snippet::ApplyBuf)]),
    suite!("sha256", &[snip!(Snippet::ApplyBuf)]),
    suite!("sha512", &[snip!(Snippet::ApplyBuf)]),
    suite!("sha512/256", &[snip!(Snippet::ApplyBuf)]),
    suite!("keccak256", &[snip!(Snippet::ApplyBuf)]),
    suite!(
        "secp256k1-recover?",
        &[snip!(Snippet::Fixed(
            "fixed-inputs",
            // hash: (buff 32), signature: (buff 65)
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
             0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        ))]
    ),
    suite!(
        "secp256k1-verify",
        &[snip!(Snippet::Fixed(
            "fixed-inputs",
            // hash: (buff 32), signature: (buff 65), public-key: (buff 33)
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
             0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
             0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        ))]
    ),
    suite!(
        "secp256r1-verify",
        &[snip!(Snippet::Fixed(
            "fixed-inputs",
            // hash: (buff 32), signature: (buff 64), public-key: (buff 33)
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
             0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
             0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        ))]
    ),
    suite!(
        "print",
        &[
            snip!(Snippet::Fixed("uint", "u42")),
            snip!(Snippet::ApplyStr),
            snip!(Snippet::ApplyBuf)
        ]
    ),
    suite!("some", &[snip!(Snippet::Fixed("uint", "u42"))]),
    suite!("ok", &[snip!(Snippet::Fixed("uint", "u42"))]),
    suite!("err", &[snip!(Snippet::Fixed("uint", "u42"))]),
    suite!(
        "default-to",
        &[
            snip!(Snippet::Fixed("some", "u0 (some u1)")),
            snip!(Snippet::Fixed("none", "u0 none"))
        ]
    ),
    suite!("unwrap!", &[snip!(Snippet::Fixed("some", "(some u1) u0"))]),
    suite!(
        "unwrap-err!",
        &[snip!(Snippet::Fixed("err", "(err u1) u0"))]
    ),
    suite!(
        "unwrap-panic",
        &[snip!(Snippet::Fixed("some", "(some u1)"))]
    ),
    suite!(
        "unwrap-err-panic",
        &[snip!(Snippet::Fixed("err", "(err u1)"))]
    ),
    suite!(
        "try!",
        &[
            snip!(Snippet::Fixed("ok", "(ok u1)")),
            snip!(Snippet::Fixed("some", "(some u1)"))
        ]
    ),
    suite!(
        "is-ok",
        &[
            snip!(Snippet::Fixed("ok", "(ok u1)")),
            snip!(Snippet::Fixed("err", "(err u1)"))
        ]
    ),
    suite!(
        "is-none",
        &[
            snip!(Snippet::Fixed("none", "none")),
            snip!(Snippet::Fixed("some", "(some u1)"))
        ]
    ),
    suite!(
        "is-err",
        &[
            snip!(Snippet::Fixed("err", "(err u1)")),
            snip!(Snippet::Fixed("ok", "(ok u1)"))
        ]
    ),
    suite!(
        "is-some",
        &[
            snip!(Snippet::Fixed("some", "(some u1)")),
            snip!(Snippet::Fixed("none", "none"))
        ]
    ),
    suite!("buff-to-int-le", &[snip!(Snippet::BuffToInt)]),
    suite!("buff-to-uint-le", &[snip!(Snippet::BuffToInt)]),
    suite!("buff-to-int-be", &[snip!(Snippet::BuffToInt)]),
    suite!("buff-to-uint-be", &[snip!(Snippet::BuffToInt)]),
    suite!(
        "is-standard",
        &[snip!(Snippet::Fixed(
            "standard-principal",
            "'SP2C2YFP12AJZB4MABJBAJ55XECVS7E4PMMZ89YZR"
        ))]
    ),
    suite!(
        "principal-destruct?",
        &[snip!(Snippet::Fixed(
            "standard-principal",
            "'SP2C2YFP12AJZB4MABJBAJ55XECVS7E4PMMZ89YZR"
        ))]
    ),
    suite!(
        "principal-construct?",
        &[
            snip!(Snippet::Fixed(
                "standard",
                "0x16 0xfa6bf38ed557fe417333710d6033e9419391a320"
            )),
            snip!(Snippet::Fixed(
                "contract",
                "0x16 0xfa6bf38ed557fe417333710d6033e9419391a320 \"my-contract\""
            )),
        ]
    ),
    suite!(
        "string-to-int?",
        &[
            snip!(Snippet::StringToIntAscii),
            snip!(Snippet::StringToIntUtf8)
        ]
    ),
    suite!(
        "string-to-uint?",
        &[
            snip!(Snippet::StringToIntAscii),
            snip!(Snippet::StringToIntUtf8)
        ]
    ),
    suite!(
        "int-to-ascii",
        &[
            snip!(Snippet::Fixed("uint", "u123456789")),
            snip!(Snippet::Fixed("int", "-123456789"))
        ]
    ),
    suite!(
        "int-to-utf8",
        &[
            snip!(Snippet::Fixed("uint", "u123456789")),
            snip!(Snippet::Fixed("int", "-123456789"))
        ]
    ),
    suite!(
        "bit-and",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int))
        ]
    ),
    suite!(
        "bit-or",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int))
        ]
    ),
    suite!(
        "bit-not",
        &[
            snip!(Snippet::Fixed("uint", "u42")),
            snip!(Snippet::Fixed("int", "42"))
        ]
    ),
    suite!(
        "bit-shift-left",
        &[
            snip!(Snippet::Fixed("uint", "u1 u4")),
            snip!(Snippet::Fixed("int", "1 u4"))
        ]
    ),
    suite!(
        "bit-shift-right",
        &[
            snip!(Snippet::Fixed("uint", "u256 u4")),
            snip!(Snippet::Fixed("int", "256 u4"))
        ]
    ),
    suite!(
        "bit-xor",
        &[
            snip!(Snippet::VarOf(ElemType::Uint)),
            snip!(Snippet::VarOf(ElemType::Int))
        ]
    ),
    suite!(
        "slice?",
        &[
            snip!(Snippet::SliceStr),
            snip!(Snippet::SliceBuf),
            snip!(Snippet::SliceList(ElemType::Uint)),
            snip!(Snippet::SliceList(ElemType::Int)),
            snip!(Snippet::SliceList(ElemType::Bool)),
            snip!(Snippet::SliceList(ElemType::AsciiStr)),
            snip!(Snippet::SliceList(ElemType::Buf)),
            snip!(Snippet::SliceList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "replace-at?",
        &[
            snip!(Snippet::ReplaceAtStr),
            snip!(Snippet::ReplaceAtBuf),
            snip!(Snippet::ReplaceAtList(ElemType::Uint)),
            snip!(Snippet::ReplaceAtList(ElemType::Int)),
            snip!(Snippet::ReplaceAtList(ElemType::Bool)),
            snip!(Snippet::ReplaceAtList(ElemType::AsciiStr)),
            snip!(Snippet::ReplaceAtList(ElemType::Buf)),
            snip!(Snippet::ReplaceAtList(ElemType::OptUint)),
        ]
    ),
    suite!(
        "to-consensus-buff?",
        &[
            snip!(Snippet::Fixed("uint", "u42")),
            snip!(Snippet::ToConsensusBuf)
        ]
    ),
    suite!(
        "from-consensus-buff?",
        &[
            snip!(Snippet::Fixed(
                "int",
                "int 0x000000000000000000000000000000000000"
            )),
            snip!(Snippet::FromConsensusBuf)
        ]
    ),
    suite!("to-ascii?", &[snip!(Snippet::ApplyUtf8)]),
    // Clarity 6
    suite!("verify-merkle-proof", &[snip!(Snippet::VerifyMerkleProof)]),
    suite!(
        "get-bitcoin-tx-output?",
        &[snip!(Snippet::GetBitcoinTxOutput)]
    ),
    suite!("ed25519-verify", &[snip!(Snippet::Ed25519Verify)]),
    suite!(
        "secp256k1-decompress?",
        &[snip!(Snippet::Fixed(
            "compressed-key",
            // compressed public key: (buff 33) = 0x02 prefix + 32-byte x-coordinate
            "0x02abababababababababababababababababababababababababababababababab"
        ))]
    ),
];

// ── STATEFUL_SUITES ──────────────────────────────────────────────────────────

const BUFS: &[u64] = &[1, 4, 16, 64, 256, 512, 1024];

pub static STATEFUL_SUITES: &[Suite] = &[
    suite!(
        "var-get",
        &[ctr!("buffer", "value_bytes", BUFS, src_var_get, "bench", 0)]
    ),
    suite!(
        "var-set",
        &[ctr!("buffer", "value_bytes", BUFS, src_var_set, "bench", 0)]
    ),
    suite!(
        "map-get?",
        &[ctr!("buffer", "value_bytes", BUFS, src_map_get, "bench", 0)]
    ),
    suite!(
        "map-set",
        &[ctr!("buffer", "value_bytes", BUFS, src_map_set, "bench", 0)]
    ),
    suite!(
        "map-insert",
        &[ctr!(
            "buffer",
            "value_bytes",
            BUFS,
            src_map_insert,
            "bench",
            0
        )]
    ),
    suite!(
        "map-delete",
        &[ctr!(
            "buffer",
            "value_bytes",
            BUFS,
            src_map_delete,
            "bench",
            0
        )]
    ),
    suite!("ft-mint?", &[ctr!(src_ft_mint, "bench", 0)]),
    suite!("ft-get-balance", &[ctr!(src_ft_balance, "bench", 0)]),
    suite!("ft-transfer?", &[ctr!(src_ft_transfer, "bench", 0)]),
    suite!("ft-get-supply", &[ctr!(src_ft_supply, "bench", 0)]),
    suite!("ft-burn?", &[ctr!(src_ft_burn, "bench", 0)]),
    suite!("nft-mint?", &[ctr!(src_nft_mint, "bench", 0)]),
    suite!("nft-get-owner?", &[ctr!(src_nft_owner, "bench", 0)]),
    suite!("nft-transfer?", &[ctr!(src_nft_transfer, "bench", 0)]),
    suite!("nft-burn?", &[ctr!(src_nft_burn, "bench", 0)]),
    suite!("stx-get-balance", &[ctr!(src_stx_balance, "bench", 0)]),
    suite!(
        "stx-transfer?",
        &[ctr!(src_stx_transfer, "bench", 1_000_000_000_000)]
    ),
    suite!(
        "stx-burn?",
        &[ctr!(src_stx_burn, "bench", 1_000_000_000_000)]
    ),
    suite!("stx-account", &[ctr!(src_stx_account, "bench", 0)]),
    suite!(
        "stx-transfer-memo?",
        &[ctr!(src_stx_transfer_memo, "bench", 1_000_000_000_000)]
    ),
    suite!(
        "get-stacks-block-info?",
        &[ctr!(src_stacks_block_info, "bench", 0)]
    ),
    suite!("get-tenure-info?", &[ctr!(src_tenure_info, "bench", 0)]),
    suite!(
        "contract-call?",
        &[ctr!(
            |_| {
                "(define-public (target) (ok true))\n             (define-public (bench) (contract-call? .bench target))".to_string()
            },
            "bench",
            0
        )]
    ),
    suite!("contract-hash?", &[ctr!(src_contract_hash, "bench", 0)]),
];

// ── lookup ───────────────────────────────────────────────────────────────────

pub fn all_suites() -> impl Iterator<Item = &'static Suite> {
    ALL_SUITES.iter().chain(STATEFUL_SUITES.iter())
}

pub fn find(function: &str, variant: &str) -> Option<(&'static Suite, &'static Case)> {
    all_suites().find_map(|s| {
        if s.function != function {
            return None;
        }
        s.cases
            .iter()
            .find(|c| c.variant == variant)
            .map(|c| (s, c))
    })
}
