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
| Independent mismatched/lossy ABCD oracle | PASS, 175 cases (72 baseline + 103 boundary) |
| Lossy attenuation/negative transmission phase, reciprocity | PASS |
| Negative/nonfinite input fields, zero L/C/Zref | PASS |
| Long/high-loss, extreme impedance/magnitude, overflow guards | PASS |
| Tiny mismatched reflection, exp(-700) nonzero transmission | PASS |
| Largest singular value, elementwise false-pass counterexample | PASS |
| Per-frequency success/error isolation | PASS |
| Sensitive long mismatched resonance is explicitly rejected | PASS |
| Tiny RLGC intermediate underflow preserves finite attenuation | PASS, regression reproduced FAIL before fix |
| Near-resistive positive AC with phase below f64 resolution | PASS; exact f=0 remains Unsupported |
| Debug and optimized builds | PASS, 12 tests each; 0 failures |
| Format / Clippy with -D warnings | PASS, no compiler/Clippy warnings |
| Actual PCB/RLGC extraction/EM/measurement comparison | Not Run |
| Windows x64, Windows ARM64, macOS arm64 native runs | PASS; see [native execution report](m0-native-platforms.md) |
| Whole-envelope proof of accuracy or all-band causality | Not Run |

Metrics identical in executed debug/release builds:

- Maximum complex S absolute error over 175 ABCD cases: **4.09536036253102e-10**
  (target <=1e-6). All four matrix entries compared.
- Required lossless impedance relative error: **0** at f64 resolution.
- Required lossless phase-delay relative error: **2.220446049250313e-16**
  (both target <=0.001, i.e. 0.1%). Expected delay 0.5 ns and impedance 50 Ω.
- Maximum sampled passivity residual: **2.220446049250313e-16**;
  maximum sampled reciprocity residual: **0** (target <=1e-6).
  Independent normal-matrix check uses max(|S11+S21|,|S11-S21|);
  production uses eigenvalue of S^H*S, not elementwise bounds.

The CSV is generated with 80-decimal-digit Python ABCD/cosh/sinh calculations,
serialized at 45 significant digits, then parsed to f64 by cargo test. The 103
boundary cases calculate their references from the exact binary64 input values
read by Rust; the original 72 decimal-input baseline remains unchanged. Production
uses bounded negative exponential/scattering, separate roots, and cancellation
avoidance. Additional half-wave and distortionless checks anchor phase, delay,
impedance and attenuation independently. See the self-contained telegrapher /
power-wave derivation in [the contract](../architecture/numerical-contract.md)
and [the generator](../../tools/validation/generate_reference.py).
CSV SHA-256: b8a8ae1322cd0187b184c8fb15f2b373871e03e431feb81f4b9820750d0f79e4.
Boundary CSV SHA-256: 67ebaa3a208b0d1610cd0baafa018149cfd0b5ea87ad4f1f77bc0d1dda2f797c.
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
- 175 samples establish those numerical comparisons, not accuracy everywhere
  in the envelope or a material/geometry/frequency applicability certificate.
- The custom scalar complex helper is limited to this bounded calculation;
  it is not a substitute for a future reviewed matrix algebra backend.

Next decisions (maximum three):
1. Select final product toolchain and verify three native architectures (#3).
2. Define RLGC provenance and physical validation records in the independent
   Canonical Model; ODB++ remains only one importer (#4).
3. Choose independently validated geometry-to-RLGC/DC extensions and reference
   datasets before enlarging supported model scope (#16).

## Follow-up numerical review (2026-10-14 KST)

A confirmed intermediate-underflow bug was reproduced before the fix. With
R=G=1e-120, L=C=1e-120 in their respective SI per-length units, length=1e120 m,
f=1e-101 Hz, Zref=1 Ω, the exact distortionless response has Zc=1 Ω,
alpha*length=1 and S21 approximately exp(-1)=0.36787944117144233.
The old alpha formula formed R*Im(y)+G*Im(z) first; it rounded to zero and
incorrectly returned S21 approximately 1. This synthetic case tests numerical
scale handling, not a physically realistic PCB.

Reordering the alpha formula to divide each imaginary term by beta before
multiplying its real partner preserves finite attenuation. If beta has rounded
to zero for positive, near-resistive AC, the positive direct real root is
retained, avoiding a 0/0 calculation. Lossy exact DC remains Unsupported.
Two independent distortionless analytic regressions were added. At that review,
local debug/release suites contained 11 tests; all PASS, with unchanged 72-case ABCD
maximum error and no Clippy warnings. Remote CI for the updated commit must
be read from the current PR checks; prior-commit success is not reused.

## Boundary verification expansion (2026-10-14 KST)

Add 103 independent ABCD fixed reference cases using exact binary64 inputs:
40 combinations of impedance ratio and electrical length (including both sides
of a half-wave and phase near 1e6 rad), 45 combinations of unequal R/L and G/C,
attenuation up to alpha*length=699 and mismatched references, and 18 combinations
with R or G zero approaching the DC limit. All 103 points return successful
responses and pass comparisons; no rejected point is silently counted as an
accuracy PASS. High attenuation also has a relative S21 check <=1e-9, preventing
an underflow-to-zero result from passing on absolute tolerance alone.

No numerical core change was needed for these cases. The current debug/release
suites contain 12 passing tests. Overall maximum S error is 4.09536036253102e-10;
the original 72-point baseline maximum remains 3.668604232247687e-12. Maximum
sampled passivity residual is 2.220446049250313e-16; reciprocity remains zero.
These results expand sampled evidence, not a mathematical proof of the entire
numerical envelope or physical applicability. Physical PCB validation remains Not Run; the subsequent
[native execution report](m0-native-platforms.md) records the three target platforms. Source license/toolchain/product gates remain open.
