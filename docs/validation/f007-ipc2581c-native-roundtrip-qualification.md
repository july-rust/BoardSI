# F_007 — IPC-2581C source Copper to Native .bsi to Main Qt

> **IMPLEMENTED ON FEATURE BRANCH / NOT YET ACCEPTED.** No completion claim before exact-HEAD CI, 4-OS runtime, independent review, protected Squash and main Push receipts.

## Bounded acceptance contract

The existing E_025 parser is a bounded source inventory, not an IPC-2581C full XSD validator. F_007 adds a stricter opt-in Native Canonical adapter.

Normative geometry reference:
- https://www.ipc2581.com/ipc-2581-revc-test-cases/
- https://webstds.ipc.org/2581/
- https://docs.rs/crate/ipc2581/0.3.3/source/IPC-2581C.xsd

Supported subset: CadHeader units MILLIMETER/MICRON/INCH, LayerFeature/Set/Features/Line or straight Polyline, inline LineDesc with ROUND lineEnd and SOLID lineProperty (or absent), positive Copper Layer and optional declared LogicalNet. Source decimal coordinates and widths are converted with checked integer arithmetic to exact nm; sub-nm, overflow, malformed and out-of-range inputs fail closed. Source SHA, Layer metadata, CAD unit and bounded PinRefs are preserved in SourceIdentity.

Unsupported Pad, Via, Drill, Arc, Polygon, Location, Xform, negative polarity, LineDescRef, manufacturing semantics and unrecognized CadData geometry are rejected rather than discarded. The source Layer declaration order is NOT verified physical z. Neither logical net names nor round-ended traces establish physical conductivity.

Existing SQLite v1 / BSIG Writer/Reader and guarded staging are reused. New Native CLI 'save-ipc2581' verifies the source SHA before parsing and before publish. The actual main Qt application adds a nonblocking import/save action and a source-only bounded artwork preview with layer filtering/picking.

**Outside F_007 scope**: full IPC XSD, external vendor fixture, all fabrication objects, electrical graph, confirmed plating, RLGC/S-parameters, manufacturing and product U/T acceptance.

## T01–T12 acceptance — awaiting actual run

| ID | Gate | State |
| --- | --- | --- |
| T01 | Revision C, malformed XML and exact subset | NOT_VERIFIED |
| T02 | Layer and Net refs | NOT_VERIFIED |
| T03 | straight Line / Polyline Copper | NOT_VERIFIED |
| T04 | precise coordinate, width and unit transforms | NOT_VERIFIED |
| T05 | Native Canonical semantic roundtrip | NOT_VERIFIED |
| T06 | F_004 v1 Golden compatibility | NOT_VERIFIED |
| T07 | independent SQL/BSIG/SHA check | NOT_VERIFIED |
| T08 | unsupported geometry and malicious XML rejection | NOT_VERIFIED |
| T09 | source SHA guard, cancel, crash, lock, ENOSPC | NOT_VERIFIED |
| T10 | real main Qt import, artwork, picking, save/open | NOT_VERIFIED |
| T11 | four native OS and Qt executions | NOT_VERIFIED |
| T12 | prior F_001–F_006 regression and post-merge main | NOT_VERIFIED |

## Reproduction

    cargo +1.75.0 fmt --all -- --check
    cargo +1.75.0 clippy --locked --workspace --all-targets -- -D warnings
    cargo +1.75.0 test -p boardsi-import --locked f007_native_tests
    cargo +1.75.0 test -p boardsi-native-file --locked --test f007_ipc2581c_roundtrip
    cargo +1.75.0 test --locked --workspace
    ctest --test-dir build-qt -R qt-f007-main-ipc-native-roundtrip --output-on-failure

Native/Qt four-OS workflows are in .github/workflows/f004-compatibility.yml and .github/workflows/qt-runtime.yml. Results must be recorded only from actual CI runs.

## Evidence receipts

- PR HEAD: PENDING
- 4-OS Rust and Qt: NOT_VERIFIED
- Independent code review: NOT_VERIFIED
- PR required checks: NOT_VERIFIED
- Protected Verified Squash SHA: NOT_VERIFIED
- Exact merged-main Push: NOT_VERIFIED

Parent format and physical connectivity issues remain open: #23, #532 and #520. Never commit confidential PCB fixtures.
