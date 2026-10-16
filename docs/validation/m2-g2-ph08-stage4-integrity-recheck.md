# PH-08 Stage 4 evidence integrity and schema reconciliation

**2026-10-16 KST; provenance only; not a physical solver qualification.**

An authorized copy of \`boardsi-ph08-stage4-qualification.zip\` was located in the personal file Library. The raw archive remains outside GitHub; only the non-confidential, limited integrity summary is committed here.

- Original archive byte size: **2,301,227**.
- Archive SHA-256: \`6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9\`; independently recomputed and matched historical external SHA.
- ZIP members: **339**; checksum list entries: **338**; actual checked digests **338/338** matched; unlisted **0**.
- Actual \`evidence/manifest.json\` contains \`sources\` as a mapping of 20 **basenames** to SHA-256 strings. The Stage 5E auditor initially returned \`NOT_VERIFIED\` because it recognized list schemas only; this was an unsupported schema, not a physical failure.
- Independently canonicalized each basename to the archive-root-relative \`evidence/source_snapshot/<basename>\`, checked the actual raw data bytes, and found **20/20 matching source digests**.
- No claim is made that exploratory \`development-failures\` copies have the same hash: some intentionally differ; only the exact source snapshot is authoritative for this inventory.
- The Stage 4 archive's own status indicates preregistration, **not a Contract freeze**. Its original uniform quadratic gradient Gate Q remains **FAIL**; AC-P1 **PENDING**, contract **NOT FROZEN**, and Rust FEM **NO-GO**.
- Stage 5B raw ZIP remains unavailable. Do not transfer this Stage 4 338/20 success onto Stage 5B.
- The GitHub auditor now accepts the exact safe Stage 4 dictionary layout, retains strict duplicate JSON key failure, and adds five synthetic mapping cases. No original archive contents or private local paths are committed.

This is an integrity and schema qualification only. It must not be cited as a verified reference for whole-space C/C0, field error or PH-08 Rust implementation.
