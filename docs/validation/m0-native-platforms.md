# M0 numerical core — native platform verification

Date: 2026-10-14 (KST). Related #3/#16, specification v1.0 §§11.3/13.1/15.
This verifies only the scalar numerical crate and its fixed synthetic references.
It does not finish M0, Qt/bindings/IPC, installation or physical PCB validation.
PR #27 remains unmerged; specification and the final product toolchain are unchanged.

## Selected verification configuration

| Platform | GitHub standard runner | Rust host | Initial execution status |
| --- | --- | --- | --- |
| Windows x64 | windows-2022 | x86_64-pc-windows-msvc | Not Run until job completion |
| Windows ARM64 | windows-11-arm | aarch64-pc-windows-msvc | Not Run until job completion |
| macOS ARM64 | macos-14 | aarch64-apple-darwin | Not Run until job completion |

Rust 1.75.0 remains the provisional numerical baseline so platform differences
are compared with the same compiler version and source/fixtures. It is not a
recommendation of the final product toolchain. Rust 1.75.0 classified the two
ARM64 hosts as Tier 2 with Host Tools; availability alone does not prove the
BoardSI tests pass. No upgrade was needed to enable this execution experiment.

Each job checks all three identities before numerical tests:
1. GitHub RUNNER_ARCH equals the expected hardware runner architecture.
2. rustc -vV reports the exact native host triple (not merely a target added to
   an unrelated compiler). RUSTUP_TOOLCHAIN explicitly names that host.
3. A compiled and executed Rust example checks std::env::consts::ARCH/OS and
   f64 storage, printing NATIVE_EXECUTION. Architecture mismatch is a failure.

Then native debug and release run all 12 tests, comparing all four complex S
entries at 175 independent ABCD reference points. Inputs/CSV are identical to
Linux; each platform must independently meet complex absolute error <=1e-6,
passivity and reciprocity residual <=1e-6, and the existing tighter analytic
and relative tiny-S21 checks. RUSTFLAGS=-D warnings rejects compilation warnings.
The existing Linux format/Clippy/accuracy job and planning-document CI are kept.

No build artifacts are uploaded, no cache added, no binaries released. Logs
record compiler/host, executed architecture, test counts and numerical metrics.
A matching native execution plus reference accuracy is the gate; cross-compiling
success alone cannot mark a platform PASS. This is common-reference validation,
not a pairwise comparison of every floating-point result across platforms.

## Official configuration sources (checked 2026-10-14 KST)

- GitHub runner labels and architecture:
  https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- Frozen Rust 1.75.0 platform support, Tier 1/Tier 2 with Host Tools:
  https://github.com/rust-lang/rust/blob/1.75.0/src/doc/rustc/src/platform-support.md
- Current Windows ARM64 image software (dynamic inventory, not a toolchain pin):
  https://github.com/actions/runner-images/blob/main/images/windows/Windows11-Arm64-Readme.md

These sources justify runner/toolchain configuration only. BoardSI execution
results must come from the CI jobs for the actual committed source, never from
the existence of a platform in official documentation.
