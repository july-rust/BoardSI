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


def visible_markdown_lines(section):
    """Strip HTML comments and fenced code, including multi-line regions."""
    in_comment = False
    fence_character = None
    fence_length = 0
    for original_line in section.splitlines():
        text = original_line
        unindented = text.lstrip(" ")
        indent = len(text) - len(unindented)
        if indent <= 3 and unindented:
            character = unindented[0]
            if character in (chr(96), "~"):
                count = len(unindented) - len(unindented.lstrip(character))
                if count >= 3:
                    if fence_character is None:
                        fence_character, fence_length = character, count
                        continue
                    if character == fence_character and count >= fence_length and not unindented[count:].strip():
                        fence_character, fence_length = None, 0
                        continue
        if fence_character is not None:
            continue
        visible = []
        index = 0
        while index < len(text):
            if in_comment:
                end = text.find("-->", index)
                if end == -1:
                    index = len(text)
                else:
                    in_comment = False
                    index = end + 3
            else:
                start = text.find("<!--", index)
                if start == -1:
                    visible.append(text[index:])
                    index = len(text)
                else:
                    visible.append(text[index:start])
                    in_comment = True
                    index = start + 4
        yield "".join(visible)
    if in_comment or fence_character is not None:
        raise ValueError("unclosed HTML comment or code fence in definition section")


def validate_definition_tables(source):
    """Require an actual contiguous Markdown definition table, not just ID text."""
    ranges = (
        ("U", 48, "## 2 요구사항 추적표", "## 3 ",
         ["ID", "요구사항", "상태", "근거"]),
        ("T", 54, "## 13 인수시험 및 독립 교차 검증", "### 13.1 ",
         ["Test ID", "주요 검증", "통과 조건"]),
    )
    for prefix, maximum, begin, end, header in ranges:
        if begin not in source or end not in source.split(begin, 1)[1]:
            raise ValueError("specification definition section missing: " + prefix)
        section = source.split(begin, 1)[1].split(end, 1)[0]
        visible = list(visible_markdown_lines(section))
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






    print(f"M0 U/T traceability coverage PASS: {count[0]} requirements, {count[1]} tests; mutation guards PASS; no system T acceptance implied")
