/// Callgrind client requests: start / stop instruction counting.
///
/// When the bench binary runs under `valgrind --tool=callgrind --instr-atstart=no`,
/// these calls act as fence posts — only the code between them is counted.
/// When running outside Valgrind (e.g. the in-process byte-count run in `cmd_bench`)
/// the asm sequence is a harmless near-no-op:
///   4 × `rolq` on an unconstrained scratch register (rdi),
///   1 × `xchgq %rbx,%rbx` (register self-swap — always a no-op).

// Callgrind request codes = VG_USERREQ_TOOL_BASE('C','T') + N
//   VG_USERREQ_TOOL_BASE('C','T') = (0x43 << 24) | (0x54 << 16) = 0x43540000
const _START: usize = 0x43540005;
const _STOP: usize = 0x43540006;

pub fn start_instrumentation() {
    // SAFETY: standard Valgrind client-request protocol; no-op outside Valgrind.
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    unsafe {
        _request(_START)
    };
}

pub fn stop_instrumentation() {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    unsafe {
        _request(_STOP)
    };
}

/// x86_64: 4 × rolq on a scratch register + xchgq %rbx,%rbx
#[cfg(target_arch = "x86_64")]
unsafe fn _request(req: usize) {
    let args: [usize; 6] = [req, 0, 0, 0, 0, 0];
    let _out: usize;
    core::arch::asm!(
        "rolq $3,  %rdi ; rolq $13, %rdi",
        "rolq $61, %rdi ; rolq $51, %rdi",
        "xchgq %rbx, %rbx",
        inout("rdx") 0_usize => _out,
        in("rax") args.as_ptr(),
        out("rdi") _,
        options(att_syntax, nostack, preserves_flags),
    );
}

/// aarch64: 4 × ror x12 + orr x10,x10,x10
/// x3 = default (in) / result (out), x4 = args ptr
#[cfg(target_arch = "aarch64")]
unsafe fn _request(req: usize) {
    let args: [usize; 6] = [req, 0, 0, 0, 0, 0];
    let _out: usize;
    unsafe {
        core::arch::asm!(
            "ror x12, x12, #3",
            "ror x12, x12, #13",
            "ror x12, x12, #51",
            "ror x12, x12, #61",
            "orr x10, x10, x10",
            inout("x3") 0_usize => _out,
            in("x4") args.as_ptr(),
            out("x10") _,
            out("x12") _,
            options(nostack, preserves_flags),
        );
    }
}
