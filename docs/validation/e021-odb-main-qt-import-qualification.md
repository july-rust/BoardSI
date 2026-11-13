# E_021 — Main Qt ODB++ nonblocking Canonical artwork qualification

**Evidence date:** 2026-11-13 KST. **Scope:** bounded, public synthetic ODB++Design SOURCE directory/one STEP, positive straight Trace and round/rectangle Pad; actual main Qt artwork, picking, cancel and re-import. **Product acceptance:** NOT_VERIFIED. This is *not* an ODB++ vendor interchange, physical electrical graph, manufactured via or SI solver acceptance.

## Actual code and CI evidence

- [Code PR #620](https://github.com/july-rust/BoardSI/pull/620) merged by GitHub **squash** as [`a423b1ba8a68726eb57ea013c5a0e8f9bbf46cef`](https://github.com/july-rust/BoardSI/commit/a423b1ba8a68726eb57ea013c5a0e8f9bbf46cef). Source PR final HEAD: `5843c2c87091af96c2a4f0e140b7f27fd03768ca` (16 PR commits; 9 changed files).
- Exact final PR HEAD triggered **11 pull_request workflow runs, 11 SUCCESS**; [Qt runtime run #43412000983](https://github.com/july-rust/BoardSI/actions/runs/43412000983) includes successful Linux x64, Windows x64, Windows ARM64 and macOS ARM64 Qt native jobs. The results are offscreen Qt software evidence, **not actual hardware GPU pixels/performance**.
- Independent Codex reviews at older HEADs `8d0c93ad07` and `58f8108c8b` identified two P2 bugs: concurrent Canonical-vs-ODB imports and missing Windows staged ODB CLI. Both corrected; GitHub review threads 2/2 resolved. Final HEAD differs from last reviewed HEAD by only one added Windows packaging entry. A fresh Codex final-HEAD review was **requested** at PR #620 but **was not yet evidenced** at this reconciliation. Do not invent a positive final-head review receipt.
- GitHub API merge response confirmed `merged=true` and squash commit SHA. The available connected commit API did **not expose commit cryptographic verification** and its workflow-run lookup returns pull_request runs only. Therefore exact merged-code-`main` `event=push` **8/8** and `verification.verified=true` remain **NOT_VERIFIED** in this report, not PASS or FAIL. PR CI must not be substituted for code-main Push.

## Exact implementation boundary

The newly isolated Rust `boardsi-odb-qt-import` calls the existing `boardsi_import::odb_source::import_directory` for one selected STEP. It checks the Canonical Board Model and emits a separate bounded `BOARDSI_ODB_QT_V1` text wire with source logical paths and individual SHA-256, layer identities, exact Rational geometry and declared net ownership (if available). Unsupported Arc, filled Area/Cutout, via/manufacturing drill, component/pin and arbitrary pad orientation/shape **fail closed** rather than being silently dropped. This is a strict **2–64 copper-layer, up to 64 source-file and up to 256 Trace/Pad shape** preview, not complete ODB++ Import or broad transport equivalence.

Actual `PreviewImportPanel` uses a dedicated `QProcess` and bounded timeout/buffer, strict Qt file-manifest verification, cancellation and stale-result rejection. `ImportedArtworkPanel` displays accepted Trace/Pad footprints as source-design geometry with Layer controls and object picking. Existing artwork is reset at the start of an import attempt; a failed import does not retain the prior displayed board. That existing contract is **not** the stronger transactional rollback / preserved-last-accepted-board behavior; do not claim that guarantee. SOURCE file mutations between Rust import and Qt manifest check are rejected, though hostile concurrent filesystem mutation/TOCTOU is not certified.

CPU software drawing is deliberately used for this bounded Trace/Pad preview; the existing Via RHI mesh is not assumed to render these new shapes. Layer identity and Source SHA are retained; electrical connectivity, plating and SI are UNKNOWN/NOT_RUN. Only Directory SOURCE and selected STEP are accepted by this new Qt entrypoint; ZIP/USTAR/GZIP Qt UI acceptance is outside E_021.

## Regression matrix and acceptance limits

| Area | Evidence | Scoped status |
|---|---|---|
| Main Qt ODB directory/selected STEP into existing Canonical | PR #620 native Qt tests | PR-head PASS |
| SOURCE manifest SHA, geometry/layers, main scene Picking | `boardsi-e021-main-odb-test` | PR-head PASS |
| Layer visibility, reimport with modified SOURCE, invalid input | Same native Qt test | PR-head PASS |
| QProcess cancel and stale/discard behavior | Same native Qt test | PR-head PASS, bounded synthetic |
| Unknown IPC version/layer ID/manifest mismatch | Strict decoder test | PR-head PASS, bounded synthetic |
| Existing PADS + Qt regression | Qt runtime CI | PR-head PASS |
| Windows x64/ARM64, macOS ARM64, Linux x64 native software tests | Qt #43412000983 | PR-head PASS |
| Last successfully accepted board retained after failing import | No such behavior implemented | NOT_IMPLEMENTED, explicit narrower rollback contract |
| Independent commercial ODB++, archive UI, negative/area Copper, Step Repeat | No independent qualification | NOT_VERIFIED |
| Verified merge signature / exact code-main Push CI | Not retrievable from available connected tools | NOT_VERIFIED |
| Final HEAD independent Codex review receipt | Request posted, no final receipt confirmed | NOT_VERIFIED |
| Full Issue #10/#11, formal U/T, E_022 electrical graph, E_023 Vendor Oracle, E_024 product gate | Separate milestones | OPEN/NOT_VERIFIED |

## Closeout decision

Code PR #620 is **MERGED**, all observed PR-head CI jobs passed and the two observed P2 findings were addressed. E_021 is **IMPLEMENTED, post-merge qualified completion PENDING** until the exact signed squash verification and independent code-main Push CI are established, with final HEAD review evidence accounted for. The stronger accepted-board rollback behavior and comprehensive ODB feature coverage are **not** claimed. Keep #10 and #11 OPEN; historical #578 remains CLOSED. The next E_022 scope remains the independently source-proven LOCAL electrical graph, not an inference from Qt artwork.
