# M0 first numerical foundation — executed validation

Date: 2026-10-14 (KST). Scope: uniform scalar externally supplied RLGC only.
Baseline main: c62e349bc365a8fc851dc3b5f508db2432792ae4.
Specification v1.0 unchanged; §§8/8.2/11/13.1/14.2/15, U01/U02/U30/U43/U47,
partial evidence toward T06/T07/T43. This does not complete those product tests,
M0/M2, or Issues #3/#4/#16. Those roadmap issues remain open.

## Environment and reproduction

Linux x86_64; rustc 1.75.0 (82e1608df 2023-12-21, Ubuntu source build),
cargo 1.75.0, Python 3.12.14, mpmath 1.2.1. Zero third-party Rust dependencies.
1.75.0 is a provisional numerical-only baseline selected to match the executed
local compiler; final product/Qt toolchain and three-native-platform tests
remain §15 decisions. Rustup binary distribution/CI is a separate execution.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline -- --nocapture
cargo test --workspace --release --locked --offline -- --nocapture
# Optional oracle regeneration, requires Python + mpmath 1.2.1:
python3 tools/validation/generate_reference.py
```

A minimal additional Linux CI runs format, Clippy, debug and release tests.
Existing planning-document CI is unchanged. Its schema/ID assertions were also
checked against the fetched unchanged baseline files. Remote CI status belongs
to the PR/checks page and is not assumed from local success.

## Executed evidence

| Test/evidence | Result |
| --- | --- |
| Required lossless 50 Ω, 100 mm, 1 GHz half-wave, delay | PASS |
| Lossless DC, lossy DC Unsupported, zero-length identity | PASS |
| Independent mismatched/lossy ABCD oracle | PASS, 72 cases |
| Lossy attenuation/negative transmission phase, reciprocity | PASS |
| Negative/nonfinite input fields, zero L/C/Zref | PASS |
| Long/high-loss, extreme impedance/magnitude, overflow guards | PASS |
| Tiny mismatched reflection, exp(-700) nonzero transmission | PASS |
| Largest singular value, elementwise false-pass counterexample | PASS |
| Per-frequency success/error isolation | PASS |
| Sensitive long mismatched resonance is explicitly rejected | PASS |
| Debug and optimized builds | PASS, 9 tests each; 0 failures |
| Format / Clippy with -D warnings | PASS, no compiler/Clippy warnings |
| Actual PCB/RLGC extraction/EM/measurement comparison | Not Run |
| Windows x64, Windows ARM64, macOS arm64 native runs | Not Run |
| Whole-envelope proof of accuracy or all-band causality | Not Run |

Metrics identical in executed debug/release builds:

- Maximum complex S absolute error over 72 ABCD cases: **3.668604232247687e-12**
  (target <=1e-6). All four matrix entries compared.
- Required lossless impedance relative error: **0** at f64 resolution.
- Required lossless phase-delay relative error: **2.220446049250313e-16**
  (both target <=0.001, i.e. 0.1%). Expected delay 0.5 ns and impedance 50 Ω.
- Maximum sampled passivity residual: **0** at f64 resolution;
  maximum sampled reciprocity residual: **0** (target <=1e-6).
  Independent normal-matrix check uses max(|S11+S21|,|S11-S21|);
  production uses eigenvalue of S^H*S, not elementwise bounds.

The CSV is generated with 80-decimal-digit Python ABCD/cosh/sinh calculations,
serialized at 45 significant digits, then parsed to f64 by cargo test. Production
uses bounded negative exponential/scattering, separate roots, and cancellation
avoidance. Additional half-wave and distortionless checks anchor phase, delay,
impedance and attenuation independently. See the self-contained telegrapher /
power-wave derivation in [the contract](../architecture/numerical-contract.md)
and [the generator](../../tools/validation/generate_reference.py).
CSV SHA-256: b8a8ae1322cd0187b184c8fb15f2b373871e03e431feb81f4b9820750d0f79e4.
All fixtures are synthetic equations; no company PCB, Stackup or Touchstone data.

## Limitations and next decisions

- Successful results are physically **Approximate**, never automatically
  Validated. Numerical agreement does not certify externally supplied RLGC.
- Lossy nonzero-length DC, degenerate L/C, unequal/complex references,
  multi-conductor/differential lines and geometry-derived RLGC are unsupported.
  No GUI, import, progressive sweep/cache, packages or releases were started.
- Numerical limits: lossy abs(z),abs(y) in [1e-120,1e120]; abs(Zc/Zref) in
  [1e-6,1e6]; alpha*length <=700; beta*length <=1e6 rad. Denominator and
  sensitivity guards may reject additional points. The sensitivity estimate
  is conservative, not a formal forward-error bound. Tiny components can
  underflow; arbitrary near-zero relative accuracy is not promised.
- 72 samples establish those numerical comparisons, not accuracy everywhere
  in the envelope or a material/geometry/frequency applicability certificate.
- The custom scalar complex helper is limited to this bounded calculation;
  it is not a substitute for a future reviewed matrix algebra backend.

Next decisions (maximum three):
1. Select final product toolchain and verify three native architectures (#3).
2. Define RLGC provenance and physical validation records in the independent
   Canonical Model; ODB++ remains only one importer (#4).
3. Choose independently validated geometry-to-RLGC/DC extensions and reference
   datasets before enlarging supported model scope (#16).
