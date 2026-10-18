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
    index = 0
    while index < len(line):
        if line[index] != chr(96):
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
            if (next_open == 0 or line[next_open - 1] != chr(96)) and (
                next_end == len(line) or line[next_end] != chr(96)
            ):
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
        r"^ {0,3}(?:</?[A-Za-z][A-Za-z0-9-]*(?:\s+[^<>]*)?\s*/?>|<![A-Za-z\[]|<\?)",
        re.IGNORECASE,
    )
    for original_line in source.splitlines():
        # A fenced block owns all its content, even HTML-looking tokens.
        if fence_character is not None:
            line = original_line.lstrip(" ")
            indent = len(original_line) - len(line)
            if indent <= 3 and line.startswith(fence_character):
                length = len(line) - len(line.lstrip(fence_character))
                if length >= fence_length and not line[length:].strip():
                    fence_character, fence_length = None, 0
            yield ""
            continue

        # Indented code outside an existing comment is not HTML markup.
        # Existing multi-line comments must still consume their closing token.
        if not in_comment and original_line.expandtabs(4).startswith("    "):
            yield ""
            continue
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
        # CommonMark expands mixed whitespace at four-column tab stops.
        if text.expandtabs(4).startswith("    "):
            yield ""
            continue
        unindented = text.lstrip(" ")
        indent = len(text) - len(unindented)
        if indent <= 3 and unindented:
            character = unindented[0]
            if character in (chr(96), "~"):
                length = len(unindented) - len(unindented.lstrip(character))
                if length >= 3:
                    fence_character, fence_length = character, length
                    yield ""
                    continue

        # Normative definition tables cannot occur inside raw HTML blocks.
        # A literal example inside a code fence/comment is already excluded.
        if html_open.match(text):
            raise ValueError("raw HTML block is unsupported in normative specification")
        yield text
    if in_comment or fence_character is not None:
        raise ValueError("unclosed HTML comment or code fence in specification")


def validate_definition_tables(source):
    """Require an actual contiguous Markdown definition table, not just ID text."""
    # Parse comments/fences across the ENTIRE document first. An HTML comment
    # starting before a section heading must not make its hidden table count.
    visible_source = "\n".join(visible_markdown_lines(source))
    ranges = (
        ("U", 48, "## 2 요구사항 추적표", "## 3 ",
         ["ID", "요구사항", "상태", "근거"]),
        ("T", 54, "## 13 인수시험 및 독립 교차 검증", "### 13.1 ",
         ["Test ID", "주요 검증", "통과 조건"]),
    )
    for prefix, maximum, begin, end, header in ranges:
        if begin not in visible_source or end not in visible_source.split(begin, 1)[1]:
            raise ValueError("visible specification definition section missing: " + prefix)
        section = visible_source.split(begin, 1)[1].split(end, 1)[0]
        visible = section.splitlines()
        found, table_count = set(), 0
        for position, line in enumerate(visible):
            if split_markdown_cells(line.strip()) != header:
                continue
            table_count += 1
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


def extract_mapping(source):
    # Rendered, not raw, crosswalk rows define the public specification.
    source = "\n".join(visible_markdown_lines(source))
    if "### 18.2 요구사항과 시험 연결" not in source or "### 18.3" not in source:
        raise ValueError("specification crosswalk section missing")
    section = source.split("### 18.2 요구사항과 시험 연결", 1)[1].split("### 18.3", 1)[0]
    mapping, u_phases, t_phases = {}, {}, {}
    for line in section.splitlines():
        if not re.match(r"\|\s*U\d{2}", line):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 4:
            raise ValueError("bad section 18.2 row width")
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
    return mapping, u_phases, t_phases

def extract_ledger(source):
    # The audit ledger is also normative Markdown: code/commented rows do not count.
    source = "\n".join(visible_markdown_lines(source))
    table_u = source.split("## U01–U48, full coverage (48/48)",1)[1].split("## T01–T54",1)[0]
    table_t = source.split("## T01–T54, full coverage (54/54)",1)[1].split("## Scope",1)[0]
    u_map, t_map, u_phases, t_phases = {}, {}, {}, {}
    for line in table_u.splitlines():
        m = re.match(r"^\|\s*(U\d{2})\s*\|",line)
        if m:
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cells)!=7 or m.group(1) in u_map or cells[3]!="DEFINED" or cells[5]!="NOT_RUN":
                raise ValueError("invalid / duplicate U ledger row: "+line)
            u_map[m.group(1)] = set(expand_ids(cells[2], "T"))
            u_phases[m.group(1)] = cells[1]
    for line in table_t.splitlines():
        m = re.match(r"^\|\s*(T\d{2})\s*\|",line)
        if m:
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cells)!=5 or m.group(1) in t_map or cells[3] not in ("NOT_RUN","PARTIAL"):
                raise ValueError("invalid / duplicate T ledger row: "+line)
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

    print(f"M0 U/T traceability coverage PASS: {count[0]} requirements, {count[1]} tests; mutation guards PASS; no system T acceptance implied")
