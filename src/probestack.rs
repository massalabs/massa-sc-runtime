//! Provides the `__rust_probestack` symbol expected by `wasmer-vm` 4.x.
//!
//! Rust >= 1.88 stopped exporting `__rust_probestack` with C linkage (it became
//! an internal, mangled item of `compiler_builtins`). `wasmer-vm` < 6.1.0 still
//! declares it as `extern "C"` to fill its builtin function table, so linking
//! fails on x86_64 targets that use it.
//!
//! `wasmer-vm` only reaches for this symbol on non-Windows x86/x86_64: on
//! aarch64 it uses an empty probestack, and on Windows it uses `__chkstk` /
//! `___chkstk_ms`. So this shim is gated to the targets that need it.
//!
//! TODO: remove this shim once wasmer is bumped to >= 6.1.0, which ships the
//! symbol itself.

// ELF flavour (Linux, BSD, ...).
#[cfg(all(target_arch = "x86_64", not(windows), not(target_vendor = "apple")))]
core::arch::global_asm!(
    r#"
    .pushsection .text.__rust_probestack
    .globl __rust_probestack
    .type  __rust_probestack, @function
    .hidden __rust_probestack
__rust_probestack:
    .cfi_startproc
    pushq  %rbp
    .cfi_adjust_cfa_offset 8
    .cfi_offset %rbp, -16
    movq   %rsp, %rbp
    .cfi_def_cfa_register %rbp

    mov    %rax,%r11
    cmp    $0x1000,%r11
    jna    3f
2:
    sub    $0x1000,%rsp
    test   %rsp,8(%rsp)
    sub    $0x1000,%r11
    cmp    $0x1000,%r11
    ja     2b
3:
    sub    %r11,%rsp
    test   %rsp,8(%rsp)
    add    %rax,%rsp

    leave
    .cfi_def_cfa_register %rsp
    .cfi_adjust_cfa_offset -8
    ret
    .cfi_endproc
    .size __rust_probestack, . - __rust_probestack
    .popsection
    "#,
    options(att_syntax)
);

// Mach-O flavour (x86_64 macOS). Same body, but Mach-O prefixes C symbols with
// an underscore and does not understand the ELF-only `.type` / `.size` /
// `.hidden` directives.
#[cfg(all(target_arch = "x86_64", target_vendor = "apple"))]
core::arch::global_asm!(
    r#"
    .section __TEXT,__text
    .globl ___rust_probestack
    .p2align 4
___rust_probestack:
    .cfi_startproc
    pushq  %rbp
    .cfi_adjust_cfa_offset 8
    .cfi_offset %rbp, -16
    movq   %rsp, %rbp
    .cfi_def_cfa_register %rbp

    mov    %rax,%r11
    cmp    $0x1000,%r11
    jna    3f
2:
    sub    $0x1000,%rsp
    test   %rsp,8(%rsp)
    sub    $0x1000,%r11
    cmp    $0x1000,%r11
    ja     2b
3:
    sub    %r11,%rsp
    test   %rsp,8(%rsp)
    add    %rax,%rsp

    leave
    .cfi_def_cfa_register %rsp
    .cfi_adjust_cfa_offset -8
    ret
    .cfi_endproc
    "#,
    options(att_syntax)
);
