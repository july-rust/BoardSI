# BoardSI — delivery-first operating policy and Preview plan

**Owner decision:** 2026-10-21 KST. **Objective:** make the earliest actually runnable BoardSI developer Preview, then expand by demonstrable end-to-end user features. This is a scheduling/delivery policy, **not** a change to the accepted functionality, minimum test thresholds or exit criteria in [Development Specification v1.0](specification/BoardSI_Development_Spec_v1.0.md).

**Tracking issue:** [Preview P0 #152](https://github.com/july-rust/BoardSI/issues/152). **Baseline reviewed at decision:** protected `main@94c66d581353b63236271507aa67ab87707f19f0` (2026-10-21 KST). Confirm live branch/CI SHA before relying on this historical checkpoint.

## A. Non-negotiable development rules

1. **Working program before additional research.** Reuse the existing Qt Widgets/C++17 GUI, `QProcess`, independent Rust Worker, immutable synthetic Canonical Model snapshot/IPC and platform tests. Do not redesign the framework without a demonstrated incompatibility.
2. **One narrow user-visible outcome per feature PR.** Confirm a reproducible runnable executable, native UI or functional demonstration. Compiler success, GitHub Verified, 17/17 CI, a PR count and documents alone do *not* prove product functionality.
3. **Two separate status lanes:** (i) formal M0 certification/acceptance; (ii) Preview engineering delivery. Technical prototypes can be tested and demonstrated while M0 is BLOCKED, but they are **NOT** M0 COMPLETE, **NOT** production deployment, and **NOT** release authorized. Preserve `M0 BLOCKED` until its original approved gates pass.
4. **No scope explosion:** hold additional PH-08 HMM numerical research, new FEM models, unneeded protocol redesign, GPU, multiformat importing, progressive sweep and packaging polish until required by the next Preview acceptance criterion. Preserve PH-08 failure/evidence unchanged.
5. **Existing protection never relaxed:** Ruleset `25046595`, 17 mandatory CI contexts, strict update, signed GitHub Verified commits, squash-only and resolved review threads; never force-push or bypass `main`, disable a required check, fabricate a PASS, request a private signing/SSH key or relabel an old FAIL.
6. **Protect IP and users:** no proprietary PCB artwork, stackup, net names, measured Touchstone, private CAD files or credentials in public repository, comments, logs, Actions or artifacts. Use only public fixtures with reviewed rights or intentionally synthetic models. Keep the application offline-first.
7. **Physical integrity first:** explicitly label synthetic, computed-from-input, unsupported, not-run and independently validated results. A scalar supplied RLGC→2-port S calculation does not establish real PCB geometry-to-RLGC accuracy. Never emit a physical `Validated` label without the appropriate independent model/measurement agreement.
8. **No resource surprises:** use supported free GitHub resources where possible; obtain owner approval before paid VM/cloud services. Do not demand end-user personal signing material.

## B. Delivery track — executable slices, not fake phase completion

| Slice | Actual deliverable | Minimum acceptance | Explicitly excluded |
| --- | --- | --- | --- |
| **P0 — executable first** ([#152](https://github.com/july-rust/BoardSI/issues/152)) | Native desktop Qt developer Preview + separate Rust Worker, synthetic A/B model fixtures, progress/Cancel/error/status and reproducible build/use instructions | Evidence of *actual executable launch and functioning GUI* on Windows x64 first; unchanged native Windows ARM64 + macOS ARM64 regression; clear SYNTHETIC / NOT PCB SI status. A legally distributable binary is not presumed | New solver geometry, FEM/HMM, broad CAD importer, claiming install/release |
| **P1 — artwork** | Render synthetic Canonical Model copper/pads/traces and support layer/net selection, pan/zoom; same canonical source for future solver | Deterministic screenshot/geometry IDs and interactive tests; unknown connectivity labeled unknown | Real CAD fidelity claim |
| **P2 — PADS ASC** | One explicitly declared supported PADS ASC revision and input-object subset → Canonical Model → Viewer | File signature/revision detection; exact coordinate/layer/net/trace/pad/via comparison to lawful fixture; reject unsupported/invalid structures, never silently guess | ODB++/IPC-2581/all revisions/full M1 acceptance |
| **P3 — limited SI** | Validated uniform transmission-line input → independent Rust Worker → scalar RLGC / 2-port complex S over specified support range | Cross-checked independent analytical/ABCD baseline and documented assumptions; file geometry-to-RLGC marked NOT VALIDATED until its own physical proof | Generic traces-with-vias, differential full matrices, 30 GHz physical qualification |
| **P4 — testable Preview package** | GUI result visualization and agreed minimal Touchstone output, platform-specific developer package | Native execution on target architectures, format round-trip, legal notices, clean OS/package checks and all applicable required gates | Public product / 1.0 unless separately accepted |

P0's earliest executable **may be a restricted developer artifact**, not a public package. If Qt/third-party licensing or clean OS proof is incomplete, mark `DISTRIBUTION BLOCKED` and show native-run evidence instead of distributing unauthorized binaries. A fully installable binary requires its own F7/licensing clearance.

The full specification still requires PADS+ODB++ for M1 and other M0-M2b gates for initial product acceptance. **Preview slices do not replace or silently weaken M0/M1/M2 or T01–T54.** Existing synthetic proofs `PH-03/05/06/07` have strictly limited scopes.

## C. Formal M0 closeout track, in parallel and accurately reported

The owner-authorized delivery-first decision **supersedes only the previous blanket M0-only scheduling freeze**, permitting independent P0+ Preview work; it does **not** waive M0 ``14.2/15 exits or associated U/T proofs. Official [M0 completion gate](validation/m0-completion-gate.md) remains authoritative; historical audit documents must not be silently rewritten.

- **F1:** reconcile individual applicable M0 technology-case evidence, fixtures, native exact-commit job IDs and independent acceptance; do not turn CI or synthetic subcase PASS into full T-case acceptance.
- **F6:** preserve verified public KiCad native-board geometry proof and 8 GiB/2 CPU Linux constrained synthetic demonstration. A qualified representative approximately 8 GiB reference host, documented RAM/CPU/SSD/OS and complete sample redistribution rights/notices remain external/formal blockers.
- **F7:** preserve tested Qt/Worker native relocation and SDK-free Linux proof. Clean Windows x64/ARM64 and macOS ARM64 installation/deployment, Qt LGPLv3 source/replacement notices, third-party rights and independent legal compliance review remain separate blockers.
- **F8:** owner-approved provisional 1,500 ms whole-test Qt global callback-gap criterion for M0 only; preserve unchanged user-facing timing/Cancel/Heartbeat/IPC limits and recorded 2,930 ms macOS FAIL. Root-cause investigation remains tracked in [Issue #46](https://github.com/july-rust/BoardSI/issues/46).
- **PH-08:** original Gate Q FAIL, AC-P1 pending and physical error bound NOT CERTIFIED; no "physics complete" claims. Leave Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) untouched.

A paid/physical reference host, license or independent owner review that cannot be satisfied in GitHub is reported as `EXTERNAL DEPENDENCY — NOT PERFORMED`, not repeatedly attacked with unchanged CI runs. No automatic lowering of formal M0 exit requirements.

## D. PR/CI execution and efficiency protocol

1. Read protected `main` current exact SHA, matching code and existing overlapping PRs *before* editing. Particularly check open PADS signature/Open PCB shell PRs [#115](https://github.com/july-rust/BoardSI/pull/115), [#117](https://github.com/july-rust/BoardSI/pull/117), [#118](https://github.com/july-rust/BoardSI/pull/118). Reuse reviewed code; do not blindly merge stacked outdated work or duplicate it.
2. Define the **one end-to-end observable outcome and negative cases** on a small Issue; implement one coherent feature branch and PR. Avoid creating new Issue/PR/CI layers solely to repeat the same evidence.
3. Perform fast local/targeted relevant tests early, then retain ALL existing mandatory 17/17 exact-head GitHub checks and required independent code review for protected promotion. Inspect raw logs, binaries and native results; avoid unnecessary non-mandatory duplicate matrix jobs where safe.
4. Use normal GitHub signing; investigate a simpler signed promotion route only after confirming its compatibility with Ruleset `25046595`. Until proven, retain compliant staging. Never pretend an unsigned source branch has been merged into protected main.
5. If identical failure appears **twice without a material change**, stop blind retries: classify root cause `CODE / TEST HARNESS / HOST / EXTERNAL`, preserve first failure, select a new bounded remedy or report `BLOCKED`. This heuristic does not permit merging failed mandatory CI.
6. Limit concurrent work to **one executable feature slice** and **one M0 evidence/external prerequisite lane**. No overlapping development of future Preview slices until the current slice has runnable acceptance evidence or an explicit blocker.
7. Review historical CI/results before re-running; prevent review/signing from becoming a more expensive task than the feature under development, while retaining protection.

## E. User-facing report contract

Every completed attempt, including a blocked one, reports succinctly:

- **Feature and artifact**: GitHub Issue/PR, immutable source SHA, executable/download only if it actually exists and redistribution permits.
- **Observed run**: operating system/architecture, GUI launched yes/no, Worker interaction yes/no, failures and native log/artifact URL.
- **Acceptance**: `PASS / PARTIAL / FAIL / NOT_RUN / BLOCKED` for the narrow slice, distinguished from formal M0 and full user T-cases.
- **Remaining blockers**: concrete next action, owner/external decision only if necessary; no invented percent completion, ETA or "ready for release" claim.
- **Next step**: exactly one user-visible feature or independent M0 prerequisite, based on the current priority.

**Next execution:** [Preview P0 #152](https://github.com/july-rust/BoardSI/issues/152), not more standalone PH-08 or redundant M0 synthetic qualification. Related [M0 Issues #2](https://github.com/july-rust/BoardSI/issues/2), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7) remain open until their evidence is actually accepted.
