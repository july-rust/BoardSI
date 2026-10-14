# M0 numerical core — native platform verification

Date: 2026-10-14 (KST). Related #3/#16, specification v1.0 §§11.3/13.1/15.
This verifies only the scalar numerical crate and its fixed synthetic references.
It does not finish M0, Qt/bindings/IPC, installation or physical PCB validation.
PR #27 remains unmerged; specification and the final product toolchain are unchanged.

## Selected verification configuration

| Platform | GitHub standard runner | Rust host | Executed status (debug / release) |
| --- | --- | --- | --- |
| Windows x64 | windows-2022 | x86_64-pc-windows-msvc | PASS: 12 / 12 tests |
| Windows ARM64 | windows-11-arm | aarch64-pc-windows-msvc | PASS: 12 / 12 tests |
| macOS ARM64 | macos-14 | aarch64-apple-darwin | PASS: 12 / 12 tests |

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

The actual source/test commit executed was
`a3db3b7b2f996eb5c420a76b5fdff4fcaf584566`; subsequent report-only changes do
not alter numerical code or fixtures. No build artifacts are uploaded, no cache added, no binaries released. Logs
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

## Executed evidence

All jobs completed successfully on the same source commit and pinned Rust 1.75.0.
The following evidence is from actual job logs, not a cross-compilation claim:

| Platform | Executed ARCH / OS | Maximum complex S error (175 points) | Job log |
| --- | --- | --- | --- |
| Windows x64 | x86_64 / windows | 4.09536036253102e-10 | [native job](https://github.com/july-rust/BoardSI/actions/runs/38525574356/job/115725013571) |
| Windows ARM64 | aarch64 / windows | 4.09536036253102e-10 | [native job](https://github.com/july-rust/BoardSI/actions/runs/38525574356/job/115725013576) |
| macOS ARM64 | aarch64 / macos | 4.09536036253102e-10 | [native job](https://github.com/july-rust/BoardSI/actions/runs/38525574356/job/115725013495) |

Each native job recorded the matching rustc host, a successful NATIVE_EXECUTION
identity check, and two `12 passed; 0 failed` results (debug and release).
Maximum sampled passivity residual across the native jobs was
2.220446049250313e-16; reciprocity residual was 0. The 72-point baseline maximum
was 3.668604232247687e-12 and the 103-point boundary maximum was
4.09536036253102e-10 on each platform. Linux format/Clippy/debug/release and
[planning-document CI](https://github.com/july-rust/BoardSI/actions/runs/38525574568)
also passed. Native builds ran with RUSTFLAGS=-D warnings.

[Full source-specific run](https://github.com/july-rust/BoardSI/actions/runs/38525574356).
This changes the native numerical execution status from Not Run to PASS only.
Physical PCB/EM/measurement comparison, Qt/GUI/IPC, deployment/install and
all-band causality remain Not Run. Uniform externally supplied RLGC results
remain physically Approximate. Toolchain selection remains provisional.

## M0 second-review source: current CI evidence

The historical 12-test/175-point runs above are not evidence for the overflow
fix. The second-review suite now has 15 tests and 180 independent ABCD points,
with unchanged native CI configuration. Local Linux Debug/Release, format and
Clippy pass; current-source Windows x64, Windows ARM64 and macOS ARM64 remain
Pending until their job results and logs are inspected. Read current PR #27
checks: https://github.com/july-rust/BoardSI/pull/27/checks .
Physical PCB validation remains Not Run; results remain Approximate.
