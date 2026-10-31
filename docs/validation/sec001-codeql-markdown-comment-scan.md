# SEC_001 — GitHub Code Scanning Alert #1

- Repository: `july-rust/BoardSI`
- Alert URL: https://github.com/july-rust/BoardSI/security/code-scanning/1
- Rule: CodeQL `py/bad-tag-filter` / Bad HTML filtering regexp
- Original location: `tools/validation/m0_traceability_check.py:201`
- Start baseline: protected `main@b284a9762fcac0590ce7481f81ca3bd26b860599`
- Tracking: [SEC_001 Issue #205](https://github.com/july-rust/BoardSI/issues/205)

## Analysis

The flagged expression `re.finditer(r"<!--.*?-->", comment_visibility)` scanned **one Markdown physical source line**, already processed by `mask_inline_code_for_comment_scan`. It was **not** a general-purpose HTML sanitizer and is not used to emit browser HTML. Existing `visible_markdown_lines()` already strips physical newlines and independently parses cross-line comments with `in_comment`. No concrete XSS or RCE exploit is established from this alert alone.

Still, an HTML-comment regex for markup validation is unnecessary and can mislead CodeQL or future maintainers about newlines and HTML parsing. CodeQL's detection was not bypassed or dismissed: the per-line token scan is now literal `str.find("<!--")` then `str.find("-->")`, preserving already-masked source positions and odd/even backslash escapes. The cross-line state machine remains the authority for multiline comments and fail-closed HTML/container validation.

## Regression coverage

The existing M0 traceability script's mutation tests still cover hidden required U/T rows, multiline comments, escaped openers, fenced/inline code, raw HTML block concealment, synthesized table delimiters, original source boundaries, and all 48 requirements/54 tests. SEC_001 adds:
- multiple same-line comments with backticks inside them and legitimate visible text afterwards;
- source HTML-looking nested tokens inside a masked same-line comment;
- actual newline-spanning comments, including a raw `<style>` string that must remain inert;
- a required crosswalk header hidden inside a physical multi-line comment, which must still fail closed.

## Verified code remediation and direct alert observation

- [SEC_001 source remediation PR #206](https://github.com/july-rust/BoardSI/pull/206) was merged on 2026-10-26 into **signed protected** `main@6208db7083f4c56f91810e79d74ae22074221ecd`.
- [Exact merged SHA CI](https://github.com/july-rust/BoardSI/commit/6208db7083f4c56f91810e79d74ae22074221ecd/checks) completed **21/21 SUCCESS**, including `docs` (M0 traceability mutation tests) and `Analyze (python)` CodeQL.
- On 2026-10-31, the then-current [main@477428da](https://github.com/july-rust/BoardSI/commit/477428da3895ee4a2ff19dda5d53932754e3a6ba/checks) also completed **21/21 SUCCESS**, with an independent new CodeQL Python analysis.
- The connected GitHub app's generic Fetch API rejects the Code Scanning family at its URL allowlist; this does **not** mean the user's account lacks repository admin rights.
- To obtain actual evidence, [C_025_02 GitHub Actions run #41183754657](https://github.com/july-rust/BoardSI/actions/runs/41183754657), using the repository's short-lived `GITHUB_TOKEN` with only `security-events: read` and `contents: read`, queried GitHub REST directly. The job **completed SUCCESS** and returned:
  - `SEC_001_RESULT=FIXED_VERIFIED`
  - `SEC_001_STATE=fixed`
  - `SEC_001_FIXED_AT=2026-10-26T08:25:02Z`
  - `SEC_001_RULE=py/bad-tag-filter`
  - `SEC_001_EQUIVALENT_OPEN_ALERT_NUMBERS=[]`
  - `SEC_001_DEFAULT_BRANCH_SHA=477428da3895ee4a2ff19dda5d53932754e3a6ba`
  - `SEC_001_OBSERVED_AT_UTC=2026-10-31T08:14:14+00:00`
- This is direct **Fixed** confirmation for Alert #1 and absence of simultaneously open alerts with the *same rule ID*, not a claim that all code scanning alerts or the entire software security surface are clear. No alert dismissal or state-update API was used.
- [C_025_02 read-only audit workflow PR #316](https://github.com/july-rust/BoardSI/pull/316) provides a repeatable manual audit after its independent protected CI/review/merge; the observed Fixed result above comes from its actual owned-branch Actions execution, not from an untested proposal.

## Scope limits

No PADS import grammar, GPU Artwork Viewer, SI algorithms, V1–V14 IPC, or source provenance semantics are changed. A_035 / P2-Z remains a separate development step; do not claim the whole security surface audited or production software certified by this single CodeQL alert remediation.
