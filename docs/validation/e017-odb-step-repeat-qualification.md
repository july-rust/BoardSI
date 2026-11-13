# E_017 — ODB++ single-level Step Repeat source qualification

**Status:** IN_PROGRESS — code PR [#610](https://github.com/july-rust/BoardSI/pull/610). Do not promote to COMPLETE before final-head required CI, independent review, Verified protected Squash, and exact merged-main Push checks are recorded.

## Normative evidence
Siemens [ODB++Design Format Specification 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), *stephdr* pp. 139–141 (X/Y, DX/DY, NX/NY and mandatory interpretation of ANGLE/FLIP/MIRROR). This is a deliberately narrow implementation and NOT Siemens-certified vendor interoperability.

## Accepted source subset
- Exactly one parent `STEP-REPEAT` record naming a distinct, matrix-declared child.
- ANGLE=0, FLIP=NO, MIRROR=NO; positive integer NX*NY <= 64.
- Rectangular X/Y plus DX/DY placements, with exact decimal (nine-place) source ticks and checked integer arithmetic.
- Explicit parent/child `UNITS=MM|INCH`, required Datum fields; parent zero Datum and zero declared origins, arbitrary bounded child Datum; exact rational metre differences.
- Child layer Source Geometry emitted using the existing E_016 parser: L/P/A, supported positive S and disjoint Multi-Island; one replicated canonical frame per child layer per placement.
- Stable object IDs per instance; original child FileMap SHA and original feature path:line preserved in source identity and witness.
- No electrical nodes or edges, no via plating or SI proof.

## Fail-closed boundary
- Missing child, self/nested/multiple Step Repeat, malformed or duplicate fields, unsupported unit/angle/mirror/flip, zero or excessive counts, coincident placements, overflow, coordinate/resource limit.
- Unsupported nonzero parent Datum or any nonzero X_ORIGIN/Y_ORIGIN; explicit parent feature files/EDA; unsupported opt-in EDA in parent array.
- Transactional failure: no partial BoardModel.
- Source fixture tests do not establish independent commercial CAD exporter equivalence (EB-01).

## Verification contract
The PR includes positive and negative tests of 2x2 placement, row-major order, exact metre-space coordinates, mixed units and child Datum, unique IDs, Source SHA mutation, E_016 multi-island preservation, malformed and unsupported stephdr, overlapping source paths, and legacy import behavior. Confirm in **final** PR head and original-source tests, not only derived geometric counts.

### Evidence to record only after completion
- Final PR HEAD: PENDING
- Exact PR Required Checks / native Win x64, Win ARM64, macOS ARM64: PENDING
- Independent review / unresolved threads: PENDING
- Protected Verified Squash SHA: PENDING
- Exact merged-main Push 8 workflow results: PENDING

**Overall importer Issue [#10](https://github.com/july-rust/BoardSI/issues/10) remains OPEN.** Historic [#578](https://github.com/july-rust/BoardSI/issues/578) remains closed (administrative closure, not complete import). E_018+ remain NOT_STARTED.