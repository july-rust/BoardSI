# D_003 — bounded 3-COPCLS / 1-COPCUT source-geometry candidate graph

**First audit:** 2026-11-01 KST. **Tracker:** [Issue #339](https://github.com/july-rust/BoardSI/issues/339). **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Baseline protected main `cb9c6e213130cee807edbd3817be691ada7e06e7`.

## New case / no duplicated physical claims

- C_022: 3–6 same-layer, same-net rectangles **without** COPCUT; groups only.
- D_002: exactly two Source-owned COPCLS with zero or one owned rectangular COPCUT.
- **D_003:** exactly three complete Source-owned same-declared-Net/Layer rectangular COPCLS and at most one internally owned rectangular COPCUT, with pairwise retained positive-area candidate edges AB/AC/BC and transitive components. This reuses D_002 exact integer-tick pairwise cells; no float, edge-only or bbox-only false contact.
- Area Source Name/Canonical ID and outer/hole segment parity, original SHA-256, full Copper inventory and unfilled-pour rejection. Explicit bounds on wire/selection, strict errors for unsupported geometry.
- Opt-in Rust command: `boardsi-import-artwork <source.asc> --d003-three-area-copcut-v1 RA RB RC NET_REF 2`. Ordinary importer stdout/wire untouched.
- Qt `--d003-three-area-selftest` independently recomputes 3 pairs from actual imported geometry and ownership, checks each wire edge, component group/overall state, and rejects stale SHA after file change; warning-only risk markers.

### Public synthetic tests
- Rust `crates/boardsi-import/tests/d003_three_area_copcut.rs`: transitive AB/BC and no AC; cutout eliminates AB; three separated/tangent rectangles; wrong Net/Layer, extra area, invalid shapes, more than one hole, source SHA mutation.
- Native Windows x64/ARM64, macOS ARM64 Qt dedicated smoke and Linux CTest run the real Rust executable/Qt importer and confirm markers removed after reimport.
- Actual execution evidence **PENDING**. A checked box, CI pass, independent review, protected merge, post-main verification and issue closure MUST NOT be pre-claimed by this document.

## Mandatory limits
These source-geometry graph edges are only candidates for a visual warning, **not manufactured or electrical conductor connectivity evidence**. Filled copper, thermal relief, padstack, split reference current, electrical Net, P009, PH-08 and production solver remain unverified. The geometric annulus-with-inner-rectangular-hole is connected as ideal geometry; this does not prove manufactured copper.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Full U09/U10/U39 and T02/T04/T24/T52 remain unchecked, parent #12 OPEN; Draft #35 unmerged. **No performance/30 GHz or physical correctness qualification.**

## Code PR acceptance — verified 2026-11-01 KST

Code [PR #340](https://github.com/july-rust/BoardSI/pull/340), exact latest HEAD `3e608c495b9685fedbef25d607a2839372759d69`: independently observed **27/27 Check Runs SUCCESS**, **11/11 `event=pull_request` Workflow Runs SUCCESS**. Rust 1.75 fmt/strict Clippy and Debug/Release, Native Rust Windows x64/ARM64 and macOS ARM64, Qt native Windows x64/ARM64 and macOS ARM64, and Linux Qt 6.8.3 offscreen D_003 selftest actually executed successfully. Earlier negative Source-tick expectation and formatting were repaired before this final HEAD; original P1 Codex thread resolved.

Independent Codex **exact latest HEAD** code review receipt `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, 0 unresolved review threads.

Protected [signed, GitHub Verified one-parent Squash main@7f43985124f4b422fc002dba8633aab06e8f7402](https://github.com/july-rust/BoardSI/commit/7f43985124f4b422fc002dba8633aab06e8f7402): GitHub REST `commit.verification.verified=true`, `reason=valid`, sole parent `cb9c6e213130cee807edbd3817be691ada7e06e7`. No bypass claimed. **Independent post-code-main exact-SHA verification PASSED 2026-11-01 KST**: [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/7f43985124f4b422fc002dba8633aab06e8f7402/checks) and **8/8 `event=push` Workflow Runs SUCCESS**, each explicitly bound to `7f43985124f4b422fc002dba8633aab06e8f7402`. This qualifies only code/source geometry D_003; separate docs-only reconciliation PR must pass its own final-head CI/Codex/protected Verified Squash/post-main before #339 closes.

Evidence: [PR HEAD Checks](https://github.com/july-rust/BoardSI/commit/3e608c495b9685fedbef25d607a2839372759d69/checks), [PR #340](https://github.com/july-rust/BoardSI/pull/340), [signed merge](https://github.com/july-rust/BoardSI/commit/7f43985124f4b422fc002dba8633aab06e8f7402).

## Pending final acceptance

- [x] Exact latest code PR HEAD and all actual checks/workflows SUCCESS.
- [x] Native Qt Windows x64/ARM64, macOS ARM64, Linux offscreen and Rust Debug/Release regression SUCCESS.
- [x] Independent exact HEAD Codex completed, 0 findings, 0 unresolved threads.
- [x] Protected one-parent GitHub Verified Squash.
- [x] Independent exact merged SHA post-main 21/21 Checks and 8/8 Push Workflows SUCCESS on 2026-11-01 KST.
- [x] **D_003 code-only** first checked/evidence date **2026-11-01 KST**, supported synthetic Source-geometry scope only.
- [ ] Issue #339 closeout awaits independent docs-only PR exact-HEAD CI/Codex, protected Verified Squash and post-main acceptance.

Original M1 #12 remains OPEN; no formal U/T, physical contact, P009, solver/cache or PH-08 qualification.
