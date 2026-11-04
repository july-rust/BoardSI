# D_027 — PADS Same-Layer 45° Oblique Via-A Endpoint Source Contact Qualification

- Scope: **SOURCE_GEOMETRY_ONLY**, not manufactured/plated copper or cross-layer electrical continuity.
- Tracker: [Issue #421](https://github.com/july-rust/BoardSI/issues/421), kept open until this documentation PR and exact docs-main Push checks pass.
- Code PR: [#422](https://github.com/july-rust/BoardSI/pull/422), already **MERGED** via protected GitHub Verified one-parent Squash.
- First recorded / bounded code acceptance / evidence: **2026-11-04 KST**.
- Rust CLI: `--d027-oblique-via-a-endpoint-contact-v1 NET_A 1 4 0 1`.
- Fixture: [PADS V9.4 MILS D_027 Source](../../crates/boardsi-import/tests/fixtures/d027_oblique_via_a_endpoint_v94.asc).

## 1. Frozen exact Source-only contract

PADS POWERPCB V9.4 MILS has `0.01 mil = 254 nm` integer Source coordinate ticks in the bounded import format. Layer 4 / NET_A Main Trace is `(15000,25000)→(35000,25000)`; independent Branch Trace `(15000,25000)→(5000,35000)`. Via-A is `(15000,25000)`; unlike D_026 Via-B, the nonshared Branch endpoint is exterior to Main on its negative-X side. Both Branch endpoint orders are allowed with exact geometry proof.

Rust `i128` dot/cross applies nonzero oblique right triangle `abs(dot)==abs(cross)` and **exteriorProjection < 0** relative to the Main start; no floating tolerance. The independent Qt geometry rebuild checks the same source ticks, Source inventory, import revision/SHA-256, exact expected wire, and Via-A pad ownership. The bounded witness has:

- **8** Source graph nodes, **6** Layer-local Source contact candidate edges, **3** local connected components, and **0 proven electrical cross-layer edges**.
- Via-A layer-4 pad node **1**, Main layer-4 Trace node **5**, Branch Trace node **7**; candidate `EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4` and `EDGE|4|5|7|SOURCE_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4`.
- Wire markers `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`; final `END|FAIL_CLOSED_NO_BARREL_PROOF`.

Reject: one-Source-tick coordinate gap/offset/angle change, collinear/perpendicular/contact at wrong Via, incorrect Net/Layer, degenerate/extra or incomplete Source objects, unsupported Source, stale/forged/cross-mode receipt, and fabricated electrical promotion. Legacy D_015–D_026 stays separate.

## 2. Exact code PR HEAD — independently verified

- Code PR [#422](https://github.com/july-rust/BoardSI/pull/422) final HEAD [`d163bfc66a2e4a2a76ed6dd028d6718021d7ba80`](https://github.com/july-rust/BoardSI/commit/d163bfc66a2e4a2a76ed6dd028d6718021d7ba80/checks): **34/34 Check Runs SUCCESS; 11/11 Pull Request Workflows SUCCESS**, including all 17 active strict Ruleset required contexts.
- Rust D_027 `tests/d027_oblique_via_a_endpoint.rs`: **3/3 Debug and 3/3 Release PASS** (Numerical Core run [41763295425](https://github.com/july-rust/BoardSI/actions/runs/41763295425)). Executed exact witness, reversed Branch/lexical SHA behavior, and hostile/wrong mode/source tests.
- Qt Linux 6.8.3 Offscreen **49/49 CTest PASS** including D_027 `qt-preview-d027-oblique-via-a-endpoint-source` (Qt run [41763295377](https://github.com/july-rust/BoardSI/actions/runs/41763295377)); same exact HEAD native Qt Windows x64, Windows ARM64 and macOS ARM64 jobs **SUCCESS**, with D_027 Native Qt step.
- Independent same HEAD Codex review `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved review threads**.
- Historical first review on `4b9d7cf` flagged a genuine **P1 forgery-test no-op**: replacement target equaled source, preventing rejection of the forged receipt. The new exact HEAD changed Via-A Pad node 1 to Via-B Pad node 2 in forged evidence, verified the negative case, resolved the original review thread, and independently re-reviewed the final HEAD with no findings. Do not erase this history or report initial-head PASS.

## 3. Protected code merge and exact SHA post-merge proof

- Protected signed Verified one-parent Squash on [`main@2de28563231aeb15c92e695ebd17b5d73bfc200e`](https://github.com/july-rust/BoardSI/commit/2de28563231aeb15c92e695ebd17b5d73bfc200e): GitHub `commit.verification.verified=true`, `reason=valid`, sole parent `b66afa1342af54f542867b3c391deb8c4993f187`.
- Independent exact **post-code-main** verification: **28/28 Check Runs SUCCESS, 8/8 event=push Workflow Runs SUCCESS**, zero pending/failed/cancelled at final check. E.g. [Qt runtime Push](https://github.com/july-rust/BoardSI/actions/runs/41764659506) and [Numerical Core Push](https://github.com/july-rust/BoardSI/actions/runs/41764659200).
- Source-only/Qt acceptance is **not** actual PCB manufactured electrical connection proof or full M1/M2 product acceptance.

## 4. Exclusions and documentation gate

**NOT VERIFIED / UNSUPPORTED:** real manufactured Copper fill, physically plated Via Barrel conduction, boardwide Open/Short, Return Current/P009, PH-08 physical certification, RLGC/S-parameter Solver, physical Scenario/Cache, formal U09/U10/U39 or T02/T04/T24/T52, release readiness. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. Legacy design-only [PR #35](https://github.com/july-rust/BoardSI/pull/35) was closed unmerged on 2026-11-04 KST; no M2 G5 implementation inferred.

**Docs-only PR gate:** only `.md` files. Native Qt/Rust compiler runners should not allocate for Markdown-only change; existing classifier must preserve required check contexts as terminal SUCCESS or intentional SKIPPED, not silently weaken Ruleset. Before docs merge, independently verify exact docs HEAD required checks, Codex same-HEAD no-findings review and zero unresolved review threads. Then protected Verified one-parent Squash and independent exact docs-main Push checks; only after that administratively close Issue #421. Documentation acceptance is distinct from the code-main evidence above.
