"""Fail-closed M0 prototype case-evidence ledger checker, not a T acceptance runner."""
import json
from pathlib import Path
import re
from urllib.parse import urlparse

LEDGER = Path("docs/validation/m0-case-evidence.json")
M0_IDS = {"T10","T11","T13","T32","T34","T35","T39","T46","T50","T54"}
SHA = re.compile(r"[0-9a-f]{40}\Z")
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
    if not isinstance(sha,str) or not SHA.fullmatch(sha):
        raise ValueError("A full immutable source commit SHA is mandatory")
    provenance = document["source_provenance"]
    if provenance.get("main_url") != f"https://github.com/july-rust/BoardSI/commit/{sha}":
        raise ValueError("Immutable source URL does not match SHA")
    if provenance.get("verified") != "valid" or provenance.get("mandatory_checks") != "17/17 success":
        raise ValueError("Provenance record lacks validated main checks")
    if not isinstance(provenance.get("run_url"), str):
        raise ValueError("Provenance requires exact document CI evidence")
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
        if not all(isinstance(item[n],str) and len(item[n].strip()) >= 20
                   for n in ("title","expected","fixture","limitations")):
            raise ValueError("Case's expected result, fixture and limits required")
        links = item["evidence_urls"]
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
