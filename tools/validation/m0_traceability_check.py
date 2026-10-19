"""Fail-closed coverage validator for the frozen v1.0 U/T ID baseline.

Associations here are traceability, never a claim of executed T acceptance.
"""
import re
from pathlib import Path

SPEC = Path("docs/specification/BoardSI_Development_Spec_v1.0.md")
LEDGER = Path("docs/audits/m0-requirements-traceability-ledger.md")

def expand_ids(text, prefix):
    results = []
    for item in text.split(","):
        match = re.fullmatch(rf"{prefix}([0-9]{{2}})(?:~(?:{prefix})?([0-9]{{2}}))?", item.strip())
        if not match:
            raise ValueError(f"invalid {prefix} crosswalk group: {item!r}")
        first = int(match.group(1))
        last = int(match.group(2) or match.group(1))
        if first > last or first < 1:
            raise ValueError(f"invalid {prefix} range: {item!r}")
        results += [f"{prefix}{n:02d}" for n in range(first, last + 1)]
    if len(results) != len(set(results)):
        raise ValueError("duplicate or overlapping " + prefix + " IDs in group: " + text)
    return results

def split_markdown_cells(line):
    """Split actual Markdown columns; escaped bars remain within text cells."""
    cells, buffer = [], []
    index = 0
    while index < len(line):
        character = line[index]
        if character == "\\" and index + 1 < len(line):
            buffer.append(character)
            buffer.append(line[index + 1])
            index += 2
            continue
        if character == "|":
            cells.append("".join(buffer).strip())
            buffer = []
        else:
            buffer.append(character)
        index += 1
    cells.append("".join(buffer).strip())
    if cells and cells[0] == "":
        cells.pop(0)
    if cells and cells[-1] == "":
        cells.pop()
    return cells


def is_escaped_markdown_token(line, position):
    """GFM backslash escape: odd preceding backslashes escape the token."""
    count = 0
    at = position - 1
    while at >= 0 and line[at] == chr(92):
        count += 1
        at -= 1
    return count % 2 != 0


def mask_inline_code_for_comment_scan(line):
    """Mask only paired backtick spans when looking for comment openers."""
    masked = list(line)
    def is_escaped(position):
        backslashes = 0
        cursor = position - 1
        while cursor >= 0 and line[cursor] == chr(92):
            backslashes += 1
            cursor -= 1
        return backslashes % 2 != 0

    index = 0
    while index < len(line):
        if line[index] != chr(96) or is_escaped(index):
            index += 1
            continue
        end_run = index
        while end_run < len(line) and line[end_run] == chr(96):
            end_run += 1
        delimiter = line[index:end_run]
        search = end_run
        closing = -1
        while search < len(line):
            next_open = line.find(delimiter, search)
            if next_open == -1:
                break
            next_end = next_open + len(delimiter)
            # GFM backslash escapes are NOT processed inside code spans:
            # a backtick preceded by "\" can still close an active span.
            if ((next_open == 0 or line[next_open - 1] != chr(96)) and
                (next_end == len(line) or line[next_end] != chr(96))):
                closing = next_end
                break
            search = next_end
        if closing == -1:
            index = end_run
        else:
            for position in range(index, closing):
                masked[position] = " "
            index = closing
    return "".join(masked)


def unclosed_inline_code_run(line):
    """True when a physical line starts a GFM backtick span it cannot close.

    This controlled normative document format forbids multiline inline code
    because a masked cross-line span can spoof HTML comments/block boundaries.
    Regular fenced code blocks are handled separately by the caller.
    """
    index = 0
    open_width = None
    while index < len(line):
        if line[index] != chr(96):
            index += 1
            continue
        # Outside code spans, escaped backticks are literal punctuation.
        if open_width is None:
            slash_count = 0
            cursor = index - 1
            while cursor >= 0 and line[cursor] == chr(92):
                slash_count += 1
                cursor -= 1
            if slash_count % 2:
                index += 1
                continue
        end = index
        while end < len(line) and line[end] == chr(96):
            end += 1
        width = end - index
        if open_width is None:
            open_width = width
        elif open_width == width:
            open_width = None
        index = end
    return open_width is not None


def visible_markdown_lines(source):
    """Return visible text; reject raw HTML containers in this controlled spec.

    Parse fences and comments across the entire source before section slicing.
    Our normative definition tables deliberately use ordinary Markdown, not
    raw-HTML container elements; fail closed on unsupported markup.
    """
    in_comment = False
    fence_character = None
    fence_length = 0
    html_open = re.compile(
        r"^ {0,3}(?:</?[A-Za-z][A-Za-z0-9-]*(?=[\s>/]|$)|<![A-Za-z\[]|<\?)",
        re.IGNORECASE,
    )
    # GFM only recognizes CRLF, CR or LF as line endings, never VT/FF/NEL/LS/PS.
    source = source.replace("\r\n", "\n").replace("\r", "\n")
    for original_line in source.split("\n"):
        # A fenced block owns all its content, even HTML-looking tokens.
        if fence_character is not None:
            line = original_line.lstrip(" ")
            indent = len(original_line) - len(line)
            if indent <= 3 and line.startswith(fence_character):
                length = len(line) - len(line.lstrip(fence_character))
                if length >= fence_length and all(ch in (" ", "\t") for ch in line[length:]):
                    fence_character, fence_length = None, 0
            yield ""
            continue

        # Indented code outside an existing comment is not HTML markup.
        # Existing multi-line comments must still consume their closing token.
        if not in_comment and original_line.expandtabs(4).startswith("    "):
            yield ""
            continue
        # A GFM inline-code span can extend across physical lines. In this
        # controlled spec/ledger such a span is unsupported: fail closed on
        # any opener without a same-line matching delimiter rather than
        # misreading protected `<!--` text as a real HTML comment.
        original_open = original_line.lstrip(" ")
        original_indent_for_fence = len(original_line) - len(original_open)
        real_fence_start = (
            original_indent_for_fence <= 3
            and original_open.startswith(("~~~", chr(96) * 3))
        )
        # Recognize a genuine source GFM fence BEFORE scanning its info
        # string for HTML-comment-looking text. The info string is inert:
        # e.g. an opening fence with info "x <!--" is NOT a comment opener.
        # Its following real closing fence must resume scanning raw HTML.
        if real_fence_start and not in_comment:
            opener_char = original_open[0]
            opener_width = len(original_open) - len(original_open.lstrip(opener_char))
            if opener_char == chr(96) and chr(96) in original_open[opener_width:]:
                raise ValueError("invalid original backtick fence info string")
            fence_character, fence_length = opener_char, opener_width
            yield ""
            continue

        # Single-line real HTML comments are invisible to GFM: backticks
        # contained in them must not create false unclosed code spans.
        # Cross-line spoofing remains rejected because it has no same-line
        # HTML-comment terminator and retains its unmatched delimiter.
        comment_visibility = mask_inline_code_for_comment_scan(original_line)
        visible_code_chars = list(original_line)
        for comment_match in re.finditer(r"<!--.*?-->", comment_visibility):
            # A backslash-escaped '<' is literal GFM text, not an opener.
            if is_escaped_markdown_token(original_line, comment_match.start()):
                continue
            for pos in range(comment_match.start(), comment_match.end()):
                visible_code_chars[pos] = " "
        line_for_code_spans = "".join(visible_code_chars)
        if not in_comment and not real_fence_start and unclosed_inline_code_run(line_for_code_spans):
            raise ValueError("multiline inline-code span unsupported in normative Markdown")
        # Inline code spans can contain literal "<!--" examples; those are
        # not real HTML comment openers and must not mask subsequent tables.
        comment_scan = mask_inline_code_for_comment_scan(original_line)
        visible = []
        index = 0
        while index < len(original_line):
            if in_comment:
                end = original_line.find("-->", index)
                if end == -1:
                    index = len(original_line)
                else:
                    in_comment = False
                    index = end + 3
            else:
                start = comment_scan.find("<!--", index)
                while start != -1 and is_escaped_markdown_token(original_line, start):
                    start = comment_scan.find("<!--", start + 4)
                if start == -1:
                    visible.append(original_line[index:])
                    index = len(original_line)
                else:
                    visible.append(original_line[index:start])
                    in_comment = True
                    index = start + 4
        text = "".join(visible)
        # Normative GFM table headers must exist in the ORIGINAL source.
        # Stripping a leading HTML comment can synthesize an openerless
        # header even though GFM did not render the original as a table.
        # These five controlled headers never permit embedded HTML comments;
        # prohibit both openerless and pipe-leading header fabrication.
        if text != original_line:
            normative_headers = (
                ("ID", "요구사항", "상태", "근거"),
                ("Test ID", "주요 검증", "통과 조건"),
                ("요구 ID", "본문", "주요 시험", "완료 단계"),
                ("Requirement ID", "Phase from v1.0 18.2",
                 "Linked test IDs", "Definition", "Implementation",
                 "Full acceptance", "Evidence scope"),
                ("Test ID", "Related stage(s)",
                 "Linked requirement IDs (coarse)",
                 "Complete acceptance result", "Evidence"),
            )
            if tuple(split_markdown_cells(text.strip())) in normative_headers:
                raise ValueError("HTML comment synthesized normative table header")
        # GFM evaluates pipe-table delimiters against ORIGINAL source text.
        # A comment inside a delimiter cell can be removed below to synthesize
        # a valid row (e.g. "| <!-- -->--- | --- | ..."), even though that
        # delimiter was invalid in the rendered source. Reject such lines
        # before any downstream consumer sees the comment-stripped version.
        stripped_cells = split_markdown_cells(text.strip())
        if (text != original_line
            and len(stripped_cells) >= 2
            and all(re.fullmatch(r":?-{3,}:?", cell) for cell in stripped_cells)):
            original_cells = split_markdown_cells(original_line.strip())
            if (len(original_cells) != len(stripped_cells)
                or not all(re.fullmatch(r":?-{3,}:?", cell) for cell in original_cells)):
                raise ValueError("HTML comment synthesized Markdown table delimiter")
        # HTML comment removal must NOT invent normative GFM block structure.
        # A line whose original first token is an HTML comment cannot become
        # an ATX heading, table row or table delimiter just by stripping it.
        visible_first = text.lstrip(" ")
        original_first = original_line.lstrip(" ")
        is_heading = lambda value: re.match(r"^#{1,6}(?:[ \t]|$)", value) is not None
        if ((visible_first.startswith("|") and not original_first.startswith("|"))
            or (is_heading(visible_first) and not is_heading(original_first))):
            raise ValueError("HTML comment synthesized Markdown block marker")
        # CommonMark expands mixed whitespace at four-column tab stops.
        if text.expandtabs(4).startswith("    "):
            yield ""
            continue
        unindented = text.lstrip(" ")
        indent = len(text) - len(unindented)
        # Fence syntax belongs to the ORIGINAL Markdown block, not text
        # synthesized by removing inline comments. Otherwise an opener such
        # as "<!-- -->" + "```" wrongly hides subsequent raw HTML.
        original_start = original_line.lstrip(" ")
        original_indent = len(original_line) - len(original_start)
        synthesized_fence = (
            indent <= 3
            and unindented.startswith(("```", "~~~"))
            and not (
                original_indent <= 3
                and original_start.startswith(("```", "~~~"))
            )
        )
        if synthesized_fence:
            raise ValueError("HTML comment cannot synthesize Markdown fence opener")
        if indent <= 3 and unindented:
            character = unindented[0]
            if character in (chr(96), "~"):
                length = len(unindented) - len(unindented.lstrip(character))
                if length >= 3:
                    # GFM: backticks inside a backtick fence info string
                    # invalidate the would-be opener. Fail closed here;
                    # otherwise raw HTML below could be masked incorrectly.
                    if character == chr(96):
                        # Validate the ORIGINAL info string too: HTML comment
                        # removal can erase a forbidden backtick, synthesizing
                        # an opener that GFM would reject in the real source.
                        original_length = len(original_start) - len(
                            original_start.lstrip(character))
                        if (chr(96) in original_start[original_length:]
                            or chr(96) in unindented[length:]):
                            raise ValueError("invalid original backtick fence info string")
                    fence_character, fence_length = character, length
                    yield ""
                    continue

        # Normative definition tables cannot occur inside raw HTML blocks.
        # A literal example inside a code fence/comment is already excluded.
        if html_open.match(text) or re.search(
            r"(?i)</?[A-Za-z][A-Za-z0-9_-]*(?=[\s>/]|$)",
            mask_inline_code_for_comment_scan(text),
        ):
            raise ValueError("raw HTML block is unsupported in normative specification")
        yield text
    if in_comment or fence_character is not None:
        raise ValueError("unclosed HTML comment or code fence in specification")


def visible_section(source, start_title, end_heading_prefix):
    """Slice only at actual unindented visible Markdown heading lines."""
    visible = "\n".join(visible_markdown_lines(source))
    starts = list(re.finditer(
        r"(?m)^" + re.escape(start_title) + r"\s*$", visible
    ))
    if len(starts) != 1:
        raise ValueError("missing/ambiguous rendered section heading: " + start_title)
    start = starts[0].end()
    end = re.search(
        r"(?m)^" + re.escape(end_heading_prefix) + r"[^\n]*$",
        visible[start:],
    )
    if not end:
        raise ValueError("missing rendered section boundary: " + end_heading_prefix)
    return visible[start:start + end.start()]


def validate_definition_tables(source):
    """Require an actual contiguous Markdown definition table, not just ID text."""
    # Parse comments/fences across the ENTIRE document first. An HTML comment
    # starting before a section heading must not make its hidden table count.

    ranges = (
        ("U", 48, "## 2 요구사항 추적표", "## 3 ",
         ["ID", "요구사항", "상태", "근거"]),
        ("T", 54, "## 13 인수시험 및 독립 교차 검증", "### 13.1 ",
         ["Test ID", "주요 검증", "통과 조건"]),
    )
    for prefix, maximum, begin, end, header in ranges:
        section = visible_section(source, begin, end)
        visible = section.split("\n")
        found, table_count = set(), 0
        for position, line in enumerate(visible):
            if split_markdown_cells(line.strip()) != header:
                continue
            table_count += 1
            if position == 0 or visible[position - 1].strip():
                raise ValueError("normative definition table requires blank block boundary")
            if position + 1 >= len(visible):
                raise ValueError("missing Markdown delimiter after " + prefix + " header")
            delim = split_markdown_cells(visible[position + 1].strip())
            if len(delim) != len(header) or not all(
                re.fullmatch(r":?-{3,}:?", c) for c in delim
            ):
                raise ValueError("missing/invalid Markdown table delimiter: " + prefix)
            row_number = position + 2
            while row_number < len(visible):
                row = visible[row_number].strip()
                if not row or "|" not in row:
                    break
                cells = split_markdown_cells(row)
                if len(cells) != len(header):
                    raise ValueError("malformed definition table row: " + row)
                item = cells[0].strip()
                # Only a SINGLE outer formatting wrapper can be removed.
                # Recursive stripping corrupts code-span semantics:
                # GFM renders `*U01*` literally, not as formatted U01.
                if not re.fullmatch(rf"{prefix}[0-9]{{2}}", item):
                    for wrapper in ("**", "__", "*", "_", chr(96)):
                        if (item.startswith(wrapper) and item.endswith(wrapper)
                            and len(item) > 2 * len(wrapper)):
                            # Whitespace adjacent to an emphasis delimiter
                            # prevents GFM from recognizing valid emphasis.
                            candidate = item[len(wrapper):-len(wrapper)]
                            if re.fullmatch(rf"{prefix}[0-9]{{2}}", candidate):
                                item = candidate
                                break
                if not re.fullmatch(rf"{prefix}[0-9]{{2}}", item):
                    raise ValueError("invalid definition ID: " + cells[0])
                if any(not c for c in cells[1:]):
                    raise ValueError("missing definition content: " + item)
                if item in found:
                    raise ValueError("duplicate definition ID: " + item)
                found.add(item)
                row_number += 1
        if table_count != 1:
            raise ValueError(f"{prefix} Markdown definition table header count: {table_count}")
        expected = {f"{prefix}{i:02d}" for i in range(1, maximum + 1)}
        if found != expected:
            raise ValueError(
                f"{prefix} definition mismatch: missing={sorted(expected-found)}, "
                f"extra={sorted(found-expected)}"
            )


def rendered_table_rows(section, header, prefix):
    """Require one contiguous, rendered Markdown table of the expected shape."""
    lines = section.split("\n")
    headers = [
        i for i, line in enumerate(lines)
        if line == line.lstrip() and split_markdown_cells(line.strip()) == header
    ]
    if len(headers) != 1:
        raise ValueError("missing/duplicate rendered " + prefix + " table header")
    start = headers[0]
    if start == 0 or lines[start - 1].strip():
        raise ValueError("normative table requires blank preceding block boundary")
    if start + 1 >= len(lines):
        raise ValueError("missing " + prefix + " table delimiter")
    if lines[start + 1] != lines[start + 1].lstrip():
        raise ValueError("normative table delimiter must start at column zero")
    delimiter = split_markdown_cells(lines[start + 1].strip())
    if len(delimiter) != len(header) or not all(
        re.fullmatch(r":?-{3,}:?", cell) for cell in delimiter
    ):
        raise ValueError("invalid " + prefix + " Markdown table delimiter")
    rows = []
    pos = start + 2
    while pos < len(lines):
        original_row = lines[pos]
        line = original_row.strip()
        if not line:
            break
        if not line.startswith("|"):
            # CommonMark tables also allow rows WITHOUT an opening pipe.
            # In this controlled specification, reject that alternate form
            # rather than silently omitting a visible extra definition/ID row.
            if "|" in line:
                raise ValueError("unsupported openerless Markdown table row")
            break
        if original_row != original_row.lstrip():
            raise ValueError("normative table row must start at column zero")
        cells = split_markdown_cells(line)
        if len(cells) != len(header):
            raise ValueError("invalid " + prefix + " table row width")
        rows.append(line)
        pos += 1
    if not rows:
        raise ValueError("empty " + prefix + " rendered table")
    # Fail closed on detached ID rows that are not part of this table.
    for i, line in enumerate(lines):
        if i < start + 2 or i >= pos:
            if re.match(r"^\s*\|\s*" + prefix + r"[0-9]", line):
                raise ValueError("detached " + prefix + " ID row outside table")
    return rows


# Frozen per-requirement body references from the approved v1.0 crosswalk.
# This is a semantically scoped contract; comparing only existence of chapter
# numbers is insufficient (e.g. U01 points to SI results, not general UI).
FROZEN_BODY_REFS_BY_GROUP = {
    "U01": "8,10,11",
    "U02": "11,12",
    "U03,U14": "12",
    "U04,U13,U22~U24": "9,12,13.1",
    "U05,U15": "12,14,15",
    "U06~U08,U31~U33,U38~U40,U48": "4,5",
    "U09,U10": "6",
    "U11,U34~U37": "7",
    "U12,U17~U21,U25,U41,U46": "9",
    "U16": "1,15",
    "U26~U30": "8.1",
    "U42": "7.8",
    "U43": "10.1",
    "U44": "9.8,11",
    "U45": "9.9,10.3",
    "U47": "13.1,15",
}
FROZEN_BODY_REFS_BY_U = {
    u: set(section.strip() for section in sections.split(","))
    for group, sections in FROZEN_BODY_REFS_BY_GROUP.items()
    for u in expand_ids(group, "U")
}


def extract_mapping(source):
    # Only a real rendered Markdown heading selects the public crosswalk.
    section = visible_section(source, "### 18.2 요구사항과 시험 연결", "### 18.3")
    mapping, u_phases, t_phases = {}, {}, {}
    # A body reference is a real section number, not free-form prose.
    section_numbers = set(re.findall(
        r"(?m)^#{2,5} +([0-9]+(?:\.[0-9]+)*)\s+",
        "\n".join(visible_markdown_lines(source))
        .split("### 18.2 요구사항과 시험 연결", 1)[0],
    ))
    for line in rendered_table_rows(
        section, ["요구 ID", "본문", "주요 시험", "완료 단계"], "U"
    ):
        if not re.match(r"\|\s*U[0-9]{2}", line):
            raise ValueError("non-U crosswalk row: " + line)
        cells = split_markdown_cells(line)
        if len(cells) != 4 or any(not cell.strip() for cell in cells):
            raise ValueError("bad/empty section 18.2 row")
        refs = [ref.strip() for ref in cells[1].split(",")]
        if len(refs) != len(set(refs)):
            raise ValueError("duplicate requirement chapter reference: " + cells[1])
        # Only normative product/engineering chapters form a requirement body.
        # 2 is the U definition list; 16-18 are sources, change history and
        # review/crosswalk metadata, not substantive requirement bodies.
        def normative_body(ref):
            chapter = int(ref.split(".", 1)[0])
            return chapter == 1 or 3 <= chapter <= 15
        if not refs or any(
            ref not in section_numbers or not normative_body(ref) for ref in refs
        ):
            raise ValueError("non-normative or unknown requirement body: " + cells[1])
        tests = expand_ids(cells[2], "T")
        for u in expand_ids(cells[0], "U"):
            if set(refs) != FROZEN_BODY_REFS_BY_U.get(u):
                raise ValueError("incorrect frozen requirement body links: " + u)
            if u in mapping:
                raise ValueError("duplicate U group: " + u)
            mapping[u] = set(tests)
            u_phases[u] = cells[3]
        for t in tests:
            # Preserve first-occurrence order from specification section 18.2.
            history = t_phases.setdefault(t, [])
            if cells[3] not in history:
                history.append(cells[3])
    expected_u = {f"U{i:02d}" for i in range(1, 49)}
    expected_t = {f"T{i:02d}" for i in range(1, 55)}
    mapped_t = set().union(*mapping.values()) if mapping else set()
    if set(mapping) != expected_u or mapped_t != expected_t:
        raise ValueError(f"U/T coverage gaps: U={sorted(expected_u-set(mapping))}, T={sorted(expected_t-mapped_t)}; extras U={sorted(set(mapping)-expected_u)}, T={sorted(mapped_t-expected_t)}")
    # Frozen v1.0 semantic mapping: T44 is the U01 export regression.
    # Bidirectional ledger agreement alone cannot prevent an incorrect
    # mutually coordinated reassignment of T44 to an unrelated U ID.
    t44_owners = {u for u, tests in mapping.items() if "T44" in tests}
    if t44_owners != {"U01"}:
        raise ValueError("frozen T44/U01 semantic association drift")
    return mapping, u_phases, t_phases

# M0 Issue references verified against the existing BoardSI M0 / Qt audit Issues.
# No live GitHub/network access is required in the reproducible docs CI.
# Frozen, row-specific limited prototype evidence. Neither these IDs nor
# PARTIAL status signifies full acceptance. Adding a new PARTIAL row requires
# explicit code review and issue-specific justification, not a global allowlist.
PARTIAL_EVIDENCE_BY_ID = {
    "U02": {3}, "U03": {3, 6}, "U23": {5},
    "U31": {4}, "U39": {4}, "U44": {5},
    "T13": {3, 6}, "T39": {3, 6}, "T46": {5, 46},
}


def has_row_specific_partial_evidence(identifier, value):
    """Require exact reviewed PARTIAL wording, not merely matching issue tokens.

    Future acceptance updates require explicit independent evidence and review.
    """
    allowed = PARTIAL_EVIDENCE_BY_ID.get(identifier)
    if not allowed:
        return False
    links = "; ".join(f"see #{issue}" for issue in sorted(allowed))
    if identifier.startswith("U"):
        expected = "M0 restricted prototype; " + links
    else:
        expected = "M0 native Qt/Worker prototype only; " + links + "; full T unqualified"
    return value == expected


def has_audited_ledger_evidence(identifier, status, evidence):
    """Fail closed on evidence contradicting the ledger's acceptance status."""
    if status == "PARTIAL":
        return has_row_specific_partial_evidence(identifier, evidence)
    if identifier.startswith("U") and status == "NOT_VERIFIED":
        return evidence in ("See specification v1.0; no ID-specific full test evidence",
                            r"See \| specification v1.0; no ID-specific full test evidence")
    if identifier.startswith("T") and status == "NOT_RUN":
        return evidence == "No complete case-specific T acceptance claimed"
    return False


def extract_ledger(source):
    # Ledger coverage requires real visible, unindented Markdown headings.
    table_u = visible_section(source, "## U01–U48, full coverage (48/48)", "## T01–T54")
    table_t = visible_section(source, "## T01–T54, full coverage (54/54)", "## Scope")
    table_u = rendered_table_rows(
        table_u, ["Requirement ID", "Phase from v1.0 18.2",
                  "Linked test IDs", "Definition", "Implementation",
                  "Full acceptance", "Evidence scope"], "U"
    )
    table_t = rendered_table_rows(
        table_t, ["Test ID", "Related stage(s)",
                  "Linked requirement IDs (coarse)", "Complete acceptance result",
                  "Evidence"], "T"
    )
    u_map, t_map, u_phases, t_phases = {}, {}, {}, {}
    for line in table_u:
        m = re.match(r"^\|\s*(U[0-9]{2})\s*\|",line)
        if not m:
            raise ValueError("unknown/malformed U ledger row: " + line)
        if m:
            cells = split_markdown_cells(line)
            if (len(cells)!=7 or m.group(1) in u_map or cells[3]!="DEFINED"
                or cells[4] not in ("NOT_VERIFIED", "PARTIAL") or cells[5]!="NOT_RUN"):
                raise ValueError("invalid / duplicate U ledger row: "+line)
            if not has_audited_ledger_evidence(m.group(1), cells[4], cells[6]):
                raise ValueError("U ledger evidence contradicts audited status: " + line)
            u_map[m.group(1)] = set(expand_ids(cells[2], "T"))
            u_phases[m.group(1)] = cells[1]
    for line in table_t:
        m = re.match(r"^\|\s*(T[0-9]{2})\s*\|",line)
        if not m:
            raise ValueError("unknown/malformed T ledger row: " + line)
        if m:
            cells = split_markdown_cells(line)
            if len(cells)!=5 or m.group(1) in t_map or cells[3] not in ("NOT_RUN","PARTIAL"):
                raise ValueError("invalid / duplicate T ledger row: "+line)
            if not has_audited_ledger_evidence(m.group(1), cells[3], cells[4]):
                raise ValueError("T ledger evidence contradicts audited status: " + line)
            t_map[m.group(1)] = set(expand_ids(cells[2], "U"))
            t_phases[m.group(1)] = cells[1].split("; ")
    return u_map, t_map, u_phases, t_phases

def validate(spec_text, ledger_text):
    # Direct API callers must use the same recognized Markdown boundaries.
    spec_text = spec_text.replace("\r\n", "\n").replace("\r", "\n")
    ledger_text = ledger_text.replace("\r\n", "\n").replace("\r", "\n")
    validate_definition_tables(spec_text)
    u_map, source_u_phases, source_t_phases = extract_mapping(spec_text)
    ledger_u, ledger_t, ledger_u_phases, ledger_t_phases = extract_ledger(ledger_text)
    if set(ledger_u)!=set(u_map):
        raise ValueError("U ledger row coverage mismatch")
    reverse = {f"T{i:02d}":set() for i in range(1,55)}
    for u,ts in u_map.items():
        if ledger_u[u]!=ts:
            raise ValueError("U test linkage drift: "+u)
        if ledger_u_phases[u] != source_u_phases[u]:
            raise ValueError("U phase drift: "+u)
        for t in ts:
            reverse[t].add(u)
    if set(ledger_t)!=set(reverse):
        raise ValueError("T ledger row coverage mismatch")
    for t,us in reverse.items():
        if ledger_t[t]!=us:
            raise ValueError("T requirement linkage drift: "+t)
        if ledger_t_phases[t] != source_t_phases[t]:
            raise ValueError("T phase drift: "+t)
    return len(u_map),len(reverse)

def must_reject(spec_text, ledger_text):
    try:
        validate(spec_text,ledger_text)
    except ValueError:
        return
    raise AssertionError("damaged U/T mapping incorrectly accepted")

if __name__=="__main__":
    specification = SPEC.read_text(encoding="utf-8")
    ledger = LEDGER.read_text(encoding="utf-8")
    count = validate(specification,ledger)
    # Five normative GFM tables: reject VT/FF/NEL/LS/PS, which are not line endings.
    boundaries = (
        (False, "| ID | 요구사항 | 상태 | 근거 |", "| --- | --- | --- | --- |"),
        (False, "| Test ID | 주요 검증 | 통과 조건 |", "| --- | --- | --- |"),
        (False, "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", "| --- | --- | --- | --- |"),
        (True, "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |", "| --- | --- | --- | --- | --- | --- | --- |"),
        (True, "| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |", "| --- | --- | --- | --- | --- |"),
    )
    for in_ledger, header, delimiter in boundaries:
        original = ledger if in_ledger else specification
        edge1 = header + "\n" + delimiter
        edge2 = delimiter + "\n| "
        if edge1 not in original or edge2 not in original:
            raise AssertionError("missing physical-line fixture: " + header)
        for invalid_separator in ("\x0b", "\x0c", "\x85", "\u2028", "\u2029"):
            for before, after in ((edge1, header + invalid_separator + delimiter),
                                  (edge2, delimiter + invalid_separator + "| ")):
                corrupt = original.replace(before, after, 1)
                if corrupt == original:
                    raise AssertionError("missing separator mutation")
                if in_ledger:
                    must_reject(specification, corrupt)
                else:
                    must_reject(corrupt, ledger)
    # Actual GFM CRLF and CR physical line endings remain accepted.
    for ending in ("\r\n", "\r"):
        if validate(specification.replace("\n", ending),
                    ledger.replace("\n", ending)) != (48, 54):
            raise AssertionError("valid Markdown ending rejected")

    # Evidence assertions MUST agree with their full acceptance state.
    for before, after in (
        ("See specification v1.0; no ID-specific full test evidence",
         "FULL IMPLEMENTATION AND ACCEPTANCE PASS; see #3"),
        ("M0 restricted prototype; see #3",
         "FULL IMPLEMENTATION AND ACCEPTANCE PASS; see #3"),
        ("No complete case-specific T acceptance claimed", "FULL ACCEPTANCE PASS"),
        ("M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified",
         "FULL ACCEPTANCE PASS; see #3; see #6"),
        ("M0 restricted prototype; see #5",
         "M0 restricted prototype; see #5; FULL ACCEPTANCE PASS"),
        ("No complete case-specific T acceptance claimed",
         "No complete case-specific T acceptance claimed; PASS"),
    ):
        if before not in ledger:
            raise AssertionError("missing semantic mutation fixture")
        must_reject(specification, ledger.replace(before, after, 1))
    # Coordinated T44 reassignment must fail despite complete reciprocal
    # ledger coverage and accurate per-stage metadata.
    reassigned_spec = specification.replace(
        "| U01 | 8,10,11 | T06,T07,T43,T44 | M2 |",
        "| U01 | 8,10,11 | T06,T07,T43 | M2 |", 1
    ).replace(
        "| U47 | 13.1,15 | T06,T10,T12,T41,T50 | 단계별 게이트 |",
        "| U47 | 13.1,15 | T06,T10,T12,T41,T44,T50 | 단계별 게이트 |", 1
    )
    reassigned_ledger = ledger.replace(
        "| U01 | M2 | T06,T07,T43,T44 |",
        "| U01 | M2 | T06,T07,T43 |", 1
    ).replace(
        "| U47 | 단계별 게이트 | T06,T10,T12,T41,T50 |",
        "| U47 | 단계별 게이트 | T06,T10,T12,T41,T44,T50 |", 1
    ).replace(
        "| T44 | M2 | U01 |",
        "| T44 | 단계별 게이트 | U47 |", 1
    )
    if reassigned_spec == specification or reassigned_ledger == ledger:
        raise AssertionError("T44 reassignment fixture not constructed")
    must_reject(reassigned_spec, reassigned_ledger)

    # An unrelated U28/T01 cannot borrow an M0 prototype Issue to claim PARTIAL.
    for identifier in ("U28", "T01"):
        old = next(line for line in ledger.split("\n")
                   if line.startswith("| " + identifier + " |"))
        if identifier.startswith("U"):
            bad = old.replace("NOT_VERIFIED", "PARTIAL").replace(
                "See specification v1.0; no ID-specific full test evidence",
                "M0 restricted prototype; see #3")
        else:
            bad = old.replace("NOT_RUN", "PARTIAL").replace(
                "No complete case-specific T acceptance claimed",
                "M0 native Qt/Worker prototype only; see #3; full T unqualified")
        if bad == old or "PARTIAL" not in bad:
            raise AssertionError("PARTIAL scope mutation not constructed: " + identifier)
        must_reject(specification, ledger.replace(old, bad, 1))
    # Even currently permitted PARTIAL rows must reject irrelevant issue evidence.
    must_reject(specification, ledger.replace(
        "M0 restricted prototype; see #4",
        "M0 restricted prototype; see #3", 1))

    # Prove the exact prior T44 omission cannot silently recur.
    must_reject(specification.replace("T06,T07,T43,T44", "T06,T07,T43"),ledger)
    must_reject(specification,ledger.replace("| T44 |","| T99 |"))
    must_reject(specification,ledger.replace("| U01 |","| U99 |"))
    must_reject(specification,ledger.replace("| U01 | M2 |", "| U01 | WRONG_PHASE |"))
    must_reject(specification,ledger.replace("| T44 | M2 |", "| T44 | WRONG_PHASE |"))
    missing_t44_definition = re.sub(r'(?m)^\| T44 \|[^\n]*\n', '', specification)
    missing_u31_definition = re.sub(r'(?m)^\| U31 \|[^\n]*\n', '', specification)
    if missing_t44_definition == specification or missing_u31_definition == specification:
        raise AssertionError("expected definition fixture not found")
    must_reject(missing_t44_definition, ledger)
    must_reject(missing_u31_definition, ledger)
    extra_t100 = specification.replace("| T54 |", "| T100 | Extra unapproved test | Must not be accepted |\n| T54 |")
    extra_u100 = specification.replace("| U48 |", "| U100 | Extra unapproved requirement | 확정 | D1 |\n| U48 |")
    must_reject(extra_t100, ledger)
    must_reject(extra_u100, ledger)
    indented_t100 = specification.replace("| T54 |", "  | T100 | Indented rogue test | Reject |\n| T54 |")
    indented_u100 = specification.replace("| U48 |", "   | U100 | Indented rogue requirement | 확정 | D1 |\n| U48 |")
    must_reject(indented_t100, ledger)
    must_reject(indented_u100, ledger)
    openerless_t100 = specification.replace("| T54 |", "T100 | Unapproved test without opening pipe | Reject |\n| T54 |")
    openerless_u100 = specification.replace("| U48 |", "U100 | Unapproved requirement without opening pipe | 확정 | D1 |\n| U48 |")
    must_reject(openerless_t100, ledger)
    must_reject(openerless_u100, ledger)
    # A styled unknown first cell must be inspected, never skipped.
    bold_t100 = specification.replace("| T54 |", "| **T100** | Extra bold test | Reject |\n| T54 |")
    bold_u100 = specification.replace("| U48 |", "| **U100** | Extra bold requirement | 확정 | D1 |\n| U48 |")
    must_reject(bold_t100, ledger)
    must_reject(bold_u100, ledger)
    # The otherwise valid openerless test-table header must still be accepted.
    openerless_header = specification.replace(
        "| Test ID | 주요 검증 | 통과 조건 |",
        "Test ID | 주요 검증 | 통과 조건 |",
        1,
    )
    if openerless_header == specification:
        raise AssertionError("T header fixture missing")
    validate(openerless_header, ledger)
    # Comment removal must never create any of the five normative headers.
    # Exercise the same GitHub review P2 repro with openerless headers, plus
    # pipe-leading variants and the distinct specification/ledger tables.
    for header, is_ledger in (
        ("| ID | 요구사항 | 상태 | 근거 |", False),
        ("| Test ID | 주요 검증 | 통과 조건 |", False),
        ("| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", False),
        ("| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |", True),
        ("| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |", True),
    ):
        original_document = ledger if is_ledger else specification
        if original_document.count(header) != 1:
            raise AssertionError("normative header fixture missing/ambiguous: " + header)
        for comment_prefix in ("<!-- -->", "<!-- hidden -->"):
            # A leading HTML comment followed by a header without the
            # opening pipe is not an original GFM table header.
            altered = original_document.replace(header, comment_prefix + header[2:], 1)
            if is_ledger:
                must_reject(specification, altered)
            else:
                must_reject(altered, ledger)
            # A pipe-leading row with comment insertion is also rejected.
            altered = original_document.replace(header, "| " + comment_prefix + header[2:], 1)
            if is_ledger:
                must_reject(specification, altered)
            else:
                must_reject(altered, ledger)
    # IDs in code fences/comments are not part of the rendered definition table.
    row_t54 = re.search(r"(?m)^\| T54 \|[^\n]*$", specification)
    if not row_t54:
        raise AssertionError("T54 test fixture not found")
    t54 = row_t54.group(0)
    commented_t54 = specification.replace(t54, "<!-- " + t54 + " -->", 1)
    fenced_t54 = specification.replace(t54, chr(96) * 3 + "\n" + t54 + "\n" + chr(96) * 3, 1)
    must_reject(commented_t54, ledger)
    must_reject(fenced_t54, ledger)
    for old, new in (
        ("| Test ID | 주요 검증 | 통과 조건 |\n| --- | --- | --- |",
         "| Test ID | 주요 검증 | 통과 조건 |"),
        ("| ID | 요구사항 | 상태 | 근거 |\n| --- | --- | --- | --- |",
         "| ID | 요구사항 | 상태 | 근거 |"),
    ):
        if old not in specification:
            raise AssertionError("definition header/delimiter fixture missing")
        must_reject(specification.replace(old, new, 1), ledger)
    escaped_pipe = specification.replace(
        "| T44 | 불균등 격자·부분 Export·한글 Net |",
        r"| T44 | 불균등 격자 \| 부분 Export·한글 Net |",
        1,
    )
    if escaped_pipe == specification:
        raise AssertionError("escaped pipe fixture missing")
    validate(escaped_pipe, ledger)
    # Four-space indentation converts the entire table to rendered code.
    for header, delim, prefix in (
        ("| Test ID | 주요 검증 | 통과 조건 |", "| --- | --- | --- |", "T"),
        ("| ID | 요구사항 | 상태 | 근거 |", "| --- | --- | --- | --- |", "U"),
    ):
        begin = specification.find(header)
        if begin == -1:
            raise AssertionError(prefix + " indented table fixture missing")
        rows = specification[begin:].split("\n")
        table_rows = []
        for line in rows:
            if not table_rows or line.startswith("|"):
                table_rows.append(line)
            else:
                break
        if len(table_rows) < 3:
            raise AssertionError("missing table rows")
        block = "\n".join(table_rows)
        indented_block = "\n".join("    " + line for line in table_rows)
        must_reject(specification.replace(block, indented_block, 1), ledger)
    # Starting an HTML comment before the section heading hides the table.
    marker_t = "## 13 인수시험 및 독립 교차 검증"
    closing_t = "### 13.1 정량 인수 목표와 측정 조건"
    hidden_test_section = specification.replace(marker_t, "<!--\n" + marker_t, 1)
    hidden_test_section = hidden_test_section.replace(closing_t, "-->\n" + closing_t, 1)
    must_reject(hidden_test_section, ledger)
    # CommonMark tab-expansion code indentation and raw HTML pre blocks must not pass.
    full_t_header = "| Test ID | 주요 검증 | 통과 조건 |"
    space_tab_table = specification.replace(
        full_t_header,
        " \t" + full_t_header,
        1,
    )
    # This isolated header is no longer a rendered table header.
    must_reject(space_tab_table, ledger)
    html_pre = specification.replace(
        "## 13 인수시험 및 독립 교차 검증",
        "<pre>\n## 13 인수시험 및 독립 교차 검증",
        1,
    ).replace(
        "### 13.1 정량 인수 목표와 측정 조건",
        "</pre>\n### 13.1 정량 인수 목표와 측정 조건",
        1,
    )
    must_reject(html_pre, ledger)








    # Fail-closed: all raw HTML block tags, not just the four type-1 tags.
    for tag in ("pre", "script", "style", "textarea", "table", "div", "details", "section", "custom-element"):
        html_wrapped = specification.replace(
            "## 13 인수시험 및 독립 교차 검증",
            "<" + tag + ">\n## 13 인수시험 및 독립 교차 검증",
            1,
        ).replace(
            "### 13.1 정량 인수 목표와 측정 조건",
            "</" + tag + ">\n### 13.1 정량 인수 목표와 측정 조건",
            1,
        )
        must_reject(html_wrapped, ledger)
    # HTML-looking literals in fences, comments and inline code are harmless.
    for literal in (
        "&#96;&#96;&#96;\n<pre>\n&#96;&#96;&#96;",
        "<!-- <pre> -->",
        "&#96;<pre>&#96;",
    ):
        # Build backticks without encoding ambiguity in this test source.
        literal = literal.replace("&#96;", chr(96))
        validate(specification.replace(
            "## 13 인수시험 및 독립 교차 검증",
            literal + "\n## 13 인수시험 및 독립 교차 검증", 1), ledger)

    # A comment-looking opener in inline/indented code is literal text.
    for literal in (chr(96) + "<!--" + chr(96), "    <!--"):
        validate(specification.replace(
            "## 13 인수시험 및 독립 교차 검증",
            literal + "\n## 13 인수시험 및 독립 교차 검증",
            1,
        ), ledger)

    # Definition, crosswalk AND audit-ledger ID rows must be rendered rows.
    crosswalk_u01 = "| U01 | 8,10,11 | T06,T07,T43,T44 | M2 |"
    ledger_u01 = "| U01 | M2 | T06,T07,T43,T44 |"
    ledger_t44 = "| T44 | M2 |"
    if crosswalk_u01 not in specification or ledger_u01 not in ledger or ledger_t44 not in ledger:
        raise AssertionError("rendered crosswalk/ledger fixture missing")
    for opening, closing in (("<!--\n", "\n-->"), (chr(96)*3 + "\n", "\n" + chr(96)*3)):
        must_reject(specification.replace(
            crosswalk_u01, opening + crosswalk_u01 + closing, 1
        ), ledger)
        for target in (ledger_u01, ledger_t44):
            row = next(line for line in ledger.split("\n") if line.startswith(target))
            must_reject(specification, ledger.replace(
                row, opening + row + closing, 1
            ))

    # A pipe-separated list without a Markdown delimiter is NOT a table.
    spec_delimiter = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |\n| --- | --- | --- | --- |"
    if spec_delimiter not in specification:
        raise AssertionError("crosswalk table fixture missing")
    must_reject(specification.replace(
        spec_delimiter, "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 1
    ), ledger)
    for header, delimiter in (
        ("| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |",
         "| --- | --- | --- | --- | --- | --- | --- |"),
        ("| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |",
         "| --- | --- | --- | --- | --- |"),
    ):
        block = header + "\n" + delimiter
        if block not in ledger:
            raise AssertionError("ledger table delimiter fixture missing")
        must_reject(specification, ledger.replace(block, header, 1))

    # Escaped '<' must never open an HTML comment and conceal real HTML
    # block containers surrounding a normative table. An even number of
    # backslashes leaves the '<' unescaped; odd count remains literal text.
    escaped_open = chr(92) + "<!--"
    doubled_open = chr(92) * 2 + "<!--"
    for table in (
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        "| ID | 요구사항 | 상태 | 근거 |",
    ):
        if table not in specification:
            raise AssertionError("escaped-comment fixture missing: " + table)
        for open_mark in (escaped_open, chr(92) * 3 + "<!--"):
            hidden_html = open_mark + "\n\n<style>\n-->\n" + table
            must_reject(specification.replace(table, hidden_html, 1), ledger)
        # A genuine opener (even slash parity) must still mask its comment
        # contents; a table inside one cannot satisfy normative coverage.
        hidden_table = doubled_open + "\n" + table + "\n-->"
        must_reject(specification.replace(table, hidden_table, 1), ledger)

    # Frozen U/T IDs use ASCII decimal digits only. Python's Unicode
    # decimal digits would otherwise normalize look-alike identifiers.
    for old, bad in (
        ("| U01 | 8,10,11 |", "| U٠١ | 8,10,11 |"),
        ("| T06,T07,T43,T44 |", "| T٠٦,T07,T43,T44 |"),
        ("| U01 | 8,10,11 |", "| U０１ | 8,10,11 |"),
    ):
        if old not in specification:
            raise AssertionError("Unicode ID fixture missing: " + old)
        must_reject(specification.replace(old, bad, 1), ledger)
    for prefix, original in (("U", "U01"), ("T", "T06")):
        for non_ascii in ("٠", "０"):
            rejected_id = original.replace("0", non_ascii, 1)
            try:
                expand_ids(rejected_id, prefix)
            except ValueError:
                pass
            else:
                raise AssertionError("non-ASCII U/T ID accepted: " + rejected_id)

    # GFM validates delimiter cells against raw source before stripping
    # inline HTML comments. Do not accept a delimiter synthesized as valid.
    for doc_name, doc in (("spec", specification), ("ledger", ledger)):
        tables = (
            ("| ID | 요구사항 | 상태 | 근거 |", "| --- | --- | --- | --- |"),
            ("| Test ID | 주요 검증 | 통과 조건 |", "| --- | --- | --- |"),
            ("| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", "| --- | --- | --- | --- |"),
        ) if doc_name == "spec" else (
            ("| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |",
             "| --- | --- | --- | --- | --- | --- | --- |"),
            ("| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |",
             "| --- | --- | --- | --- | --- |"),
        )
        for header, delimiter in tables:
            original = header + "\n" + delimiter
            if original not in doc:
                raise AssertionError("original delimiter guard fixture missing: " + doc_name)
            # Left, middle and final cells must each be checked against the
            # original GFM delimiter, not an HTML-comment-stripped variant.
            for position in (0, len(split_markdown_cells(delimiter)) // 2,
                             len(split_markdown_cells(delimiter)) - 1):
                cells = split_markdown_cells(delimiter)
                cells[position] = "<!-- -->" + cells[position]
                bad = "| " + " | ".join(cells) + " |"
                damaged = doc.replace(original, header + "\n" + bad, 1)
                if doc_name == "spec":
                    must_reject(damaged, ledger)
                else:
                    must_reject(specification, damaged)

    # CommonMark type-6 HTML blocks start on '<div' even without a '>'.
    for doc, row in (
        (specification, "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"),
        (ledger, "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs"),
    ):
        if row not in doc:
            raise AssertionError("raw HTML partial opener fixture missing")
        changed = doc.replace(row, "<div\n" + row, 1)
        if doc is specification:
            must_reject(changed, ledger)
        else:
            must_reject(specification, changed)
    # Extra visible ledger IDs are not silently discarded.
    must_reject(specification, ledger.replace(
        "| U48 |", "| U100 | M0 | T13 | DEFINED | NOT_VERIFIED | NOT_RUN | Unknown |\n| U48 |", 1))
    must_reject(specification, ledger.replace(
        "| T54 |", "| T100 | M0 | U01 | NOT_RUN | Unknown |\n| T54 |", 1))
    # Escaped Markdown pipes in prose are legal data, not new cells.
    escaped_u = ledger.replace(
        "See specification v1.0; no ID-specific full test evidence",
        r"See \| specification v1.0; no ID-specific full test evidence", 1)
    if escaped_u == ledger:
        raise AssertionError("escaped ledger fixture missing")
    validate(specification, escaped_u)
    escaped_crosswalk = specification.replace(
        "| U01 | 8,10,11 |", r"| U01 | 8 \| 10,11 |", 1)
    if escaped_crosswalk == specification:
        raise AssertionError("escaped crosswalk fixture missing")
    # Body refs are strictly numbered headings; an escaped pipe is NOT a section ref.
    must_reject(escaped_crosswalk, ledger)

    # Raw HTML style container tokens may occur in inline prose before a table.
    for name, doc, token in (
        ("spec", specification, "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"),
        ("ledger", ledger, "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs"),
    ):
        if token not in doc:
            raise AssertionError("inline style attack fixture missing")
        modified = doc.replace(token, "prefix <style>\\n" + token, 1)
        if name == "spec":
            must_reject(modified, ledger)
        else:
            must_reject(specification, modified)
    # No unsupported implementation/acceptance assertions in a ledger.
    if "| DEFINED | NOT_VERIFIED | NOT_RUN |" not in ledger:
        raise AssertionError("implementation status fixture missing")
    must_reject(specification, ledger.replace(
        "| DEFINED | NOT_VERIFIED | NOT_RUN |",
        "| DEFINED | IMPLEMENTED | NOT_RUN |", 1))
    must_reject(specification, ledger.replace(
        "| DEFINED | NOT_VERIFIED | NOT_RUN |",
        "| DEFINED | PASS | NOT_RUN |", 1))

    # All normative section markers must be real Markdown heading lines.
    must_reject(specification.replace(
        "## 2 요구사항 추적표", "x ## 2 요구사항 추적표", 1), ledger)
    must_reject(specification.replace(
        "### 18.2 요구사항과 시험 연결",
        "x ### 18.2 요구사항과 시험 연결", 1), ledger)
    must_reject(specification, ledger.replace(
        "## U01–U48, full coverage (48/48)",
        "x ## U01–U48, full coverage (48/48)", 1))
    # Crosswalk source-body references are required.
    must_reject(specification.replace(
        "| U01 | 8,10,11 | T06,T07,T43,T44 |",
        "| U01 |  | T06,T07,T43,T44 |", 1), ledger)
    # PARTIAL is an evidence-linked claim, not a label.
    must_reject(specification, ledger.replace(
        "| DEFINED | PARTIAL | NOT_RUN | M0 restricted prototype; see #3 |",
        "| DEFINED | PARTIAL | NOT_RUN |  |", 1))
    must_reject(specification, ledger.replace(
        "| PARTIAL | M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified |",
        "| PARTIAL |  |", 1))
    # Attribute-bearing raw HTML text containers cannot hide tables.
    for tag in ('<style type="text/css">', '</style >'):
        modified = specification.replace(
            "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
            "prefix " + tag + "\n| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 1)
        must_reject(modified, ledger)

    # Markdown list nesting cannot masquerade as canonical table rows.
    source_header = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
    for indent in ("  ", "   "):
        section = specification.split("### 18.2 요구사항과 시험 연결", 1)[1].split("### 18.3", 1)[0]
        table = [line for line in section.split("\n") if line.startswith("|")]
        modified = specification.replace(
            source_header, "- <div>\n" + indent + source_header, 1)
        # The delimiter is indented as well, creating a Markdown list block.
        modified = modified.replace(
            source_header + "\n| --- | --- | --- | --- |",
            source_header + "\n" + indent + "| --- | --- | --- | --- |", 1)
        must_reject(modified, ledger)
    # Both malformed and nonexistent section/body links are invalid.
    for invalid_ref in ("nonsense", "999"):
        must_reject(specification.replace(
            "| U01 | 8,10,11 |", "| U01 | " + invalid_ref + " |", 1), ledger)

    # Escaped backticks do NOT create inline code; hidden raw pre is rejected.
    escaped_tick = chr(92) + chr(96)
    must_reject(specification.replace(
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        escaped_tick + "<pre>" + escaped_tick + "\n| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 1
    ), ledger)
    # Repeated/overlapping IDs must not silently collapse into sets.
    must_reject(specification.replace(
        "T06,T07,T43,T44", "T06,T06,T07,T43,T44", 1
    ), ledger)
    must_reject(specification, ledger.replace(
        "T06,T07,T43,T44", "T06,T06,T07,T43,T44", 1
    ))
    must_reject(specification, ledger.replace(
        "| T44 | M2 | U01", "| T44 | M2 | U01,U01", 1
    ))

    # A preceding blockquote paragraph consumes lazy continuation table lines.
    for header in (
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        "| ID | 요구사항 | 상태 | 근거 |",
    ):
        if header not in specification:
            raise AssertionError("blockquote boundary fixture missing")
        must_reject(specification.replace(
            header, "> prefix\n" + header, 1), ledger)
    ledger_header = "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs"
    if ledger_header not in ledger:
        raise AssertionError("ledger blockquote boundary fixture missing")
    must_reject(specification, ledger.replace(
        ledger_header, "> prefix\n" + ledger_header, 1))

    # Any real inline raw HTML tag could make normative table invisible.
    # This specification's table sections intentionally permit no raw HTML.
    for tag in ("template", "noscript", "div", "svg", "iframe"):
        for target in (
            "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
            "| ID | 요구사항 | 상태 | 근거 |",
        ):
            must_reject(specification.replace(
                target, "prefix <" + tag + ">\n\n" + target, 1), ledger)
        must_reject(specification, ledger.replace(
            "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs",
            "prefix <" + tag + ">\n\n| Requirement ID | Phase from v1.0 18.2 | Linked test IDs", 1))

    # Within an active code span backslashes do not escape closing ticks.
    # Here the <template> becomes visible raw HTML, not protected code.
    broken_span = chr(96) + "x" + chr(92) + chr(96) + "<template>" + chr(96)
    must_reject(specification.replace(
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        broken_span + "\n| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 1
    ), ledger)

    # Even NOT_VERIFIED/NOT_RUN rows must disclose evidence *scope*.
    for target, replacement in (
        ("| U01 | M2 | T06,T07,T43,T44 | DEFINED | NOT_VERIFIED | NOT_RUN | See specification v1.0; no ID-specific full test evidence |",
         "| U01 | M2 | T06,T07,T43,T44 | DEFINED | NOT_VERIFIED | NOT_RUN |  |"),
        ("| T01 |", "| T01 |"),
    ):
        if target.startswith("| U01"):
            if target not in ledger:
                raise AssertionError("missing U01 evidence regression source")
            must_reject(specification, ledger.replace(target, replacement, 1))
        else:
            row = next((line for line in ledger.split("\n") if line.startswith("| T01 |")), None)
            if row is None:
                raise AssertionError("missing T01 evidence regression source")
            cells = split_markdown_cells(row)
            cells[4] = ""
            must_reject(specification, ledger.replace(row, "| " + " | ".join(cells) + " |", 1))

    # A token that merely says "see #" is not a resolvable evidence link.
    must_reject(specification, ledger.replace(
        "M0 restricted prototype; see #3",
        "M0 restricted prototype; see #bogus", 1))
    must_reject(specification, ledger.replace(
        "M0 restricted prototype; see #3",
        "M0 restricted prototype; see #99999", 1))
    must_reject(specification, ledger.replace(
        "M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified",
        "M0 native Qt/Worker prototype only; see #bogus; full T unqualified", 1))

    # Openerless Markdown table rows are visible rows, not a table boundary.
    spec_tail = next(line for line in specification.split("\n") if line.startswith("| U47 |"))
    must_reject(specification.replace(
        spec_tail, spec_tail + "\nU100 | 1 | T01 | M0 |", 1), ledger)
    ledger_tail_u = next(line for line in ledger.split("\n") if line.startswith("| U48 |"))
    must_reject(specification, ledger.replace(
        ledger_tail_u, ledger_tail_u + "\nU100 | M0 | T01 | DEFINED | NOT_VERIFIED | NOT_RUN | Bogus |", 1))
    ledger_tail_t = next(line for line in ledger.split("\n") if line.startswith("| T54 |"))
    must_reject(specification, ledger.replace(
        ledger_tail_t, ledger_tail_t + "\nT100 | M0 | U01 | NOT_RUN | Bogus |", 1))

    # Literal issue tokens are not resolvable evidence links.
    for fake in (chr(96) + "#3" + chr(96), chr(92) + "#3"):
        must_reject(specification, ledger.replace(
            "M0 restricted prototype; see #3",
            "M0 restricted prototype; see " + fake, 1))
        must_reject(specification, ledger.replace(
            "M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified",
            "M0 native Qt/Worker prototype only; see " + fake, 1))

    # Arbitrary URL fragments are not BoardSI Issue evidence autolinks.
    for fake in ("https://example.invalid/#3", "https://example.invalid/path?issue=#3"):
        must_reject(specification, ledger.replace(
            "M0 restricted prototype; see #3",
            "M0 restricted prototype; see " + fake, 1))
        must_reject(specification, ledger.replace(
            "M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified",
            "M0 native Qt/Worker prototype only; see " + fake, 1))

    # Explicit Markdown links can display a valid issue label while
    # navigating to a foreign destination; do not count their labels.
    for fake in ("[see #3](https://example.invalid)", "[see #3][external]"):
        must_reject(specification, ledger.replace(
            "M0 restricted prototype; see #3",
            "M0 restricted prototype; " + fake, 1))
        must_reject(specification, ledger.replace(
            "M0 native Qt/Worker prototype only; see #3; see #6; full T unqualified",
            "M0 native Qt/Worker prototype only; " + fake, 1))

    # Literal HTML-comment-like tokens inside paired inline code are not
    # comments and must not be removed before balancing backticks.
    for literal_code in ("`<!-- -->`", "``<!-- -->``"):
        intro = "### 18.2 요구사항과 시험 연결"
        unaffected = specification.replace(
            intro, literal_code + "\n\n" + intro, 1)
        if validate(unaffected, ledger) != (48, 54):
            raise AssertionError("valid comment-like inline code rejected")

    # GFM emphasis delimiters cannot consume adjacent inner whitespace.
    for invalid_markup in ("** U01 **", "* U01 *", "__ U01 __"):
        bad_spec = specification.replace(
            "| U01 |", "| " + invalid_markup + " |", 1)
        if bad_spec == specification:
            raise AssertionError("emphasis fixture absent")
        must_reject(bad_spec, ledger)

    # Markup inside an inline code span is literal, not recursively
    # interpreted emphasis. It must not silently resolve to a valid ID.
    for marker in ("`*U01*`", "`**U01**`", "**`U01`**"):
        bad_spec = specification.replace("| U01 |", "| " + marker + " |", 1)
        if bad_spec == specification:
            raise AssertionError("code-span ID mutation not constructed")
        must_reject(bad_spec, ledger)
    # Duplicate chapter links must reject even when the set of links
    # matches the frozen U01 reference set.
    must_reject(specification.replace(
        "| U01 | 8,10,11 | T06,T07,T43,T44 | M2 |",
        "| U01 | 8,8,10,11 | T06,T07,T43,T44 | M2 |", 1), ledger)

    # A single-line HTML comment may contain unmatched backticks: these
    # are invisible to rendered Markdown and cannot begin inline code.
    for comment_line in ("<!-- reference ` inside comment -->", "<!-- double `` here -->"):
        crosswalk_header = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
        normal_spec = specification.replace(
            crosswalk_header, comment_line + "\n" + crosswalk_header, 1)
        if validate(normal_spec, ledger) != (48, 54):
            raise AssertionError("valid HTML comment unexpectedly rejected")
        ledger_header = "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs"
        normal_ledger = ledger.replace(
            ledger_header, comment_line + "\n" + ledger_header, 1)
        if validate(specification, normal_ledger) != (48, 54):
            raise AssertionError("valid ledger HTML comment unexpectedly rejected")

    # A requirement's chapter refs cannot be reassigned to an unrelated but
    # valid engineering chapter while preserving syntactic coverage.
    must_reject(specification.replace(
        "| U01 | 8,10,11 | T06,T07,T43,T44 | M2 |",
        "| U01 | 3 | T06,T07,T43,T44 | M2 |", 1), ledger)

    # GFM permits multi-line inline code, but this controlled normative
    # format rejects it to avoid disguising real HTML template containers.
    crosswalk_first = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
    crosswalk_last = "| U47 | 13.1,15 | T06,T10,T12,T41,T50 | 단계별 게이트 |"
    for width in (1, 2):
        tick = chr(96) * width
        template_open = tick + "<!--\n" + tick + "\n<template>\n-->\n"
        template_close = tick + "<!--\n" + tick + "\n</template>\n-->\n"
        concealed = specification.replace(
            crosswalk_first, template_open + crosswalk_first, 1
        ).replace(
            crosswalk_last, crosswalk_last + "\n" + template_close, 1
        )
        must_reject(concealed, ledger)

    # Source-level block markers are normative: comments cannot synthesize
    # a table header/delimiter, a definition row or section heading.
    for marker in (
        "### 18.2 요구사항과 시험 연결",
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        "| U01 | 8,10,11 | T06,T07,T43,T44 | M2 |",
    ):
        if marker not in specification:
            raise AssertionError("comment-block regression source missing: " + marker)
        must_reject(specification.replace(
            marker, "<!-- -->" + marker, 1), ledger)
    for marker in (
        "## U01–U48, full coverage (48/48)",
        "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs",
        "| U01 | M2 | T06,T07,T43,T44 | DEFINED | NOT_VERIFIED | NOT_RUN |",
    ):
        if marker not in ledger:
            raise AssertionError("comment-ledger regression source missing: " + marker)
        must_reject(specification, ledger.replace(
            marker, "<!-- -->" + marker, 1))

    # A backtick hidden by a removable HTML comment still makes the
    # ORIGINAL backtick-fence info string invalid in GFM. The old parser
    # treated this as an opener, then the next real opener as its closer,
    # falsely accepting a crosswalk hidden from the actual renderer.
    invalid_original_opener = chr(96) * 3 + "<!-- " + chr(96) + " -->"
    start_crosswalk = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
    end_crosswalk = "| U47 | 13.1,15 | T06,T10,T12,T41,T50 | 단계별 게이트 |"
    if start_crosswalk not in specification or end_crosswalk not in specification:
        raise AssertionError("original fence-info regression fixture missing")
    false_rendered_crosswalk = specification.replace(
        start_crosswalk,
        invalid_original_opener + "\n" + chr(96) * 3 + "\n" + start_crosswalk, 1
    ).replace(
        end_crosswalk,
        end_crosswalk + "\n" + chr(96) * 3
        + "\n<template>\n" + chr(96) * 3, 1
    )
    must_reject(false_rendered_crosswalk, ledger)

    # Invalid GFM backtick-fence info strings must not hide raw HTML.
    bogus_fence = chr(96)*3 + "tag" + chr(96) + "info"
    must_reject(specification.replace(
        "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
        bogus_fence + "\n<template>\n| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 1), ledger)
    must_reject(specification, ledger.replace(
        "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs",
        bogus_fence + "\n<template>\n| Requirement ID | Phase from v1.0 18.2 | Linked test IDs", 1))

    # Removing HTML comments must never CREATE a fenced block.
    # In GFM the source opener below is not at the beginning of a block;
    # a false fence would hide the raw HTML template around the table.
    for ch in (chr(96), "~"):
        false_fence = "<!-- -->" + ch * 3
        for target in (
            "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |",
            "| ID | 요구사항 | 상태 | 근거 |",
        ):
            must_reject(specification.replace(
                target, false_fence + "\n<template>\n" + target, 1
            ), ledger)
        must_reject(specification, ledger.replace(
            "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs",
            false_fence + "\n<template>\n| Requirement ID | Phase from v1.0 18.2 | Linked test IDs", 1))

    # Only ASCII space/tab can follow a real fenced-code closer (GFM).
    # NBSP must not prematurely close a fence and expose hidden ID tables.
    fence_header = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
    bad_close = chr(96) * 3 + chr(160)
    must_reject(specification.replace(
        fence_header,
        chr(96) * 3 + "\n" + bad_close + "\n" + fence_header + "\n"
        + chr(96) * 3 + "\n" + chr(96) * 3, 1
    ), ledger)
    # Crosswalk/appendix sections are not eligible requirement-body targets.
    for non_body in ("18.2", "18.3"):
        must_reject(specification.replace(
            "| U01 | 8,10,11 |", "| U01 | " + non_body + " |", 1
        ), ledger)

    # Change log, review summary and crosswalk sections are NOT requirement bodies.
    for invalid_ref in ("2", "16", "17", "18", "18.1", "18.3"):
        must_reject(specification.replace(
            "| U01 | 8,10,11 |", "| U01 | " + invalid_ref + " |", 1), ledger)

    # GFM treats HTML-looking tokens in a valid fenced-code INFO string
    # as inert text. Scan resumes after the real closing fence, so a
    # subsequent raw HTML style block cannot conceal normative tables.
    crosswalk_header = "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |"
    crosswalk_last = "| U47 | 13.1,15 | T06,T10,T12,T41,T50 | 단계별 게이트 |"
    opening_info_fence = chr(96) * 3 + "x <!--\n" + chr(96) * 3
    harmless_info_fence = specification.replace(
        crosswalk_header,
        opening_info_fence + "\n\n" + crosswalk_header, 1
    )
    if harmless_info_fence == specification:
        raise AssertionError("GFM fence info fixture not constructed")
    if validate(harmless_info_fence, ledger) != (48, 54):
        raise AssertionError("valid source GFM fence info rejected")
    hidden_by_style = specification.replace(
        crosswalk_header,
        opening_info_fence + "\n<style>\n-->\n" + crosswalk_header, 1
    ).replace(
        crosswalk_last,
        crosswalk_last + "\n" + opening_info_fence
        + "\n</style>\n-->", 1
    )
    must_reject(hidden_by_style, ledger)

    # GFM validates both pipe-leading AND openerless table delimiters
    # against ORIGINAL Markdown before removing an HTML comment.
    # An openerless delimiter fabricated by stripping comments is invalid.
    for original_source, h, dash_count, is_ledger in (
        (specification, "| ID | 요구사항 | 상태 | 근거 |", 4, False),
        (specification, "| Test ID | 주요 검증 | 통과 조건 |", 3, False),
        (specification, "| 요구 ID | 본문 | 주요 시험 | 완료 단계 |", 4, False),
        (ledger, "| Requirement ID | Phase from v1.0 18.2 | Linked test IDs | Definition | Implementation | Full acceptance | Evidence scope |", 7, True),
        (ledger, "| Test ID | Related stage(s) | Linked requirement IDs (coarse) | Complete acceptance result | Evidence |", 5, True),
    ):
        normal_delimiter = "| " + " | ".join(["---"] * dash_count) + " |"
        marker = h + "\n" + normal_delimiter
        disguised = "<!-- -->" + " | ".join(["---"] * dash_count)
        if marker not in original_source:
            raise AssertionError("openerless delimiter fixture not constructed: " + h)
        corrupt = original_source.replace(marker, h + "\n" + disguised, 1)
        if is_ledger:
            must_reject(specification, corrupt)
        else:
            must_reject(corrupt, ledger)

    print(f"M0 U/T traceability coverage PASS: {count[0]} requirements, {count[1]} tests; mutation guards PASS; no system T acceptance implied")
