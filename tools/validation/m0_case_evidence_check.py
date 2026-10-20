"""Fail-closed M0 prototype case-evidence ledger checker, not a T acceptance runner."""
import json
from pathlib import Path
import re
from urllib.parse import urlparse

LEDGER = Path("docs/validation/m0-case-evidence.json")
M0_IDS = {"T10","T11","T13","T32","T34","T35","T39","T46","T50","T54"}
SHA = re.compile(r"[0-9a-f]{40}\Z")
# Intentionally separate from the editable evidence JSON. A new baseline
# requires source-code change, tests and independent review; self-consistency
# of two editable strings must never count as immutable provenance proof.
BASE_SHA = "2a08c167d89279e15f1e65b689a58ed0682afc6e"
BASE_DOC_RUN = "https://github.com/july-rust/BoardSI/actions/runs/39391822708"
QT_RUN = "https://github.com/july-rust/BoardSI/actions/runs/39391822772"
RUST_RUN = "https://github.com/july-rust/BoardSI/actions/runs/39391823062"
F6_PR = "https://github.com/july-rust/BoardSI/pull/127"
EXPECTED_EVIDENCE = {
    "T10": [QT_RUN],
    "T11": [],
    "T13": [QT_RUN, RUST_RUN],
    "T32": [QT_RUN],
    "T34": [QT_RUN],
    "T35": [F6_PR],
    "T39": [QT_RUN, RUST_RUN],
    "T46": [QT_RUN],
    "T50": [QT_RUN],
    "T54": [],
}
# These IDs were observed as successful on signed BASE_SHA; exact source
# verification is recorded separately via the run URLs, not fabricated by CI.
PROTOTYPE = {"PARTIAL", "NOT_RUN"}
EXPECTED_KEYS = {
    "id", "title", "expected", "fixture", "prototype_status",
    "full_acceptance", "evidence_urls", "limitations",
    "case_specific_acceptance_review",
}

def validate(document):
    if set(document) != {"schema","source_commit","source_provenance","semantics","cases"}:
        raise ValueError("Unknown/missing top-level evidence keys")
    if document["schema"] != "boardsi.m0.case-evidence.v1":
        raise ValueError("Unsupported M0 case schema")
    sha = document["source_commit"]
    if not isinstance(sha,str) or not SHA.fullmatch(sha) or sha != BASE_SHA:
        raise ValueError("Immutable SHA does not match independently pinned baseline")
    provenance = document["source_provenance"]
    if provenance.get("main_url") != f"https://github.com/july-rust/BoardSI/commit/{sha}":
        raise ValueError("Immutable source URL does not match SHA")
    if provenance.get("verified") != "valid" or provenance.get("mandatory_checks") != "17/17 success":
        raise ValueError("Provenance record lacks validated main checks")
    if provenance.get("run_url") != BASE_DOC_RUN:
        raise ValueError("Provenance must contain independently pinned successful docs CI run")
    if not isinstance(document["semantics"], dict) or set(document["semantics"]) != {
        "prototype_status","full_acceptance","review_scope"
    } or any(not isinstance(v, str) or len(v) < 25 for v in document["semantics"].values()):
        raise ValueError("Scope and review limitations must be explicit")
    cases = document["cases"]
    if not isinstance(cases,list) or len(cases) != len(M0_IDS):
        raise ValueError("Exactly ten M0-related case records are required")
    seen = set()
    for item in cases:
        if not isinstance(item,dict) or set(item) != EXPECTED_KEYS:
            raise ValueError("Case keys missing/unknown")
        key = item["id"]
        if key not in M0_IDS or key in seen:
            raise ValueError("Missing, duplicate or unexpected M0 T case")
        seen.add(key)
        if item["prototype_status"] not in PROTOTYPE:
            raise ValueError("Prototype status cannot claim full PASS")
        if item["full_acceptance"] != "NOT_RUN" or item["case_specific_acceptance_review"] != "NOT_PERFORMED":
            raise ValueError("Do not claim unperformed full acceptance / reviewer")
        if not isinstance(item["title"],str) or len(item["title"].strip()) < 8 or not all(
            isinstance(item[n],str) and len(item[n].strip()) >= 20
            for n in ("expected","fixture","limitations")
        ):
            raise ValueError("Case's expected result, fixture and limits required")
        links = item["evidence_urls"]
        if links != EXPECTED_EVIDENCE[key]:
            raise ValueError("Evidence URLs do not match case-specific pinned provenance")
        if not isinstance(links,list) or len(links) != len(set(links)):
            raise ValueError("Case links must be a unique list")
        if item["prototype_status"] == "NOT_RUN" and links:
            raise ValueError("NOT_RUN must not silently cite passing evidence")
        if item["prototype_status"] == "PARTIAL" and not links:
            raise ValueError("PARTIAL requires at least one concrete run or PR")
        for url in links:
            if not isinstance(url,str):
                raise ValueError("URL must be literal string")
            parsed = urlparse(url)
            if parsed.scheme != "https" or parsed.netloc != "github.com" or not re.fullmatch(
                r"/july-rust/BoardSI/(?:actions/runs/[0-9]+|pull/[0-9]+)",parsed.path
            ) or parsed.params or parsed.query or parsed.fragment:
                raise ValueError("Evidence must be a concrete BoardSI run/PR URL")
    if seen != M0_IDS:
        raise ValueError("M0 case coverage mismatch")
    return True

if __name__ == "__main__":
    validate(json.loads(LEDGER.read_text(encoding="utf-8")))
    print("M0 F1: 10/10 scoped evidence records structurally valid; full T acceptance NOT_RUN")
