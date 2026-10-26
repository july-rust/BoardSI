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

## Exact-head acceptance

**Pending CI/evidence gates:**

1. Native M0 validator mutation run and mandatory protected GitHub CI (17 required contexts);
2. CodeQL `Analyze (python)` at the exact PR HEAD;
3. independent Codex exact-HEAD review with no unresolved security/correctness issues;
4. protected signed squash merge and post-merge main CI including CodeQL Python analysis;
5. actual Alert #1 state observed via the GitHub Security UI or a suitably authorized Code Scanning API.

The current GitHub connector can read repository source/checks but **does not expose the Code Scanning alert details/status endpoint**. Accordingly no claim that Alert #1 is closed will be made solely on the basis of a green CodeQL job. A green job confirms scanner execution, not by itself the count of open alerts. If visibility remains restricted after main analysis, the precise user-facing follow-up will be to check the original alert URL for `Fixed` and whether an equivalent alert remains open.

## Scope limits

No PADS import grammar, GPU Artwork Viewer, SI algorithms, V1–V14 IPC, or source provenance semantics are changed. A_035 / P2-Z remains a separate development step; do not claim the whole security surface audited or production software certified by this single CodeQL alert remediation.
