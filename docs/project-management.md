# BoardSI Project Management

Source of truth: [Development Specification v1.0](specification/BoardSI_Development_Spec_v1.0.md). Requirements U01–U48; acceptance tests T01–T54. A document's completion is NOT proof of program correctness.

## Milestones and labels

[Project plan JSON](project-plan.json) defines 11 development milestone stages, 22 labels and 24 planning Issues. The workflow [bootstrap-management.yml](../.github/workflows/bootstrap-management.yml) creates missing GitHub Milestones/Labels and links existing Issues by their exact titles. Trigger: merge of setup PR into main, or Actions → Bootstrap GitHub Milestones and Labels → Run workflow. It is idempotent; it must show a successful Actions run before considering Milestones created.

## GitHub Projects v2

The connected GitHub service does not offer a direct Projects creation action. User: create a Board project **BoardSI Development** at https://github.com/users/july-rust/projects/new and link the BoardSI repository. Recommended status columns: Backlog, Ready, In Progress, In Review, Blocked, Done. Add Priority (P0/P1/P2), Area and Requirement/Test fields when supported. Issues and Milestones remain the durable task records.

## Development policy

- M0: Qt/Rust binding, versioned IPC, canonical geometry model, native OS builds, public fixtures and offline confidentiality.
- M1/M1a/M1b: import, artwork, substrate geometry, port configuration and validation.
- M2/M2a/M2b: validated RLGC/direct S, progressive sweep, live statuses, bounded RAM/SSD cache and recovery — initial release gates.
- M2c/M3/M4/M5: **future scope only**; never imply implemented.
- All work on focused branches; merge only after tests and written evidence.
- No real company PCB/stackup/netlist/Touchstone data in repository, PRs, Issues, CI or artifacts.
- Native compilation and actual runtime acceptance are distinct; the docs-only CI intentionally checks neither.

## M2 G0 / G1 change history — 2026-10-15 KST

- [#33](https://github.com/july-rust/BoardSI/pull/33) G0 independent synthetic PEC-grounded-box FVM oracle and frozen mathematical/numerical gates were squash merged at `main@ec5fa4b048c0c00d8588dfd90f2c35e1107e220b`.
- [#34](https://github.com/july-rust/BoardSI/pull/34) G1 isolates a triangular Rust FEM prototype and an independent SciPy FVM / Python ABCD comparator. Final review hardened Mesh dimension overflow and enforced the frozen `1e-12` *complex* S tolerance, distinguishing matched PH-01 from mismatched 50-ohm PH-02. [G1 evidence and exact status ledger](validation/m2-g1-lossless-fem-evidence.md) records partial scoped PASS, Not Run, raw observed numbers, and an initial M0 Worker regression failure requiring clean subsequent CI before merge.
- All actual PCB lossy RLGC, DC, multiconductor, open-boundary/frequency qualification, full product T-tests, and the PH-03 exact FEM plate remain outside the accepted G1 scope. Issue [#16](https://github.com/july-rust/BoardSI/issues/16) stays **Open**.
- Subsequent order is **G2** PH-05..08 other Single-ended geometries; **G3** copper/dielectric losses; **G4** differential/multiconductor; **G5** Canonical Board Model→Solver integration. [#35](https://github.com/july-rust/BoardSI/pull/35) is a draft **G5 architecture preflight**, not proof that G2 or G5 is implemented.
