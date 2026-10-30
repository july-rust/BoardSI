# C_018 — V5–V8 source-only integration

Tracker: https://github.com/july-rust/BoardSI/issues/290

Baseline: `main@96c3c9f74dd7326dd309341ed4a7bb4dfb04e667` (2026-10-30 KST).

Status: IN PROGRESS. No tests or CI for C_018 are yet certified.

Scope: Public synthetic PADS V9.4 MILS existing V5/V6/V7/V8 receipt ancestry, source SHA, COPCLS/COPCUT area identity, Pad Footprint and Via Pad separation; reuse existing native Qt tests.

Nonclaims: `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. Formal U09/U10/U39/T02/T04/T24/T52 and parent #12 remain OPEN. No fabricated Copper/thermal/anti-pad, RLGC/S or production Solver/Cache validation.

Pending gates: Rust Debug/Release, rustfmt/Clippy, existing native Qt Windows x64/ARM64, macOS ARM64 and Linux CTest, latest HEAD Codex review, protected signed Verified squash merge, independent post-main CI, dated checklist and documentation-only verification.
