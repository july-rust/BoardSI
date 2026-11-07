# Contributing to BoardSI

[Project overview](README.md) · [Documentation hub](docs/README.md) · [Developer Quickstart](docs/developer-quickstart.md) · [Current checklist](docs/development-checklist.md) · [Security](SECURITY.md)

BoardSI is a **pre-release Rust/Qt engineering prototype** with a partially supported public-synthetic PADS V9.4-MILS Import/Artwork path. **There is no product-qualified, measured-PCB SI solver or verified public installer.** A focused contribution must preserve that distinction.

## 1. Source of truth and ownership

| Purpose | Authoritative place | Must not be replaced by |
| :--- | :--- | :--- |
| Product requirements | [Frozen v1.0 specification](docs/specification/BoardSI_Development_Spec_v1.0.md), U01–U48 and T01–T54 | A code comment, proposal, or passing prototype test |
| Living engineering status | **Single [development checklist](docs/development-checklist.md)** with per-item first-check date and evidence | Second checklist, dashboards with manually inconsistent counts, or completed GitHub issue alone |
| Executed test proof | [Validation records](docs/validation/README.md) and exact PR-head/CI SHA | CI green with missing scope or unrelated later commits |
| Architecture | [Architecture map](docs/architecture/README.md) and explicit ADRs/contracts | An unapproved experimental implementation |
| Historical findings | Dated [project audit](docs/audits/2026-10-18-pre-ph08-project-audit.md) and test ledgers | Silent retrospective rewriting of a FAIL or changing evidence thresholds |

The owner-approved BoardSI SVG brand master and frozen physical acceptance contracts remain unchanged unless the owner explicitly approves a separate revision. Historical PH-08 Gate Q **FAIL** stays FAIL; the AC-P1/physical model approval gates do not pass just because synthetic CI succeeds.

## 2. Feature pull requests

1. **Keep the work small, meaningful and reusable.** One product capability or one coherent integration boundary per PR; prefer generic geometry/algorithm tests over separate work for every trace angle, Via-side entry or Pad corner. [D_054/D_055 pattern](docs/validation/d054-d055-general-trace-contact-qualification.md).
2. **State what is supported and rejected.** Include exact input revision/object subset, layer/Net provenance, units, performance budgets, uncertainty handling and output contract. Unsupported or ambiguous records must fail closed instead of silently producing fictitious copper or a false “PASS.”
3. **Use independent evidence.** Couple Rust tests with synthetic source inputs, adversarial/negative cases and platform tests as applicable. Exact source SHA, expected and actual results, compiler/OS, test type and dates must be clear.
4. **Preserve the full safety boundary.** \`local geometric contact candidate\` is not \`verified electrical edge\`; \`supplied scalar RLGC mathematics\` is not \`measured PCB physics\`; a native Qt build is not a redistributable release.
5. **Update the checklist in the same code PR** when implementation status truly changes. If no requirement/test acceptance changed, state so rather than marking formal U/T completed. Avoid creating a second PR just to transcribe that same evidence.
6. **Use protected PR workflows and Squash Merge.** Required checks and Verified-commit policy are not bypassed for speed. Keep review issues resolved, and distinguish PR-head CI from the post-main Push check.

A native-platform compilation does not equal a native runtime pass. A geometric/circuit test does not equal final full-board or product approval. Track both explicitly.

## 3. Documentation standards

Every new or substantially revised **living** page should answer, near its top:

- **What is this?** Audience, subsystem, normative/reference/historical status, and the relevant specification / Issue / PR.
- **When is it true?** Date (KST) and, for evidence, exact commit or CI run; do not label a years-old historical report “current.”
- **What actually passed?** Input and expected result, code/test SHA, OS/backend and all limitations. An approval is valid only for its named scope.
- **What remains open?** Missing objects, unsafe inputs, absent independent oracle, actual user-product integration, U/T status and original PH-08 gate.
- **Where next?** Relative links to the authoritative canonical page and neighboring interfaces, not a duplicated manually maintained progress table.

Prefer short introductory paragraphs, small comparison tables, one diagram when it clarifies ownership/flow, and specific link labels. **Do not bury the only warning in a footnote.** Avoid overlong chronological narratives on the root landing page; preserve that level of detail in dated validation/audit files.

Use clear evidence labels: \`PASS (scoped)\`, \`PARTIAL\`, \`PROPOSED\`, \`BLOCKED\`, \`NOT_RUN\`, \`NOT_VERIFIED\`, \`FAIL\`. These are **not interchangeable**. Distinguish verified facts from engineering inference or a planned target. Use SI units in engineering prose; PADS' intrinsic source-unit notation must be identified as the file format, not silently converted or rounded.

**Links and structure:** use relative links for repository documents; check that both target *files* and referenced anchors exist. Do not link to mutable branch content as permanent proof when an exact SHA/PR evidence link is available. Keep the specification, test contracts and old PASS/FAIL receipts immutable except for a clearly identified erratum that cites the original.

## 4. Documentation-only CI (save Windows/macOS runners)

A PR limited to supported Markdown documentation files (\`docs/**/*.md\`, \`README.md\`, \`CONTRIBUTING.md\`, \`SECURITY.md\`, plus other patterns explicitly recognized by the [shared classifier](.github/actions/classify-docs-only/action.yml)) keeps all protected status-check **jobs** visible, but **skips actual native Rust/Qt/FEM runner allocation**. A lightweight Linux classification check and [required docs checker](.github/workflows/docs-check.yml) still run.

- **Do not** add workflow-level \`paths-ignore\` to a required check: skipped workflows can remain pending and block protected merges.
- Edits to code, \`*.json\` metadata, tests, fixtures or workflows are **not** docs-only and continue through full applicable native validations. Git diff/classifier failures fail closed.
- Group related editorial changes into a **single docs-only PR**, check link targets and document consistency locally, and preserve the single canonical checklist.
- Documentation PR CI **does not** test actual PCB SI physics or Qt GUI; never treat it as a product validation event.

The number and names of required workflow groups may evolve. Use the **current** protected repository rules and workflow files; do not assume a historical count from an older audit is the active set.

## 5. Security, licensing and release

Never upload a company PCB, schematic, board-export archive, stackup, netlist, confidential Touchstone/SNP, proprietary IBIS, customer measurement, absolute internal path, credentials or private HMM source to public Issues, PRs, CI logs, attachments or generated reports. Use public synthetic fixtures with explicit provenance. Keep source and processing local/offline when dealing with confidential designs.

Do not claim an open-source license or permission to redistribute a Qt bundle unless those rights and third-party notices are **actually verified**. Clean OS, platform binaries, model correctness, privacy and official U/T acceptance need separate release evidence. [Preview delivery policy](docs/preview-delivery-playbook.md) is a **development strategy**, not a release certificate.

## 6. Before submitting

- Run the relevant Rust unit/integration/negative tests and code formatting when code changes.
- Verify input SOURCE identity, expected behavior, explicit limitations and Net/layer/provenance.
- Check relative links, spelling, glossary/status consistency and KST evidence dates.
- Update **one** checklist only if a checked item truly has an executable/test evidence basis.
- Confirm required CI reports and resolved review threads before Squash Merge.
- For physical modelling: retain historical Gate Q FAIL and do not promote a scoped synthetic PASS to a full PCB SI result.

**Questions about current status:** [Development checklist](docs/development-checklist.md). **Questions about why an earlier result was accepted or rejected:** [Validation index](docs/validation/README.md) and date-pinned audits.
