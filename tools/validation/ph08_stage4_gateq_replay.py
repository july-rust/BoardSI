#!/usr/bin/env python3
"""Reproduce the *historical FAIL* from a SHA-pinned private PH-08 Stage 4 archive.

This is provenance / numerical regression ONLY. A successful run means the
historical HMM Gate Q failure was reproduced, NOT that the HMM is qualified.
The archive is never extracted outside a private TemporaryDirectory, uploaded,
or written to a GitHub Actions log. Only run previously trusted, SHA-pinned code.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

from ph08_evidence_audit import inspect

STAGE4_ZIP_SHA256 = '6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9'
ZIP_ROOT = 'boardsi-ph08-stage4-qualification/'
COMPARE_FIELDS = ('bulk_phi_rms', 'bulk_gradient_rms', 'linear_residual',
                  'interface_phi_rms', 'interface_gradient_rms',
                  'interface_normal_D_exact_error_rms_normalized',
                  'exact_data_max_error')
ABS_TOL = 2e-13
REL_TOL = 5e-10


def compare_rows(actual: dict, reference: dict) -> list[str]:
    errors = []
    if actual.get('status') != 'FAIL' or reference.get('status') != 'FAIL':
        errors.append('historical FAIL status not preserved')
    if actual.get('failures') != reference.get('failures'):
        errors.append('MMS gate failures differ')
    a_rows, b_rows = actual.get('rows', []), reference.get('rows', [])
    if len(a_rows) != 36 or len(b_rows) != 36:
        errors.append('expected 36 MMS rows')
        return errors
    for index, (a, b) in enumerate(zip(a_rows, b_rows)):
        for key in ('grid', 'kind', 'n'):
            if a.get(key) != b.get(key):
                errors.append(f'row {index}: {key} changed')
        for key in COMPARE_FIELDS:
            if (key in a) != (key in b):
                errors.append(f'row {index}: {key} missing')
            elif key in a:
                va, vb = a[key], b[key]
                if (not isinstance(va, (float, int)) or
                    not isinstance(vb, (float, int)) or
                    not math.isfinite(va) or not math.isfinite(vb) or
                    not math.isclose(va, vb, abs_tol=ABS_TOL, rel_tol=REL_TOL)):
                    errors.append(f'row {index}: {key} differs')
    return errors


def check_historical_gate(mms: dict) -> bool:
    """This reproducer *requires* the original fail; it cannot approve PH-08."""
    target = ('uniform', 'quadratic', 'bulk_gradient_rms', 'reduction')
    hits = [x for x in mms.get('failures', []) if tuple(x[:4]) == target]
    return (mms.get('status') == 'FAIL' and len(mms.get('failures', [])) == 1
            and len(hits) == 1 and len(hits[0][4]) == 2
            and all(isinstance(x, (float, int)) and math.isfinite(x) and x < 1.8 for x in hits[0][4]))


def rerun(archive: Path) -> dict:
    if not archive.is_file():
        raise ValueError('original Stage 4 archive is missing')
    actual_sha = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual_sha != STAGE4_ZIP_SHA256:
        raise ValueError('archive SHA-256 mismatch: refusing to run untrusted source')
    audit = inspect(archive, expected_sha256=STAGE4_ZIP_SHA256,
                    expected_entries=338, expected_sources=20)
    if audit.get('result') != 'PASS' or audit.get('checksum_verified') != 338 or audit.get('source_hash_verified') != 20:
        raise ValueError('ZIP integrity / source inventory audit not PASS')
    with zipfile.ZipFile(archive) as z:
        saved = json.loads(z.read(ZIP_ROOT + 'evidence/qualification.json'))['mms']
        manifest = json.loads(z.read(ZIP_ROOT + 'evidence/manifest.json'))
        inventory = manifest.get('sources')
        if not isinstance(inventory, dict) or len(inventory) != 20:
            raise ValueError('unexpected Stage 4 source layout')
        with tempfile.TemporaryDirectory(prefix='boardsi-ph08-replay-') as temp:
            root = Path(temp)
            for basename, expected_digest in sorted(inventory.items()):
                if (not isinstance(basename, str) or Path(basename).name != basename or
                    '/' in basename or '\\' in basename or not basename.endswith(('.py', '.json', '.md'))):
                    raise ValueError('unsafe source snapshot name')
                content = z.read(ZIP_ROOT + 'evidence/source_snapshot/' + basename)
                if hashlib.sha256(content).hexdigest() != expected_digest.lower():
                    raise ValueError('source snapshot digest mismatch')
                (root / basename).write_bytes(content)
            # Subprocess isolates module imports and guarantees no modifications
            # to the caller environment. The archive itself is NOT passed to it.
            program = '''import json,sys
sys.path.insert(0,sys.argv[1])
from m2_ph08_stage4_qualification import mms_suite
print(json.dumps(mms_suite(),allow_nan=False))'''
            env = os.environ.copy()
            env.pop('PYTHONPATH', None)
            p = subprocess.run([sys.executable, '-c', program, str(root)], cwd=root,
                               env=env, capture_output=True, text=True, timeout=120,
                               check=False)
            if p.returncode:
                raise RuntimeError('original HMM replay failed: ' + p.stderr[-1500:])
            replay = json.loads(p.stdout)
    diffs = compare_rows(replay, saved)
    if not check_historical_gate(replay):
        diffs.append('original uniform quadratic gradient failure not reproduced')
    return {
        'status': 'REPRODUCED_HISTORICAL_FAIL' if not diffs else 'REPLAY_MISMATCH',
        'archive_sha256': actual_sha,
        'source_snapshots_verified': 20,
        'checksum_entries_verified': 338,
        'mms_rows_reexecuted': len(replay.get('rows', [])),
        'mms_comparison_errors': diffs,
        'original_gate_q': 'FAIL',
        'AC_P1': 'PENDING',
        'contract_frozen': False,
        'rust_fem_go': False,
        'historical_failures': replay.get('failures', []),
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('archive', type=Path)
    ap.add_argument('--output-json', type=Path)
    args = ap.parse_args(argv)
    try:
        result = rerun(args.archive)
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError, zipfile.BadZipFile,
            subprocess.TimeoutExpired) as exc:
        result = {'status': 'REPLAY_ERROR', 'error': f'{type(exc).__name__}: {exc}',
                  'original_gate_q': 'FAIL', 'AC_P1': 'PENDING', 'rust_fem_go': False}
    output = json.dumps(result, indent=2, allow_nan=False) + '\n'
    if args.output_json:
        args.output_json.write_text(output, encoding='utf-8')
    print(output)
    return 0 if result['status'] == 'REPRODUCED_HISTORICAL_FAIL' else 2


if __name__ == '__main__':
    raise SystemExit(main())
