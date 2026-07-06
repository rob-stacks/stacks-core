/// Element / field type for parameterised snippet variants.
/// Every variant that accepts a sequence or tuple uses this to range over
/// all valid Clarity value types, instead of hard-coding one per variant.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElemType {
    Uint,     // u42 / u0, u1, … for indexed use
    Int,      // 42  / 0, 1, …
    Bool,     // true
    AsciiStr, // "a"   (1-char fixed literal)
    Buf,      // 0xab  (1-byte fixed literal)
    OptUint,  // (some u42)
}

impl ElemType {
    /// Short label used as suffix in `variant()` strings.
    pub const fn label(&self) -> &'static str {
        match self {
            ElemType::Uint => "uint",
            ElemType::Int => "int",
            ElemType::Bool => "bool",
            ElemType::AsciiStr => "string-ascii",
            ElemType::Buf => "buffer",
            ElemType::OptUint => "optional-uint",
        }
    }

    /// A single fixed literal value for this type (used as append arg, tuple field value, etc.).
    pub const fn lit(&self) -> &'static str {
        match self {
            ElemType::Uint => "u42",
            ElemType::Int => "42",
            ElemType::Bool => "true",
            ElemType::AsciiStr => "\"a\"",
            ElemType::Buf => "0xab",
            ElemType::OptUint => "(some u42)",
        }
    }

    /// Build a list of n elements.  Uint/Int use indexed values so that
    /// worst-case `index-of` searches can target the last position.
    /// All other types repeat the fixed literal.
    pub fn list(&self, n: u64) -> String {
        let elems: Vec<String> = match self {
            ElemType::Uint => (0..n.max(1)).map(|i| format!("u{i}")).collect(),
            ElemType::Int => (0..n.max(1)).map(|i| format!("{i}")).collect(),
            other => (0..n.max(1)).map(|_| other.lit().to_string()).collect(),
        };
        format!("(list {})", elems.join(" "))
    }

    /// The worst-case search target for `index-of`: last element for
    /// indexed types, a non-existent value for fixed-literal types.
    pub fn index_of_target(&self, n: u64) -> String {
        match self {
            ElemType::Uint => format!("u{}", n - 1),
            ElemType::Int => format!("{}", n as i64 - 1),
            ElemType::Bool => "false".to_string(), // list is all-true → not found
            ElemType::AsciiStr => "\"z\"".to_string(), // list is all-"a" → not found
            ElemType::Buf => "0xcc".to_string(),   // list is all-0xab → not found
            ElemType::OptUint => "none".to_string(), // list is all-(some …) → not found
        }
    }
}

// ── Snippet enum ─────────────────────────────────────────────────────────────

/// Declarative description of how to build a Clarity 6 snippet.
///
/// `generate(op, n)` produces the Clarity source:
///   `op`  — outer function name from `Suite::function` (never repeated here)
///   `n`   — size / count for this benchmark point
///
/// Variants carrying `&'static str` hold an *inner* function name
/// (for map/filter/fold), not the outer one.
#[derive(Clone, Copy)]
pub enum Snippet {
    /// `Fixed("label", "arg1 arg2 …")` → `"({op} arg1 arg2 …)"`, n ignored.
    Fixed(&'static str, &'static str),

    // ── single buffer/string apply ───────────────────────────────────────
    ApplyBuf,  // (op 0xab…)   n=bytes
    ApplyStr,  // (op "aa…")   n=chars
    ApplyUtf8, // (op u"aa…")  n=chars

    // ── single list apply ────────────────────────────────────────────────
    /// `(op (list e0 e1 …))` n=length
    ApplyList(ElemType),

    // ── two-arg: each side gets n/2 (concat-style) ───────────────────────
    TwoHalfBuf,
    TwoHalfStr,
    /// `(op (list …) (list …))` n=total elements
    TwoHalfList(ElemType),

    // ── two-arg: each side gets full n (is-eq-style) ─────────────────────
    TwoBuf,
    TwoStr,
    /// `(op list list)` n each
    TwoList(ElemType),

    // ── variadic n args ──────────────────────────────────────────────────
    /// `(op e0 e1 … e{n-1})`
    VarOf(ElemType),
    /// `(op u{n×1000} u1 … u1)` — large first arg for `-`, `/`
    VarLeadUint,
    VarLeadInt,
    /// `(op false false …)` — for `or` worst case (all false)
    VarFalse,

    // ── append: (op list lit) ─────────────────────────────────────────────
    AppendList(ElemType),

    // ── sequence + index 0 ────────────────────────────────────────────────
    ElementAtBuf,
    ElementAtStr,
    ElementAtList(ElemType),

    // ── worst-case search ─────────────────────────────────────────────────
    IndexOfStr,
    IndexOfBuf,
    IndexOfList(ElemType),

    // ── slice / replace / as-max-len ─────────────────────────────────────
    SliceBuf,
    SliceStr,
    SliceList(ElemType),
    ReplaceAtBuf,
    ReplaceAtStr,
    ReplaceAtList(ElemType),
    AsMaxLenBuf,
    AsMaxLenStr,
    AsMaxLenList(ElemType),

    // ── higher-order (inner function kept, outer from suite) ─────────────
    /// `(op fn_name (list …))`
    MapList {
        elem: ElemType,
        fn_name: &'static str,
    },
    FilterList {
        elem: ElemType,
        fn_name: &'static str,
    },
    /// `(op fn_name (list …) init)`
    FoldList {
        elem: ElemType,
        fn_name: &'static str,
        init: &'static str,
    },

    // ── tuple ─────────────────────────────────────────────────────────────
    /// `(op (f0 val) (f1 val) …)` n=fields
    TupleOf(ElemType),
    /// `(get f0 (tuple …))`
    TupleGet,
    /// `(merge (tuple …) (tuple …))`
    TupleMerge,

    // ── conversions ───────────────────────────────────────────────────────
    BuffToInt,
    StringToIntAscii,
    StringToIntUtf8,
    ToConsensusBuf,
    FromConsensusBuf,

    // ── misc ──────────────────────────────────────────────────────────────
    LetUint,
    XorPair,
    XorPairInt,

    // ── Clarity 6 ─────────────────────────────────────────────────────────
    VerifyMerkleProof,
    GetBitcoinTxOutput,
    Ed25519Verify,
}

// ── derived metadata (const) ─────────────────────────────────────────────────

impl Snippet {
    pub const fn variant(&self) -> &'static str {
        match self {
            Snippet::Fixed(label, _) => label,

            // buffers
            Snippet::ApplyBuf
            | Snippet::TwoHalfBuf
            | Snippet::TwoBuf
            | Snippet::SliceBuf
            | Snippet::ReplaceAtBuf
            | Snippet::AsMaxLenBuf
            | Snippet::BuffToInt
            | Snippet::ToConsensusBuf
            | Snippet::FromConsensusBuf
            | Snippet::GetBitcoinTxOutput
            | Snippet::Ed25519Verify
            | Snippet::ElementAtBuf
            | Snippet::IndexOfBuf => "buffer",

            // strings
            Snippet::ApplyStr
            | Snippet::TwoHalfStr
            | Snippet::TwoStr
            | Snippet::SliceStr
            | Snippet::ReplaceAtStr
            | Snippet::AsMaxLenStr
            | Snippet::StringToIntAscii
            | Snippet::IndexOfStr
            | Snippet::ElementAtStr => "string-ascii",

            Snippet::ApplyUtf8 | Snippet::StringToIntUtf8 => "string-utf8",

            // variadic scalars
            Snippet::VarLeadUint | Snippet::XorPair | Snippet::LetUint => "uint",
            Snippet::VarLeadInt | Snippet::XorPairInt => "int",
            Snippet::VarFalse => "bool-false",

            // ElemType-parameterised scalar
            Snippet::VarOf(e) => e.label(),

            // ElemType-parameterised list
            Snippet::ApplyList(ElemType::Uint)
            | Snippet::TwoHalfList(ElemType::Uint)
            | Snippet::TwoList(ElemType::Uint)
            | Snippet::AppendList(ElemType::Uint)
            | Snippet::ElementAtList(ElemType::Uint)
            | Snippet::IndexOfList(ElemType::Uint)
            | Snippet::SliceList(ElemType::Uint)
            | Snippet::ReplaceAtList(ElemType::Uint)
            | Snippet::AsMaxLenList(ElemType::Uint)
            | Snippet::MapList {
                elem: ElemType::Uint,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::Uint,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::Uint,
                ..
            } => "list-uint",

            Snippet::ApplyList(ElemType::Int)
            | Snippet::TwoHalfList(ElemType::Int)
            | Snippet::TwoList(ElemType::Int)
            | Snippet::AppendList(ElemType::Int)
            | Snippet::ElementAtList(ElemType::Int)
            | Snippet::IndexOfList(ElemType::Int)
            | Snippet::SliceList(ElemType::Int)
            | Snippet::ReplaceAtList(ElemType::Int)
            | Snippet::AsMaxLenList(ElemType::Int)
            | Snippet::MapList {
                elem: ElemType::Int,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::Int,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::Int,
                ..
            } => "list-int",

            Snippet::ApplyList(ElemType::Bool)
            | Snippet::TwoHalfList(ElemType::Bool)
            | Snippet::TwoList(ElemType::Bool)
            | Snippet::AppendList(ElemType::Bool)
            | Snippet::ElementAtList(ElemType::Bool)
            | Snippet::IndexOfList(ElemType::Bool)
            | Snippet::SliceList(ElemType::Bool)
            | Snippet::ReplaceAtList(ElemType::Bool)
            | Snippet::AsMaxLenList(ElemType::Bool)
            | Snippet::MapList {
                elem: ElemType::Bool,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::Bool,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::Bool,
                ..
            } => "list-bool",

            Snippet::ApplyList(ElemType::AsciiStr)
            | Snippet::TwoHalfList(ElemType::AsciiStr)
            | Snippet::TwoList(ElemType::AsciiStr)
            | Snippet::AppendList(ElemType::AsciiStr)
            | Snippet::ElementAtList(ElemType::AsciiStr)
            | Snippet::IndexOfList(ElemType::AsciiStr)
            | Snippet::SliceList(ElemType::AsciiStr)
            | Snippet::ReplaceAtList(ElemType::AsciiStr)
            | Snippet::AsMaxLenList(ElemType::AsciiStr)
            | Snippet::MapList {
                elem: ElemType::AsciiStr,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::AsciiStr,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::AsciiStr,
                ..
            } => "list-string-ascii",

            Snippet::ApplyList(ElemType::Buf)
            | Snippet::TwoHalfList(ElemType::Buf)
            | Snippet::TwoList(ElemType::Buf)
            | Snippet::AppendList(ElemType::Buf)
            | Snippet::ElementAtList(ElemType::Buf)
            | Snippet::IndexOfList(ElemType::Buf)
            | Snippet::SliceList(ElemType::Buf)
            | Snippet::ReplaceAtList(ElemType::Buf)
            | Snippet::AsMaxLenList(ElemType::Buf)
            | Snippet::MapList {
                elem: ElemType::Buf,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::Buf,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::Buf,
                ..
            } => "list-buffer",

            Snippet::ApplyList(ElemType::OptUint)
            | Snippet::TwoHalfList(ElemType::OptUint)
            | Snippet::TwoList(ElemType::OptUint)
            | Snippet::AppendList(ElemType::OptUint)
            | Snippet::ElementAtList(ElemType::OptUint)
            | Snippet::IndexOfList(ElemType::OptUint)
            | Snippet::SliceList(ElemType::OptUint)
            | Snippet::ReplaceAtList(ElemType::OptUint)
            | Snippet::AsMaxLenList(ElemType::OptUint)
            | Snippet::MapList {
                elem: ElemType::OptUint,
                ..
            }
            | Snippet::FilterList {
                elem: ElemType::OptUint,
                ..
            }
            | Snippet::FoldList {
                elem: ElemType::OptUint,
                ..
            } => "list-optional-uint",

            // tuple
            Snippet::TupleOf(ElemType::Uint) => "tuple-uint",
            Snippet::TupleOf(ElemType::Int) => "tuple-int",
            Snippet::TupleOf(ElemType::Bool) => "tuple-bool",
            Snippet::TupleOf(ElemType::AsciiStr) => "tuple-string-ascii",
            Snippet::TupleOf(ElemType::Buf) => "tuple-buffer",
            Snippet::TupleOf(ElemType::OptUint) => "tuple-optional-uint",

            Snippet::TupleGet => "tuple-uint",
            Snippet::TupleMerge => "tuple",
            Snippet::VerifyMerkleProof => "siblings",
        }
    }

    pub const fn n_unit(&self) -> &'static str {
        match self {
            Snippet::Fixed(_, _) | Snippet::XorPair | Snippet::XorPairInt => "ignored",

            Snippet::ApplyBuf
            | Snippet::TwoHalfBuf
            | Snippet::TwoBuf
            | Snippet::SliceBuf
            | Snippet::ReplaceAtBuf
            | Snippet::AsMaxLenBuf
            | Snippet::BuffToInt
            | Snippet::ToConsensusBuf
            | Snippet::FromConsensusBuf
            | Snippet::GetBitcoinTxOutput
            | Snippet::Ed25519Verify
            | Snippet::ElementAtBuf
            | Snippet::IndexOfBuf => "buffer_bytes",

            Snippet::ApplyStr
            | Snippet::TwoHalfStr
            | Snippet::TwoStr
            | Snippet::SliceStr
            | Snippet::ReplaceAtStr
            | Snippet::AsMaxLenStr
            | Snippet::StringToIntAscii
            | Snippet::StringToIntUtf8
            | Snippet::ApplyUtf8
            | Snippet::IndexOfStr
            | Snippet::ElementAtStr => "string_chars",

            // all list operations
            Snippet::ApplyList(_)
            | Snippet::TwoHalfList(_)
            | Snippet::TwoList(_)
            | Snippet::AppendList(_)
            | Snippet::ElementAtList(_)
            | Snippet::IndexOfList(_)
            | Snippet::SliceList(_)
            | Snippet::ReplaceAtList(_)
            | Snippet::AsMaxLenList(_)
            | Snippet::MapList { .. }
            | Snippet::FilterList { .. }
            | Snippet::FoldList { .. } => "list_length",

            Snippet::VarOf(_) | Snippet::VarLeadUint | Snippet::VarLeadInt | Snippet::VarFalse => {
                "arg_count"
            }

            Snippet::TupleOf(_) | Snippet::TupleGet => "field_count",
            Snippet::TupleMerge => "total_field_count",
            Snippet::LetUint => "binding_count",
            Snippet::VerifyMerkleProof => "sibling_count",
        }
    }

    pub const fn default_sizes(&self) -> &'static [u64] {
        match self {
            Snippet::Fixed(_, _) | Snippet::XorPair | Snippet::XorPairInt => &[1],

            Snippet::ApplyBuf
            | Snippet::TwoHalfBuf
            | Snippet::TwoBuf
            | Snippet::SliceBuf
            | Snippet::ReplaceAtBuf
            | Snippet::AsMaxLenBuf
            | Snippet::ToConsensusBuf
            | Snippet::FromConsensusBuf
            | Snippet::GetBitcoinTxOutput
            | Snippet::Ed25519Verify
            | Snippet::ElementAtBuf
            | Snippet::IndexOfBuf
            | Snippet::ApplyStr
            | Snippet::TwoHalfStr
            | Snippet::TwoStr
            | Snippet::SliceStr
            | Snippet::ReplaceAtStr
            | Snippet::AsMaxLenStr
            | Snippet::IndexOfStr
            | Snippet::ElementAtStr
            | Snippet::ApplyUtf8 => &[1, 4, 16, 64, 256, 512, 1024],

            Snippet::ApplyList(_)
            | Snippet::TwoHalfList(_)
            | Snippet::TwoList(_)
            | Snippet::AppendList(_)
            | Snippet::ElementAtList(_)
            | Snippet::IndexOfList(_)
            | Snippet::SliceList(_)
            | Snippet::ReplaceAtList(_)
            | Snippet::AsMaxLenList(_)
            | Snippet::MapList { .. }
            | Snippet::FilterList { .. }
            | Snippet::FoldList { .. } => &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1000],

            Snippet::VarOf(_) | Snippet::VarLeadUint | Snippet::VarLeadInt | Snippet::VarFalse => {
                &[1, 2, 4, 8, 16, 32, 64, 128]
            }

            Snippet::BuffToInt => &[1, 2, 4, 8, 16],

            Snippet::StringToIntAscii | Snippet::StringToIntUtf8 => &[1, 2, 4, 8, 16, 20],

            Snippet::TupleOf(_) | Snippet::TupleGet | Snippet::TupleMerge | Snippet::LetUint => {
                &[1, 2, 4, 8, 16, 32]
            }

            Snippet::VerifyMerkleProof => &[1, 2, 4, 8, 16, 24],
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn buf(n: u64) -> String {
    format!("0x{}", "ab".repeat(n.max(1) as usize))
}
fn astr(n: u64) -> String {
    format!("\"{}\"", "a".repeat(n.max(1) as usize))
}
fn ustr(n: u64) -> String {
    format!("u\"{}\"", "a".repeat(n.max(1) as usize))
}

// ── generate ─────────────────────────────────────────────────────────────────

impl Snippet {
    pub fn generate(&self, op: &str, n: u64) -> String {
        match self {
            Snippet::Fixed(_, args) => format!("({op} {args})"),

            // buffer/string sequences
            Snippet::ApplyBuf => format!("({op} {})", buf(n)),
            Snippet::ApplyStr => format!("({op} {})", astr(n)),
            Snippet::ApplyUtf8 => format!("({op} {})", ustr(n)),

            Snippet::TwoHalfBuf => {
                let h = n.max(2) / 2;
                format!("({op} {} {})", buf(h), buf(h))
            }
            Snippet::TwoHalfStr => {
                let h = n.max(2) / 2;
                format!("({op} {} {})", astr(h), astr(h))
            }

            Snippet::TwoBuf => format!("({op} {} {})", buf(n), buf(n)),
            Snippet::TwoStr => format!("({op} {} {})", astr(n), astr(n)),

            Snippet::SliceBuf => format!("({op} {} u0 u{})", buf(n), n / 2),
            Snippet::SliceStr => format!("({op} {} u0 u{})", astr(n), n / 2),
            Snippet::ReplaceAtBuf => format!("({op} {} u0 0xcc)", buf(n)),
            Snippet::ReplaceAtStr => format!("({op} {} u0 \"z\")", astr(n)),
            Snippet::AsMaxLenBuf => format!("({op} {} u1024)", buf(n)),
            Snippet::AsMaxLenStr => format!("({op} {} u1024)", astr(n)),
            Snippet::ElementAtBuf => format!("({op} {} u0)", buf(n)),
            Snippet::ElementAtStr => format!("({op} {} u0)", astr(n)),
            Snippet::IndexOfBuf => format!("({op} {} 0xcc)", buf(n)), // 0xcc not in all-0xab buf
            Snippet::IndexOfStr => format!("({op} {} \"z\")", astr(n)), // "z" not in all-"a" str

            // list sequences
            Snippet::ApplyList(e) => format!("({op} {})", e.list(n)),
            Snippet::TwoHalfList(e) => {
                let h = n.max(2) / 2;
                format!("({op} {} {})", e.list(h), e.list(h))
            }
            Snippet::TwoList(e) => format!("({op} {} {})", e.list(n), e.list(n)),
            Snippet::AppendList(e) => format!("({op} {} {})", e.list(n), e.lit()),
            Snippet::ElementAtList(e) => format!("({op} {} u0)", e.list(n)),
            Snippet::IndexOfList(e) => format!("({op} {} {})", e.list(n), e.index_of_target(n)),
            Snippet::SliceList(e) => format!("({op} {} u0 u{})", e.list(n), n / 2),
            Snippet::ReplaceAtList(e) => format!("({op} {} u0 {})", e.list(n), e.lit()),
            Snippet::AsMaxLenList(e) => format!("({op} {} u2000)", e.list(n)),

            // variadic
            Snippet::VarOf(e) => {
                let elems = match e {
                    ElemType::Uint => (0..n.max(1)).map(|i| format!("u{i}")).collect::<Vec<_>>(),
                    ElemType::Int => (0..n.max(1)).map(|i| format!("{i}")).collect::<Vec<_>>(),
                    other => (0..n.max(1))
                        .map(|_| other.lit().to_string())
                        .collect::<Vec<_>>(),
                };
                format!("({op} {})", elems.join(" "))
            }
            Snippet::VarLeadUint => {
                let mut a = vec![format!("u{}", n.max(1) * 1000)];
                a.extend((1..n.max(1)).map(|_| "u1".to_string()));
                format!("({op} {})", a.join(" "))
            }
            Snippet::VarLeadInt => {
                let mut a = vec![format!("{}", n.max(1) as i64 * 1000)];
                a.extend((1..n.max(1)).map(|_| "1".to_string()));
                format!("({op} {})", a.join(" "))
            }
            Snippet::VarFalse => {
                format!("({op} {})", vec!["false"; n.max(1) as usize].join(" "))
            }

            // higher-order
            Snippet::MapList { elem, fn_name } => format!("({op} {fn_name} {})", elem.list(n)),
            Snippet::FilterList { elem, fn_name } => format!("({op} {fn_name} {})", elem.list(n)),
            Snippet::FoldList {
                elem,
                fn_name,
                init,
            } => {
                format!("({op} {fn_name} {} {init})", elem.list(n))
            }

            // tuple
            Snippet::TupleOf(e) => {
                let fields: Vec<_> = (0..n.max(1))
                    .map(|i| format!("(f{i} {})", e.lit()))
                    .collect();
                format!("({op} {})", fields.join(" "))
            }
            Snippet::TupleGet => {
                let fields: Vec<_> = (0..n.max(1)).map(|i| format!("(f{i} u{i})")).collect();
                format!("({op} f0 (tuple {}))", fields.join(" "))
            }
            Snippet::TupleMerge => {
                let h = n.max(2) / 2;
                let a: Vec<_> = (0..h).map(|i| format!("(fa{i} u{i})")).collect();
                let b: Vec<_> = (0..h).map(|i| format!("(fb{i} u{i})")).collect();
                format!("({op} (tuple {}) (tuple {}))", a.join(" "), b.join(" "))
            }

            // conversions
            Snippet::BuffToInt => format!("({op} {})", buf(n.min(16))),
            Snippet::StringToIntAscii => {
                format!("({op} \"{}\")", "1".repeat(n.min(20).max(1) as usize))
            }
            Snippet::StringToIntUtf8 => {
                format!("({op} u\"{}\")", "1".repeat(n.min(20).max(1) as usize))
            }
            Snippet::ToConsensusBuf => format!("({op} {})", buf(n)),
            Snippet::FromConsensusBuf => {
                let len_be = format!("{:08x}", n.min(255));
                let data = "aa".repeat(n.min(255) as usize);
                format!("({op} (buff {}) 0x02{len_be}{data})", n.min(255))
            }

            // misc
            Snippet::LetUint => {
                let b: Vec<_> = (0..n.max(1)).map(|i| format!("(x{i} u{i})")).collect();
                format!("(let ({}) u0)", b.join(" "))
            }
            Snippet::XorPair => format!("({op} u{n} u{})", n + 1),
            Snippet::XorPairInt => format!("({op} {n} {})", n + 1),

            // Clarity 6
            Snippet::VerifyMerkleProof => {
                let s: Vec<_> = (0..n.max(1)).map(|_| buf(32)).collect();
                format!(
                    "({op} {} {} u0 u{} (list {}))",
                    buf(32),
                    buf(32),
                    n.max(1) + 1,
                    s.join(" ")
                )
            }
            Snippet::GetBitcoinTxOutput => format!("({op} {} u0)", buf(n)),
            Snippet::Ed25519Verify => format!("({op} {} {} {})", buf(n), buf(64), buf(32)),
        }
    }
}
