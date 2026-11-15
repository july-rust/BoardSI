# E_028 — PADS/KiCad bounded cross-format via geometry qualification

Evidence date: **2026-11-15 KST**. Baseline: protected main 557a9c685f468768ae27a1da81a58beda1a9351c (E_027 #640 merged). Parent: #532 / #520.

**Status: [x] SOURCE-LIMITED QUALIFIED (2026-11-15 KST)** — single 4-copper-layer PADS V9.4/KiCad 9 Through Via common geometry. Code PR 30/30 checks PASS, independent final-HEAD review 0/0, protected verified Squash and exact code-main Push 8/8 workflows (28/28 checks) PASS. **Full Canonical BoardModel cross-format trace/pad equivalence, manufactured electrical contact, vendor fixture and SI/product U/T remain NOT_ACCEPTED.**

## Independent numeric oracle (synthetic original, not vendor)

Two separately parsed raw sources represent one **four-copper-layer through via**, with the following manually specified, integer-nm reference values. Neither importer derives the reference data.

| Parameter | Independent expected value |
|---|---|
| PADS position | (300 mil, 200 mil) |
| Physical centre | (7,620,000 nm, 5,080,000 nm) |
| PADS via diameter | 35 mil = 889,000 nm |
| PADS drill diameter | 15 mil = 381,000 nm |
| Copper order | L1 Top, L2 Inner1, L3 Inner2, L4 Bottom |
| Via Span and type | L1 to L4, Through |
| KiCad source | version 20241229, F.Cu/In1.Cu/In2.Cu/B.Cu |
| PADS-only pad proof | Circle pad on L1 and L4, no inferred inner copper pads |
| **NOT PROVED** | KiCad per-layer conductive pads; full trace/pad/Net parity; manufactured barrel; electrical contact; material, z, SI |

1 mil is exactly 25,400 nm. No floating-point tolerance is allowed. Both source files are independently parsed and each checked against this static witness. Source identities stay distinct.

Fixture bytes are pinned to LF using scoped `.gitattributes` rules. This is essential on Windows checkout (`core.autocrlf`), where unpinned raw text was converted into CRLF and failed immutable SHA checks. Changes to bytes must be independently re-pinned, not silently normalized by the oracle.

Original fixture files:
- crates/boardsi-import/tests/fixtures/e028-via-pads-v94.asc
- crates/boardsi-import/tests/fixtures/e028-via-kicad9.kicad_pcb

Pinned original byte SHA-256 (not Git blob SHA-1):
- PADS: dda44f2b647c8c715c2fe2a3300fed8f0fd34ce7fa683e3ace0e9d4382cc6fba
- KiCad: 3ea780fea60bb02c91be5dcf91ecc1609cae6c15758b31e6201d8c609bf902f9

## Implementation

The opt-in e028_cross_format_oracle module calls the existing PADS generalized parser and **actual Canonical BoardModel conversion**, and the KiCad 9 Source Inventory reader. It checks exact PADS frame, physical copper layer count/order, canonical Via/Drill, source-declared end Padstack geometry and source SHA-256; KiCad inventory layer order, Via kind/envelope, physical centre and dimensions are checked independently. Reject partial/unsupported scope, missing or contradictory evidence, and invalid source. No existing Source Adapter is widened.

The KiCad inventory does not expose standalone traces, pads or canonical BoardModel, so this is intentionally **one single via common-subset qualification**. It cannot demonstrate full Canonical BoardModel geometry equivalence. KiCad envelope is **not a conductive pad/contact set**. Return fields for KiCad layer-pad contacts, electrical connectivity and solver readiness remain false.

## Negative tests

Run: cargo test -p boardsi-import --test e028_cross_format_via_oracle

- Fixed SHA and exact same geometry: bounded via equivalence, but no electrical proof.
- Changed raw PADS/KiCad Source SHA: reject without deliberate repin.
- Re-pinned PADS changed via coordinates, pad diameter, drill or added inner pad: reject.
- Re-pinned KiCad changed via centre, size or drill: reject.
- Re-pinned partial Through span, unsupported Layer/atom, unsupported remove-unused flag: reject.
- Truncation with new valid SHA: reject.
- Unchanged physical geometry but altered PADS net label with deliberate SHA repin: geometric match only, not Net equivalence.
- Unsupported oracle (e.g. Blind): reject.

## Evidence and acceptance boundary

Before concluding SOURCE-LIMITED QUALIFIED, require real final HEAD Debug/Release test execution, all required PR CI checks, independent reviewer outcome with 0 unresolved threads, protected Verified Squash merge and exact merged-main Push CI.

E_023 / EB-01 actual independent Vendor Source and same-revision CAD Oracle remain BLOCKED. Parent #532/#520 and full product U/T remain OPEN / NOT_ACCEPTED. No invented PAD contacts, electrical Graph or SI PASS.

## Final observed code PR and exact main Push evidence (2026-11-15 KST)

- Code [PR #641](https://github.com/july-rust/BoardSI/pull/641) — **MERGED**.
- Final reviewed/tested PR HEAD: `1fd050631ede26358e132d37fd44e96870b4fc41`.
- PR-head **30/30 GitHub Checks SUCCESS**, including all Numerical Core and Qt native Windows x64, Windows ARM64 and macOS ARM64; **9/9 pull_request workflow runs SUCCESS**.
- Independent Codex review on the **exact final HEAD** completed with **0 findings, 0 unresolved findings, 0 unresolved review threads**.
- Protected Squash commit [`d882d8f7f73a78e6f3f24c00950e6d5df73fc191`](https://github.com/july-rust/BoardSI/commit/d882d8f7f73a78e6f3f24c00950e6d5df73fc191). GitHub commit REST `commit.verification.verified=true`, reason `valid`, **one parent** `557a9c685f468768ae27a1da81a58beda1a9351c`. Exact `main` ref points to this Squash.
- Accurate original failure history: initial HEAD `0d0129a9` had `cargo fmt --check` FAIL in Numerical Core (log under [run #43654522308](https://github.com/july-rust/BoardSI/actions/runs/43654522308)); next HEAD `f74a8f88` had Native Windows x64 FAIL because `include_bytes!` saw CRLF instead of the pinned LF fixture bytes ([run #43654685289](https://github.com/july-rust/BoardSI/actions/runs/43654685289)). Both defects were corrected; historical FAIL receipts are **not** retroactively PASS.
- Exact code-main `d882d8f7f73a78e6f3f24c00950e6d5df73fc191`, event=`push`: **8/8 independent workflow runs completed SUCCESS and 28/28 check runs SUCCESS**. Numerical [#43656614970](https://github.com/july-rust/BoardSI/actions/runs/43656614970), Qt [#43656615003](https://github.com/july-rust/BoardSI/actions/runs/43656615003), docs [#43656615009](https://github.com/july-rust/BoardSI/actions/runs/43656615009), PH-06 [#43656615124](https://github.com/july-rust/BoardSI/actions/runs/43656615124), PH-07 [#43656615032](https://github.com/july-rust/BoardSI/actions/runs/43656615032), native FEM [#43656615049](https://github.com/july-rust/BoardSI/actions/runs/43656615049), synthetic G0 [#43656615015](https://github.com/july-rust/BoardSI/actions/runs/43656615015), isolated FEM [#43656614997](https://github.com/july-rust/BoardSI/actions/runs/43656614997). Qt Push run independently completed its Windows ARM64 native job successfully; this result is not inferred from PR-head CI.
- The connected container has no Rust executable; native Debug/Release evidence is from real GitHub Actions runners and must not be misreported as local tests.
- This final validation receipt, sole authoritative development checklist and E-stage closure-plan update are delivered together in docs-only [PR #642](https://github.com/july-rust/BoardSI/pull/642). The documentation PR must separately pass its final-HEAD required CI, independent code review and protected Verified Squash; final exact docs-main Push status is verified externally, not assumed from this text.

**Evidence gate passed for this bounded SOURCE subset only.** Historical failures remain visible. Any separately failed, canceled, skipped-required or unobserved broader acceptance case stays NOT_VERIFIED / BLOCKED; **never** promote to full-board Canonical equivalence, manufactured copper or SI based on this qualification.
