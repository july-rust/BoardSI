# PH-08 Stage 4 original HMM Gate Q replay — GitHub-first evidence

**Date:** 2026-10-16 KST. **Issue:** #51. **Scope:** Stage 4 original HMM MMS replay and exploratory cubic diagnostics. **Not** a physical accuracy acceptance or Contract freeze.

## Provenance

- Authoritative private archive: `boardsi-ph08-stage4-qualification.zip`; known-good SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`.
- Main auditor (PR #53): 338/338 checksums and 20/20 `evidence/source_snapshot` hashes verified.
- Re-execute `m2_ph08_stage4_qualification.mms_suite()` from actual ZIP source, not surrogate Stage 5C/5D local interpolation. Full 36 MMS cases at n=8/16/32 across uniform/skew/graded were compared to archived `evidence/qualification.json`. 36/36 match for selected scalar fields using abs tolerance 2e-13, rel 5e-10; 0 differences. Python 3.13.5, NumPy 2.3.5, SciPy 1.17.0.
- Preserved historical one failure: `uniform/quadratic/bulk_gradient_rms/reduction` ratios **0.953506146487913** and **0.45916314091601546**, below the preregistered **1.8** limit.
- Gradient RMS at n=8/16/32: 4.043940694889194e-15 / 4.241127033930931e-15 / 9.236645226944874e-15. The apparent non-convergence is consistent with roundoff dominance; this remains an **inference**, not a post-hoc excuse.

## Separate original-solver cubic probe — exploratory, not a preregistered gate

Used the archived Stage 4 HMM `solve` + Stage 3 `Recovery`, with u=x³+z³, source -6(x+z), Simpson face-mean Dirichlet conditions, six probes, n=8/16/32.

| Grid | Gradient RMS (8,16,32) | Reduction (8/16,16/32) | Max quadratic recovery condition |
| --- | --- | --- | ---: |
| Uniform | 0.03730645157 / 0.01125194737 / 0.00268803040 | 3.315555 / 4.185945 | 11.60 |
| Skew | 0.03522338663 / 0.01103166678 / 0.00258444934 | 3.192934 / 4.268479 | 15.72 |
| Graded | 0.08627807520 / 0.02892997014 / 0.00779240128 | 2.982308 / 3.712587 | 25.62 |

This supports a truncation-dominated cubic gradient regime alongside a double-precision-level quadratic regime. It is **not** new PH-08 acceptance and does not validate real PCB, open-domain/finite-precision C/C0 enclosure or the original Stage 5B result. A +0.0001 V/m x-field probe mutation changed the RMS but was **not** preregistered as an acceptance threshold.

## Reproducibility path (private original archive required)

```bash
python -m pip install numpy==2.3.5 scipy==1.17.0
python tools/validation/ph08_stage4_gateq_replay.py /path/to/boardsi-ph08-stage4-qualification.zip --output-json /private/stage4-replay.json
python -m unittest discover -s tools/validation -p "test_ph08_stage4_gateq_replay.py" -v
```

The runner requires exact pinned external digest and passes the checksum/source inventory audit before executing the **trusted, SHA-pinned** archived source in a temporary directory. Successful execution returns `REPRODUCED_HISTORICAL_FAIL`, *never* physics PASS. CI without the original archive exercises only synthetic positive and negative tests. The original ZIP is not in public GitHub.

## Remaining requirements

Historical Stage 4 Gate Q **FAIL**, AC-P1 **PENDING**, Contract **NOT FROZEN**, Rust FEM **NO-GO**. Stage 5B original archive unavailable. Need separate source-backed Gate Q error-floor methodology that is prospectively registered and reviewed, and independent quantitative bounds including finite precision and open-domain tail before physical acceptance. No Frozen PH-03/05/06/07 changes; no PR #35 merge.
