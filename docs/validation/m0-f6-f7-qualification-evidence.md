# BoardSI M0 F6/F7 evidence inventory — 2026-10-19 KST

**Status: F6 BLOCKED / F7 BLOCKED.** This is a reproducible evidence inventory and a safe candidate shortlist, not a redistribution approval, PC baseline qualification, installed prototype test, physics certification, or completed M0 gate. No confidential PCB data is copied into this repository.

## F6 — candidate public reference PCB designs

Official KiCad source mirror (public):
- Repository: https://github.com/KiCad/kicad-source-mirror
- Inspected upstream immutable commit: `181d683185b20890085b0e8f1e0e89fbda63a56b`
- Design-format boundary: these are **KiCad** board sources; BoardSI M0 does **not** have a KiCad/ODB++/PADS importer. These files are proposed future independent CAD-format fixtures, not successful BoardSI imports.

| Candidate | Source (pinned link) | Source metadata | Stated license | Current qualification |
| --- | --- | --- | --- | --- |
| Small-form-factor candidate, RoyalBlue54L Feather | [board](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/RoyalBlue54L-Feather.kicad_pcb) / [README](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/README.md) / [license](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/royalblue54L_feather/LICENSE) | Upstream board Git blob SHA-1 `5d47510c87e3d4c09bfd18d91dec003a718a6ca6`, size 3,549,654 bytes; independently calculated SHA-256 of pinned GitHub text response `535dbe212f43a8bab12b23277e176a94ac9694177258af2981c0e7b08e4b7252` (UTF-8 byte count matched 3,549,654) | README states CERN-OHL-V2-P; accompanying LICENSE says CERN Open Hardware Licence 2 Permissive | **CANDIDATE ONLY**; board physical dimensions, independently reproduced CI byte-level SHA-256, third-party notices and redistribution bundle not yet independently verified |
| More complex candidate, TinyTapeout demo | [board](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/tiny_tapeout/tinytapeout-demo.kicad_pcb) / [license](https://github.com/KiCad/kicad-source-mirror/blob/181d683185b20890085b0e8f1e0e89fbda63a56b/demos/tiny_tapeout/LICENSE.txt) | Upstream board Git blob SHA-1 `a8465878d46656c36617870f511e62c979298854`, size 4,541,774 bytes; independently calculated SHA-256 of pinned GitHub text response `0d26bf9b6d561e32cf610d428541d6433e60b45e1838f34f73467e25b9144814` (UTF-8 byte count matched 4,541,774) | The project directory contains Apache License 2.0 text | **CANDIDATE ONLY**; precise physical dimensions, independently reproduced CI byte-level SHA-256, provenance of every included third-party asset and redistribution notice still require checking |

These are **not certified small/medium benchmark boards** yet. File length is not an acceptable proxy for physical PCB size or complexity. Do not assume the two existing KiCad sources cover M1 PADS/ODB++ syntax. The Git blob SHA-1 identifies a Git object but **is not a raw-file SHA-256**. The SHA-256 values displayed here were calculated using a self-tested SHA-256 implementation on the GitHub connector's UTF-8 text response from the immutable source commit (the byte lengths exactly matched upstream metadata); **an independent native GitHub Actions byte-for-byte recomputation is still required** before design fixture sign-off.

### Remaining F6 actions

1. Use a reproducible GitHub Action at the immutable source commit to fetch/read the two boards; independently recompute the SHA-256 values above and verify Git blob identity and byte counts. Reject any mismatch.
2. Extract actual Edge.Cuts extents, layer count, net count and geometry complexity; classify small/medium only on actual board evidence. Substitute a suitably licensed candidate if necessary.
3. Inspect license/third-party notices and decide whether files may actually be redistributed in BoardSI's proposed release; retain notices as required. Merely hosting the upstream source in a public mirror is insufficient.
4. Establish a real approximately 8 GiB CPU-only baseline host and record OS, CPU, installed RAM, available RAM, SSD, Qt/Rust build versions, reproducible memory/latency results. Do not treat an unspecified GitHub runner as the baseline PC.
5. Keep M1 import and numerical/physical PCB acceptance **NOT RUN**. F6 stays BLOCKED until verified.

## F7 — native executable toolchains and Qt license/deployment boundary

**Observed source/CI candidate stack:**
- `.github/workflows/qt-runtime.yml`: Qt **6.8.3**, `aqtinstall==3.3.0`, Linux `cmake==3.31.6`, Rust Worker **1.75.0**, host-matched native Windows x64, Windows ARM64 and macOS ARM64; Linux native Qt offscreen + Xcb verification is additional evidence.
- `apps/boardsi-qt-prototype/CMakeLists.txt`: Qt 6.5 minimum, actual Qt 6.8.3 CI, C++17, Qt6 Core/Gui/Widgets only; separate Rust Worker. These are prototype pins, not an installer/deployment certification.
- [Qt open-source licensing FAQ](https://www.qt.io/faq/qt-open-source-licensing) and [Qt LGPL obligations](https://www.qt.io/development/open-source-lgpl-obligations): check the actual shipped modules/dependencies individually; Qt LGPLv3 use often allows closed-source applications subject to compliance, generally using replaceable dynamically linked Qt modules. It requires recipient notices/license text and access to corresponding Qt source (or compliant offer), and the ability to replace/relink the LGPL library. Static linkage or restricted installation may need additional obligations/commercial licensing. This is **not legal clearance**.

### Remaining F7 actions

1. Verify the exact per-module/per-third-party license manifest and Qt binary linkage, including Qt plugins and runtime dependencies for all three target architectures.
2. Specify/prove an M0-level redistributable prototype bundle per OS/arch containing correct Qt runtime, platform plugins, Rust Worker, license/notices, and a source/relink compliance strategy.
3. Execute a clean environment smoke test without preinstalled Qt SDK/developer tools, validating platform library loading, signature/architecture, IPC, GUI start and process-exit behavior.
4. Use only lawfully redistributable dependencies. Explicitly distinguish prototype deployment proof from a later commercial installer and product release.
5. If no such clean environment or legal verification is available, report F7 BLOCKED instead of PASS.

## Independent closeout guardrails

- F4 restricted synthetic model→snapshot→Worker→Qt proof was Squash-merged as PR #102 into GitHub Verified `main@6ccaa840ceb3da02913b6e83162716400b6e4e95` with 17/17 successful **post-merge** required check runs. F4 does not imply F6/F7 or F8.
- F8 Issue #46 retains historical native Qt >1,000 ms failures. **As of the explicit user decision on 2026-10-20**, provisional **M0 whole-runtime global Qt timer gap <=1,500 ms** is the new acceptance bound; historical failures retain their original status, and all independent UI/IPC/Cancel and product-specification timings stay unchanged. See [M0 F8 provisional policy](m0-f8-provisional-1500ms-policy.md). Fresh exact-head CI remains required; cold-start root cause is deferred to release stabilization.
- No broad PH-08 or M1+ research and no change to Draft PR #35. Original PH-08 Gate Q remains FAIL / BOUND_NOT_CERTIFIED.
