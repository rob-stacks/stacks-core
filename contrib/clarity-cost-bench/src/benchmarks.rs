/// A single benchmark: one Clarity function exercised across a range of input sizes.
pub struct Benchmark {
    /// Unique identifier, used as the `--bench-id` argument in `run` mode.
    pub id: &'static str,
    /// Human-readable description shown in `analyze` output.
    pub description: &'static str,
    /// What the `size` parameter represents (e.g. "buffer_bytes", "list_length").
    pub dimension: &'static str,
    /// Sizes to test.
    pub sizes: &'static [u64],
    /// Returns a Clarity 6 snippet that exercises the function at the given size.
    pub generate: fn(u64) -> String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn hex_buf(bytes: u64) -> String {
    format!("0x{}", "ab".repeat(bytes as usize))
}

fn ascii_str(len: u64) -> String {
    format!("\"{}\"", "a".repeat(len as usize))
}

fn uint_list(len: u64, elem: &str) -> String {
    let elems = std::iter::repeat(elem).take(len as usize).collect::<Vec<_>>().join(" ");
    format!("(list {})", elems)
}

fn bool_list(len: u64) -> String {
    uint_list(len, "false")
}

// ---------------------------------------------------------------------------
// Size ranges
// ---------------------------------------------------------------------------

const SMALL_SIZES: &[u64]  = &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const LIST_SIZES: &[u64]   = &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1000];
const VARIADIC_SIZES: &[u64] = &[1, 2, 4, 8, 16, 32, 64, 128];

// ---------------------------------------------------------------------------
// Benchmark catalog
// ---------------------------------------------------------------------------

pub static BENCHMARKS: &[Benchmark] = &[
    // --- Hash functions -------------------------------------------------------
    Benchmark {
        id: "hash160",
        description: "hash160 on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(hash160 {})", hex_buf(n)),
    },
    Benchmark {
        id: "sha256",
        description: "sha256 on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(sha256 {})", hex_buf(n)),
    },
    Benchmark {
        id: "sha512",
        description: "sha512 on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(sha512 {})", hex_buf(n)),
    },
    Benchmark {
        id: "sha512/256",
        description: "sha512/256 on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(sha512/256 {})", hex_buf(n)),
    },
    Benchmark {
        id: "keccak256",
        description: "keccak256 on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(keccak256 {})", hex_buf(n)),
    },

    // --- Clarity 6 crypto functions ------------------------------------------
    // secp256k1-verify: (secp256k1-verify msg-hash sig pubkey)
    // Fixed-size inputs – we vary nothing, but we still measure the baseline.
    Benchmark {
        id: "secp256k1-verify",
        description: "secp256k1-verify (fixed-size inputs)",
        dimension: "calls",
        sizes: &[1],
        generate: |_| {
            // 32-byte msg hash, 65-byte sig (recoverable), 33-byte compressed pubkey
            // These are syntactically valid buffers; the call will return false but executes.
            "(secp256k1-verify \
                0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
                0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc)".to_string()
        },
    },
    Benchmark {
        id: "secp256k1-decompress",
        description: "secp256k1-decompress? (fixed-size 33-byte compressed key)",
        dimension: "calls",
        sizes: &[1],
        generate: |_| {
            "(secp256k1-decompress? \
                0x02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa)".to_string()
        },
    },
    Benchmark {
        id: "ed25519-verify",
        description: "ed25519-verify (fixed 32-byte msg, 64-byte sig, 32-byte pubkey)",
        dimension: "calls",
        sizes: &[1],
        generate: |_| {
            "(ed25519-verify \
                0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
                0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc)".to_string()
        },
    },

    // --- String / buffer functions -------------------------------------------
    Benchmark {
        id: "len-string",
        description: "len on an ASCII string",
        dimension: "string_chars",
        sizes: SMALL_SIZES,
        generate: |n| format!("(len {})", ascii_str(n)),
    },
    Benchmark {
        id: "len-buffer",
        description: "len on a buffer",
        dimension: "buffer_bytes",
        sizes: SMALL_SIZES,
        generate: |n| format!("(len {})", hex_buf(n)),
    },
    Benchmark {
        id: "concat-string",
        description: "concat two ASCII strings",
        dimension: "total_chars",
        sizes: SMALL_SIZES,
        generate: |n| {
            let half = n / 2;
            format!("(concat {} {})", ascii_str(half), ascii_str(half))
        },
    },
    Benchmark {
        id: "concat-buffer",
        description: "concat two buffers",
        dimension: "total_bytes",
        sizes: SMALL_SIZES,
        generate: |n| {
            let half = n / 2;
            format!("(concat {} {})", hex_buf(half), hex_buf(half))
        },
    },
    Benchmark {
        id: "to-utf8",
        description: "to-utf8 on an ASCII string",
        dimension: "string_chars",
        sizes: SMALL_SIZES,
        generate: |n| format!("(to-utf8 {})", ascii_str(n)),
    },
    Benchmark {
        id: "to-ascii",
        description: "to-ascii on a UTF-8 string",
        dimension: "string_chars",
        sizes: SMALL_SIZES,
        generate: |n| format!("(to-ascii u\"{}\")", "a".repeat(n as usize)),
    },

    // --- List functions ------------------------------------------------------
    Benchmark {
        id: "len-list",
        description: "len on a list",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(len {})", uint_list(n, "u1")),
    },
    Benchmark {
        id: "append-list",
        description: "append an element to a list",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(append {} u1)", uint_list(n, "u1")),
    },
    Benchmark {
        id: "map-not",
        description: "map not over a list of booleans",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(map not {})", bool_list(n)),
    },
    Benchmark {
        id: "filter-not",
        description: "filter not over a list of booleans",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(filter not {})", bool_list(n)),
    },
    Benchmark {
        id: "fold-add",
        description: "fold + over a list of uints",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(fold + {} u0)", uint_list(n, "u1")),
    },
    Benchmark {
        id: "index-of",
        description: "index-of? searching a list of uints (last position)",
        dimension: "list_length",
        sizes: LIST_SIZES,
        // element is at the end to exercise the worst case
        generate: |n| {
            let elems = (0..n).map(|i| format!("u{}", i)).collect::<Vec<_>>().join(" ");
            format!("(index-of? (list {}) u{})", elems, n - 1)
        },
    },
    Benchmark {
        id: "element-at",
        description: "element-at? on a list",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(element-at? {} u0)", uint_list(n, "u1")),
    },
    Benchmark {
        id: "slice",
        description: "slice? the full length of a list",
        dimension: "list_length",
        sizes: LIST_SIZES,
        generate: |n| format!("(slice? {} u0 u{})", uint_list(n, "u1"), n),
    },

    // --- Arithmetic (variadic) -----------------------------------------------
    Benchmark {
        id: "add-variadic",
        description: "variadic + with N uint arguments",
        dimension: "arg_count",
        sizes: VARIADIC_SIZES,
        generate: |n| {
            let args = std::iter::repeat("u1").take(n as usize).collect::<Vec<_>>().join(" ");
            format!("(+ {})", args)
        },
    },
    Benchmark {
        id: "mul-variadic",
        description: "variadic * with N uint arguments",
        dimension: "arg_count",
        sizes: VARIADIC_SIZES,
        generate: |n| {
            let args = std::iter::repeat("u1").take(n as usize).collect::<Vec<_>>().join(" ");
            format!("(* {})", args)
        },
    },
    Benchmark {
        id: "is-eq-string",
        description: "is-eq comparing two equal ASCII strings",
        dimension: "string_chars",
        sizes: SMALL_SIZES,
        generate: |n| format!("(is-eq {} {})", ascii_str(n), ascii_str(n)),
    },
];

pub fn find(id: &str) -> Option<&'static Benchmark> {
    BENCHMARKS.iter().find(|b| b.id == id)
}
