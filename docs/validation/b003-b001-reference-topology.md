# B_003 — Trace/Via source-grid contact closure (B_001 continuation)

Date opened: 2026-10-27 KST · [Issue #12](https://github.com/july-rust/BoardSI/issues/12) · starting `main@fca3ffda5fcd03a44510c57a4a253caa603d991c`

## Explicitly bounded implementation

- Extend existing `boardsi-import::connectivity_preview` exact endpoint contacts with source-coordinate integer, **orthogonal straight-trace interior** witnesses, including source endpoints of other traces and explicitly plated Through-Via centre locations on conductive padstack layers.
- Validate the interior witness independently in `boardsi-model::BoardModel::validate`: same frame/layer, strictly interior cross/dot products in i128, matching known Net required for a graph edge. A distinct Net only increments an unqualified potential conflict count.
- Preserve A_032 `BOARDSI_CONTACT_RECORDS_V1` as endpoint-only by filtering new interior-witness edges from its legacy source transport. Never misrepresent an interior as endpoint 0/1.
- Keep pad Net `None` and Pin Net `None`: the current PADS PART importer has no qualified net binding. **No Pad-to-Trace electrical edge** is authorized. This is a remaining B_003 gate, not silently passed.
- Diagonal trace interiors, arc interiors, trace-width overlaps, area/pour/plane contact, pads lacking electrical net identity, arbitrary via stack, overall short/open and SI/Return Path remain NOT_VERIFIED.

## Reproducible regression cases

The code includes synthetic tests for orthogonal T-junction, one-tick noncontact, source plated Through Via with both layer contacts, independent rejection of a forged interior witness, and unchanged endpoint-only legacy V1 serialization.

## Evidence checkpoints

- [ ] Final PR-head Rust compile and unit tests.
- [ ] Native Qt Windows x64, Windows ARM64, macOS ARM64 and Linux checks, plus regressions.
- [ ] Independent Codex same-HEAD review.
- [ ] Protected Verified Squash Merge and post-main CI.
- [ ] Check exactly what functionality is verified, and record checked/evidence KST dates with exact SHA in development checklist.

## Hard gate

**Until test and review evidence is collected, the code is a candidate and nothing is formally accepted.**
B_001 remains PARTIAL and Issue #12 OPEN. U09/U10/U39 and T02/T04/T24/T52 remain unchecked.
B_003 Pad-to-Trace net identity and wider geometry evidence are explicitly unimplemented.

`reference_continuity=UNSUPPORTED_NO_COPPER_NET_EVIDENCE`,
`return_path_p009=UNSUPPORTED`,
`electrical_net_separation=NOT_VERIFIED`,
`solver_ready=false`,
`physical_si=NOT_RUN` remain unchanged.
