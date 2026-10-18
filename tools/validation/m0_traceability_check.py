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

def validate_definition_tables(source):
    # The crosswalk/ledger is invalid if the actual source definitions vanish.
    ranges = (
        ("U", 48, "## 2 요구사항 추적표", "## 3 "),
        ("T", 54, "## 13 인수시험 및 독립 교차 검증", "### 13.1 "),
    )
    for prefix, maximum, begin, end in ranges:
        if begin not in source or end not in source.split(begin, 1)[1]:
            raise ValueError("specification definition section missing: " + prefix)
        section = source.split(begin, 1)[1].split(end, 1)[0]
        found = set()
        for line in section.splitlines():
            match = re.match(rf"^\|\s*({prefix}[^|]*)\|", line)
            if not match:
                continue
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            expected_columns = 4 if prefix == "U" else 3
            if len(cells) != expected_columns or any(not c for c in cells):
                raise ValueError("malformed definition for " + match.group(1))
            item = match.group(1).strip()
            if prefix == "T" and item == "Test ID" and line.strip() == "| Test ID | 주요 검증 | 통과 조건 |":
                continue  # Known table header, not an acceptance test definition.
            if not re.fullmatch(rf"{prefix}\d{{2}}", item):
                raise ValueError("malformed or out-of-range definition ID: " + item)
            if item in found:
                raise ValueError("duplicate " + prefix + " definition: " + item)
            found.add(item)
        expected = {f"{prefix}{i:02d}" for i in range(1, maximum + 1)}
        if found != expected:
            raise ValueError(f"{prefix} definition mismatch: missing={sorted(expected-found)}, extra={sorted(found-expected)}")

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


    print(f"M0 U/T traceability coverage PASS: {count[0]} requirements, {count[1]} tests; mutation guards PASS; no system T acceptance implied")
