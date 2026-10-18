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
        match = re.fullmatch(rf"{prefix}(\d{{2}})(?:~(?:{prefix})?(\d{{2}}))?", item.strip())
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
    for original_line in source.splitlines():
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
        if not in_comment and not real_fence_start and unclosed_inline_code_run(original_line):
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
                if start == -1:
                    visible.append(original_line[index:])
                    index = len(original_line)
                else:
                    visible.append(original_line[index:start])
                    in_comment = True
                    index = start + 4
        text = "".join(visible)
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
        visible = section.splitlines()
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
                while len(item) >= 2 and (
                    (item.startswith("**") and item.endswith("**"))
                    or (item.startswith("__") and item.endswith("__"))
                ):
                    item = item[2:-2].strip()
                while len(item) >= 2 and item[0] == item[-1] and item[0] in ("*", "_", chr(96)):
                    item = item[1:-1].strip()
                if not re.fullmatch(rf"{prefix}\d{{2}}", item):
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
    lines = section.splitlines()
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
            if re.match(r"^\s*\|\s*" + prefix + r"\d", line):
                raise ValueError("detached " + prefix + " ID row outside table")
    return rows


def extract_mapping(source):
    # Only a real rendered Markdown heading selects the public crosswalk.
    section = visible_section(source, "### 18.2 요구사항과 시험 연결", "### 18.3")
    mapping, u_phases, t_phases = {}, {}, {}
    # A body reference is a real section number, not free-form prose.
    section_numbers = set(re.findall(
        r"(?m)^#{2,5} +(\d+(?:\.\d+)*)\s+",
        "\n".join(visible_markdown_lines(source))
        .split("### 18.2 요구사항과 시험 연결", 1)[0],
    ))
    for line in rendered_table_rows(
        section, ["요구 ID", "본문", "주요 시험", "완료 단계"], "U"
    ):
        if not re.match(r"\|\s*U\d{2}", line):
            raise ValueError("non-U crosswalk row: " + line)
        cells = split_markdown_cells(line)
        if len(cells) != 4 or any(not cell.strip() for cell in cells):
            raise ValueError("bad/empty section 18.2 row")
        refs = [ref.strip() for ref in cells[1].split(",")]
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
    # The audited ledger uses explicit "see #N" issue autolinks only.
    # Reject Markdown links, inline-code decoys, unknown IDs and evidence
    # borrowed from another row's unrelated engineering boundary.
    if "[" in value or "]" in value:
        return False
    visible = mask_inline_code_for_comment_scan(value)
    references = {
        int(number) for number in
        re.findall(r"(?i)(?<!\w)see\s+#([1-9][0-9]*)(?!\w)", visible)
    }
    return references and references == PARTIAL_EVIDENCE_BY_ID.get(identifier)


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
        m = re.match(r"^\|\s*(U\d{2})\s*\|",line)
        if not m:
            raise ValueError("unknown/malformed U ledger row: " + line)
        if m:
            cells = split_markdown_cells(line)
            if (len(cells)!=7 or m.group(1) in u_map or cells[3]!="DEFINED"
                or cells[4] not in ("NOT_VERIFIED", "PARTIAL") or cells[5]!="NOT_RUN"):
                raise ValueError("invalid / duplicate U ledger row: "+line)
            if not cells[6].strip():
                raise ValueError("U ledger row has empty evidence scope: " + line)
            if cells[4] == "PARTIAL" and not has_row_specific_partial_evidence(m.group(1), cells[6]):
                raise ValueError("PARTIAL U row lacks linked evidence: " + line)
            u_map[m.group(1)] = set(expand_ids(cells[2], "T"))
            u_phases[m.group(1)] = cells[1]
    for line in table_t:
        m = re.match(r"^\|\s*(T\d{2})\s*\|",line)
        if not m:
            raise ValueError("unknown/malformed T ledger row: " + line)
        if m:
            cells = split_markdown_cells(line)
            if len(cells)!=5 or m.group(1) in t_map or cells[3] not in ("NOT_RUN","PARTIAL"):
                raise ValueError("invalid / duplicate T ledger row: "+line)
            if not cells[4].strip():
                raise ValueError("T ledger row has empty evidence scope: " + line)
            if cells[3] == "PARTIAL" and not has_row_specific_partial_evidence(m.group(1), cells[4]):
                raise ValueError("PARTIAL T row lacks linked evidence: " + line)
            t_map[m.group(1)] = set(expand_ids(cells[2], "U"))
            t_phases[m.group(1)] = cells[1].split("; ")
    return u_map, t_map, u_phases, t_phases

def validate(spec_text, ledger_text):
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
        old = next(line for line in ledger.splitlines()
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
        rows = specification[begin:].splitlines()
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
            row = next(line for line in ledger.splitlines() if line.startswith(target))
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
        table = [line for line in section.splitlines() if line.startswith("|")]
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
            row = next((line for line in ledger.splitlines() if line.startswith("| T01 |")), None)
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
    spec_tail = next(line for line in specification.splitlines() if line.startswith("| U47 |"))
    must_reject(specification.replace(
        spec_tail, spec_tail + "\nU100 | 1 | T01 | M0 |", 1), ledger)
    ledger_tail_u = next(line for line in ledger.splitlines() if line.startswith("| U48 |"))
    must_reject(specification, ledger.replace(
        ledger_tail_u, ledger_tail_u + "\nU100 | M0 | T01 | DEFINED | NOT_VERIFIED | NOT_RUN | Bogus |", 1))
    ledger_tail_t = next(line for line in ledger.splitlines() if line.startswith("| T54 |"))
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

    print(f"M0 U/T traceability coverage PASS: {count[0]} requirements, {count[1]} tests; mutation guards PASS; no system T acceptance implied")
