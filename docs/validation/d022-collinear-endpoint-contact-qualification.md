# D_022 — Collinear Source Trace endpoint contact qualification

- Tracker: [Issue #406](https://github.com/july-rust/BoardSI/issues/406)
- Initial recorded date: **2026-11-03 KST**; acceptance check date: **2026-11-03 KST**; evidence date: **2026-11-03 KST**; status: **LIMITED SOURCE-ONLY QUALIFICATION ACCEPTED**
- Base protected main: `c0a41cfbd968ac16bbdef95878b5501be2cf61ed`
- Opt-in: `--d022-collinear-endpoint-contact-v1 NET_A 1 4 0 1`
- Synthetic fixture: `crates/boardsi-import/tests/fixtures/d022_collinear_endpoint_v94.asc`

## Frozen limited contract

PADS V9.4 MILS Source tick = 0.01 mil = 254 nm. Main Layer-4 trace (15000,25000) → (35000,25000), branch (35000,25000) → (45000,25000), same NET_A. Shared endpoint (35000,25000), exactly zero centreline overlap length. The main endpoint is also pre-existing Through-Via B's Layer-4 pad; **all three** Source primitives are represented. Candidate graph: 8 nodes / 6 layer-local candidate edges / 3 components; one legitimate three-node Source contact triangle, zero proven electrical cross-layer edges. No manufactured copper or plated Via barrel conduction inferred.

The current bounded topology intentionally supports the main Trace's Via-B endpoint only. Opposite Via-A endpoint would require a different third candidate edge and is fail-closed, not silently accepted. Strict i128 integer collinearity and exterior projection, native source complete inventory and SHA binding. Reverse branch orientation accepted, 1-tick gap/offset, noncollinear bend, zero length, partial/contained overlap, different Net/Layer, unsupported artwork and forged/stale receipts rejected.

## Final code and independent acceptance evidence — 2026-11-03 KST

- Code PR: [#407](https://github.com/july-rust/BoardSI/pull/407), **MERGED** to protected `main`. Final pre-merge code HEAD: [`7b7c1828af94e1692d5404dc00bf7e41b546bed6`](https://github.com/july-rust/BoardSI/commit/7b7c1828af94e1692d5404dc00bf7e41b546bed6/checks).
- Exact final HEAD: **34/34 GitHub PR Check Runs SUCCESS; 11/11 PR Workflows SUCCESS**. Rust format/Clippy/workspace tests, Source hostile regressions, native Qt and independent Linux Qt included.
- Native Qt: Linux x86_64 Qt Offscreen **44/44 CTest PASS** ([Qt PR workflow](https://github.com/july-rust/BoardSI/actions/runs/41646346504)); dedicated `D_022 native Qt source-bound endpoint collinear Trace contact qualification` step **SUCCESS** on Windows x64, Windows ARM64 and macOS ARM64. Rust CLI, independent Qt importer and SHA-bound receipt reconstruction validated; Source stale/forged/electrical-promotion rejection included.
- Independent same-HEAD Codex review: **completed**, `has_any_findings=false`, `has_unresolved_findings=false`; unresolved review threads **0**. The initial P1 newline-serialization finding on superseded HEAD was corrected, replied to and resolved before final review.
- Protected Squash: [`main@f7c1d183a91bfac536f07038d89ae8909247b4f9`](https://github.com/july-rust/BoardSI/commit/f7c1d183a91bfac536f07038d89ae8909247b4f9), parent `c0a41cfbd968ac16bbdef95878b5501be2cf61ed` (exactly one), GitHub commit signature **verified=true/reason=valid**.
- Independent post-main verification of that exact merged SHA: **28/28 Check Runs SUCCESS; 8/8 event=push Workflows SUCCESS**, including [Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41648213420) and [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41648213657).
- [Issue #406](https://github.com/july-rust/BoardSI/issues/406): **CLOSED/completed**; GitHub closed it automatically on code PR merge, **before** the independently checked exact-main post-push CI. This distinction is preserved.
- Earlier failures **not erased**: initial Rustfmt formatting, Clippy `too_many_arguments`, Windows CRLF lexical-test fixture mutation and Qt receipt newline mismatch were followed by corrections and a clean final-HEAD CI; no unverified additional root cause is claimed.
- Administrative checkoff is performed in a separate **Markdown-only** PR, as a one-time evidence reconciliation. Docs-only CI must not allocate platform-native runners when the classifier succeeds.

## Accepted limited scope



- [x] Rust format, strict Clippy and hostile regression on final implementation HEAD.
- [x] Real Qt Import/Reimport, complete SHA-bound Receipt and forged/stale rejection in Linux Qt CTest.
- [x] Windows x64/ARM64 and macOS ARM64 **dedicated D_022 native Qt** source tests.
- [x] Required protected gate and complete 34/34 PR Checks · 11/11 PR Workflows.
- [x] Independent exact-final-HEAD Codex review, zero findings and unresolved threads.
- [x] Protected Verified single-parent Squash + exact-main post-push 28/28 Checks · 8/8 Workflows.
- [x] Issue #406 CLOSED/completed, separately recorded as auto-closed at code merge before post-push verification.


**Explicit exclusions:** manufactured copper, electrical barrel proof, whole-board Open/Short, P009, PH-08, scenario/cache, RLGC/S-parameters, formal U/T or release acceptance. Parent #12 OPEN and Draft #35 unchanged. Historical D_021 Windows ARM64 D006 failed-job retry is not interpreted as root-cause correction.
