# D_016 — Source-only X-Junction Qualification
Recorded: 2026-11-03 KST. Issue #382; code PR #383.
This public PADS V9.4 MILS fixture contains 4 layers, 2 Through-Via placements and 4 straight traces. On Layer 4, the main (150,250)-(350,250) and branch (250,350)-(250,150) centreline segments cross strictly in both interiors. D_015 endpoint/interior T-Junction is excluded. This is a perpendicular-only source geometry candidate and not proof of manufactured electrical connectivity.
The bounded graph reports 8 source nodes, 5 same-layer candidate edges, 3 local components and 0 proven cross-layer electrical edges. SOURCE SHA-256, full strict source inventory, Net/Layer/Via/Trace, Canonical consistency and Qt reimport evidence must match. Off-layer, near miss, endpoint, collinear, skew, extra Copper/COPCLS/VIA, forged witness, and stale SHA are rejected.
Code PR #383 final HEAD: 8f189b8228d114cf14bdcafeed002f2edcb07197. PR 27/27 checks and 11/11 workflows SUCCESS, exact-HEAD independent Codex completed with zero findings and zero unresolved review threads. GitHub Verified one-parent Squash main ccffcb6598e07dd95cc37b526c8ff015207b3546 (parent a8d5e1d7cac6c9a26a9cb8643f7225b8bc28e3db).
- [ ] Confirm code-main exact SHA independent push checks/workflows after completion.
- [ ] Confirm separate documentation PR exact HEAD CI/review, protected Verified Squash and doc-main push checks.
- [ ] Only then close Issue #382.
Explicitly NOT verified: fabricated via plating/barrel, global open/short, physical Copper continuity, return path P009, PH-08, 3D EM, SI Solver, Scenario/cache, or formal M1 U/T. Parent #12 OPEN; Draft PR #35 unchanged.
