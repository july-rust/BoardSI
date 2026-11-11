# E_006 — Three Native Platform Toolchain & Runtime Evidence Qualification

**Audit date:** 2026-11-11 KST. **Tracking:** [Issue #6](https://github.com/july-rust/BoardSI/issues/6).  
**Scope:** existing M0 *synthetic* Rust/Qt Widgets/QProcess native build and runtime **evidence freshness audit**, not a new feature or full M0/product sign-off.  
**Audited latest main:** [`400fe8e1261fa6eb1297ef757732c32f7815e981`](https://github.com/july-rust/BoardSI/commit/400fe8e1261fa6eb1297ef757732c32f7815e981); **last native-executed code main:** [`c229eccdbbba41af5f4c4d622c7a1bb6421bdf02`](https://github.com/july-rust/BoardSI/commit/c229eccdbbba41af5f4c4d622c7a1bb6421bdf02) (2026-11-11 KST).

## Decision and source identity — never count a SKIPPED job as PASS

The [GitHub compare](https://github.com/july-rust/BoardSI/compare/c229eccdbbba41af5f4c4d622c7a1bb6421bdf02...400fe8e1261fa6eb1297ef757732c32f7815e981) returned **ahead by 1, exactly 5 documentation files changed**: `docs/README.md`, `docs/development-checklist.md`, `docs/e-stage-closure-plan.md`, `docs/validation/INDEX.md`, and `docs/validation/e005-worker-cancellation-stale-timeout-qualification.md`. No runtime source, workflow, Cargo lockfile, Qt build config, fixtures or executable tests changed between these two SHAs. This is an **exact compared source-content equivalence**, not a claim that two distinct commits have the same Git SHA.

For `main@400fe8e`, [Qt push run 42984874203](https://github.com/july-rust/BoardSI/actions/runs/42984874203) and [Numerical push run 42984874249](https://github.com/july-rust/BoardSI/actions/runs/42984874249) each ended with overall workflow `success`, but each **three native platform jobs was SKIPPED** by the docs-only classifier. These runs provide **no fresh native execution** for that SHA. Their skipped jobs are not platform successes.

The unchanged *executed code* at `main@c229eccd` independently completed:
- [Qt runtime push run **42982428188**](https://github.com/july-rust/BoardSI/actions/runs/42982428188): Windows x64, Windows ARM64, macOS ARM64 **each completed/success**, plus Linux auxiliary Qt success.
- [Numerical core push run **42982428141**](https://github.com/july-rust/BoardSI/actions/runs/42982428141): Windows x64, Windows ARM64, macOS ARM64 **each completed/success**, plus Linux numerical success.
- Existing E_005 [code-head PR #588](https://github.com/july-rust/BoardSI/pull/588) / [validation record](e005-worker-cancellation-stale-timeout-qualification.md): cancellation-acknowledged late-response exclusion and same-window retry are among this *synthetic* runtime's regression coverage; this audit adds no new fault-injection code.

## Per-platform job-level native execution evidence

| Target / identity | Qt native **job** on main code SHA `c229eccd` | Rust native **job** on same main code SHA | Boundaries |
| :--- | :--- | :--- | :--- |
| Windows x64 · `x86_64-pc-windows-msvc` · `RUNNER_ARCH=X64` | [130395747849](https://github.com/july-rust/BoardSI/actions/runs/42982428188/job/130395747849) **SUCCESS**, Windows backend, Qt 6.8.3; real QApplication/Worker/IPC regression | [130395762089](https://github.com/july-rust/BoardSI/actions/runs/42982428141/job/130395762089) **SUCCESS**, native Rust | Not a clean-host installation or OS-version matrix |
| Windows ARM64 · `aarch64-pc-windows-msvc` · `RUNNER_ARCH=ARM64` | [130395747821](https://github.com/july-rust/BoardSI/actions/runs/42982428188/job/130395747821) **SUCCESS**, Windows ARM64 backend, Qt 6.8.3; actual ARM64 Worker | [130395762020](https://github.com/july-rust/BoardSI/actions/runs/42982428141/job/130395762020) **SUCCESS**, native Rust | Host x64 Qt tool availability **does not** replace native ARM64 test execution |
| macOS Apple Silicon · `aarch64-apple-darwin` · `RUNNER_ARCH=ARM64` | [130395747795](https://github.com/july-rust/BoardSI/actions/runs/42982428188/job/130395747795) **SUCCESS**, Cocoa backend, Qt 6.8.3; real ARM64 Worker | [130395762063](https://github.com/july-rust/BoardSI/actions/runs/42982428141/job/130395762063) **SUCCESS**, native Rust | Not notarized, packaged product or clean-host acceptance |

**Actual inspected job logs:** all six job logs were fetched, not inferred from workflow labels. The Qt logs record the intended `BOARDSI_CI_TARGET`, `BOARDSI_CI_RUNNER_ARCH`, `BOARDSI_EXPECTED_WORKER_ARCH`, `RUSTUP_TOOLCHAIN=1.75.0-<target>`, Qt 6.8.3 SDK, a successful native Qt backend and `Native Qt metrics PASS is insufficient if following IPC selftest fails. ... ok` at each target. The Rust jobs independently report successful native builds/tests. The Qt logs include synthetic M0 F1 subcase evidence **37 PASS** each; this is **not** 37 formal product T-test acceptances.

**Observed toolchain boundary:** Rust **1.75.0** (target-specific native hosts), Qt **6.8.3** (Widgets, Core/Gui, QProcess, C++17); Windows MSVC/CMake and macOS AppleClang/CMake are runner-provisioned versions. No claim of a single immutable compiler/OS image, complete OS-version matrix, release toolchain selection or physical GUI display qualification. Actual lockfiles and CI steps take precedence over old historical version tables.

## Additional F7 *relocation* evidence — strictly separate source identity

The [additional pull_request-triggered F7 relocation run **42980487868**](https://github.com/july-rust/BoardSI/actions/runs/42980487868) is **associated** with PR #588 HEAD `a4c935ed74e4ce60fe1f4678a4dbbdb4a6f0175b`, **but did not directly check out that commit**. All three actual job logs show `actions/checkout` fetched `refs/remotes/pull/588/merge` and `git log -1` reported **executed synthetic merge SHA `3ddd02b529d89945de1fc682cf0e730fde0e05ab`** (merge of `a4c935ed` into `1ceaef92`). Independently fetched Git **commit tree identities** for the [executed merge SHA](https://github.com/july-rust/BoardSI/commit/3ddd02b529d89945de1fc682cf0e730fde0e05ab), [PR HEAD](https://github.com/july-rust/BoardSI/commit/a4c935ed74e4ce60fe1f4678a4dbbdb4a6f0175b), and [protected Squash code-main](https://github.com/july-rust/BoardSI/commit/c229eccdbbba41af5f4c4d622c7a1bb6421bdf02): **all three have the same exact Git tree SHA `b7b591ae7a72286387896d842424bcdb61a4148e`**. Therefore the F7 run executed a **different commit with an identical complete checked-out source tree**, not the literal PR-head SHA. Equality was verified through Git trees, not assumed from run metadata. Three independently successful native jobs and artifacts:

| Target | Job | SDK-detached restricted synthetic selftest | Artifact |
| :--- | :--- | :--- | :--- |
| Windows x64 | [130389575153](https://github.com/july-rust/BoardSI/actions/runs/42980487868/job/130389575153) | **PASS**; process exit 0 | [13837915616](https://github.com/july-rust/BoardSI/actions/runs/42980487868/artifacts/13837915616) |
| Windows ARM64 | [130389575228](https://github.com/july-rust/BoardSI/actions/runs/42980487868/job/130389575228) | **PASS**; process exit 0 | [13837701419](https://github.com/july-rust/BoardSI/actions/runs/42980487868/artifacts/13837701419) |
| macOS ARM64 | [130389575175](https://github.com/july-rust/BoardSI/actions/runs/42980487868/job/130389575175) | **PASS**; process exit 0, Mach-O dependencies inspected | [13837072826](https://github.com/july-rust/BoardSI/actions/runs/42980487868/artifacts/13837072826) |

The three actual job logs and their **real checkout commit** were inspected. The underlying workflow [`m0-f7-relocation.yml`](../../.github/workflows/m0-f7-relocation.yml) creates a relocated prototype bundle and detaches the Qt *build* SDK before executing the selftest. This proves neither clean-OS deployment nor legal redistribution clearance. **F7 executed synthetic merge `3ddd02b5` (tree identical to `a4c935ed` and `c229eccd`); Qt/Rust *push* workflows executed code-main `c229eccd`.** Event metadata, checked-out commit and proven source tree are separate facts. For full F7, see [F6/F7 evidence](m0-f6-f7-qualification-evidence.md).

## Negative and fail-closed audit matrix

| Invariant | Existing enforcement / observed status | New coverage? |
| :--- | :--- | :--- |
| Cross-compile-only is not native execution | Each native Qt and Rust target has a separate actual `completed/success` job, not a cross-compile inference | **No**: audit only |
| Wrong runner architecture or Worker host | Qt workflow expectations and F7 `RUNNER_ARCH` identity checks; verified native job logs | **No**: existing |
| Qt build without working GUI/IPC | Qt job's real Native GUI/Worker/IPC selftest required beyond Qt timing metrics | **No**: existing |
| Missing Qt plugin/Worker in relocated bundle | F7 3-platform relocated selftest plus fail-closed manifest unit tests, SDK detached | **No**: existing |
| SKIPPED/CANCELLED/FAILED equals PASS | Explicitly disallowed; `main@400fe8e` three Qt + three Rust native jobs are **SKIPPED**, not PASS | **No**: audit contract |
| Stale/cancelled responses, IPC mismatch | E_003 and E_005 native synthetic Rust/Qt regressions covered by executed Qt jobs | **No**: keep original tests |
| Historical macOS Qt stall | #46 retains its historical FAIL; owner-approved M0 provisional 1,500 ms whole-runtime exception is not a root-cause fix | **No**: E_029 separate |

**No new code or test is required for this bounded audit:** existing current-code evidence plus exact source-identity comparison cover the requested native prototype evidence. Do not claim that this audit itself launched *new* runners or reran the physics solver.

## Disposition

- **E_006 bounded audit:** *EVIDENCE_QUALIFIED* on 2026-11-11 KST for all three native **synthetic prototype** targets. Administrative completion additionally requires the E_006 documentation PR's checks/review/protected Squash and the exact post-merge main push verification; record that final gate in the Issue #6 completion comment.
- **Issue #6 overall:** **OPEN / PARTIAL**. It remains incomplete for clean installation, legal runtime/third-party license and distribution verification, baseline PC/OS-version coverage and full issue-level U03/U04/U14, T13/T39 acceptance.
- **M0 F5:** actual native prototype execution evidence PASS in its restricted scope; **M0 overall BLOCKED** (F1/F6/F7), [M0 completion gate](m0-completion-gate.md). No formal product claim.
- **F7:** relocation synthetic smoke PASS at **PR synthetic merge checkout `3ddd02b5`** with a verified Git tree identical to PR HEAD `a4c935ed` and code-main `c229eccd`; clean installation and redistribution compliance **BLOCKED / NOT_VERIFIED** (see [blocked register](../e-stage-blocked-register.md) EB-05).
- **F8:** #46 unresolved cause and historical FAIL preserved; full release/performance acceptance NOT_VERIFIED.
- **Products / physics:** real-board import, commercial deployment, full PCB Solver and PH-08 original Gate Q **not changed or certified**.

**Closure rule:** A successful docs-only PR can legitimately skip native runners, but it **must not** be cited as a new three-platform execution. The executed `main@c229eccd` plus proof of byte-identical runtime sources through audited `main@400fe8e` is the technical evidence; the new E_006 docs PR itself will require its own CI/review/Squash/push validation.
