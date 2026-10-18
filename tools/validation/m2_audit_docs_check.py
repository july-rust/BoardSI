"""Protect preregistered contracts and current status wording (no physics claim)."""
import hashlib
from pathlib import Path

FROZEN_CONTRACTS = {'docs/validation/m2-g2-ph05-acceptance-contract.md': '994d18f1bda16b6a5d54ee4afa8bc562d68cd7f20ecd6bb76c300552001cb4e9', 'docs/validation/m2-g2-ph06-acceptance-contract.md': '039ea7f80089e146a6b1d3f2a39bf227cad394e511559fc7be860375bcae6b3e', 'docs/validation/m2-g2-ph07-acceptance-contract.md': '50c025a9ecd9d2c789463fe5fe5119f059d03a7624b85d4ab5a3f6d0fe89beb7', 'docs/validation/m2-ph03-parallel-plate-acceptance-contract.md': '20bbb2310f1d9c1f59df58d22b3cba4447e0d3f6a91efc97928c36b29e8e294f'}
for path, expected in FROZEN_CONTRACTS.items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==expected, path+' frozen acceptance changed'
for path in ('README.md','docs/development-history.md','docs/validation/README.md',
             'docs/validation/m2-g2-ph07-fem-evidence.md',
             'docs/validation/m2-g2-ph07-independent-reference.md',
             'docs/validation/m2-rlgc-independent-verification-plan.md'):
    text=Path(path).read_text()
    assert 'Draft PR #40' not in text, path+' stale Draft PR #40'
    assert 'does not yet contain a working solver' not in text, path+' stale M0-only status'
    assert '740 independent gates' not in text, path+' inflated oracle independence'
    assert 'm2-validation-audit-hardening.md' in text, path+' missing audit traceability'
print('Audit docs PASS: 4 unchanged frozen contracts and 6 current-status documents')

# Baseline status/asset consistency checks; never a substitute for original HMM physics.
PRE_PH08_STATUS = {
    'README.md': ('Stage 5B', 'Gate Q', 'NOT_FROZEN', 'BOUND_NOT_CERTIFIED'),
    'docs/development-history.md': ('Stage 5B original HMM privately re-executed', '1761/1761', 'Gate Q FAIL', 'BOUND_NOT_CERTIFIED'),
    'docs/architecture/README.md': ('Qt Widgets', 'Gate Q', 'BOUND_NOT_CERTIFIED'),
    'docs/audits/2026-10-18-pre-ph08-project-audit.md': ('Stage 5B', 'FAIL', '17/17', 'Draft PR #35'),
    'docs/branding/boardsi-icon-decision.md': ('option B', 'SOURCE_ASSET_MISSING', 'RECONSTRUCTION_ONLY'),
    'assets/branding/boardsi-option-b-reconstruction.svg': ('RECONSTRUCTION_ONLY', '<svg', '</svg>'),
}
for audit_path, phrases in PRE_PH08_STATUS.items():
    audit_text = Path(audit_path).read_text(encoding='utf-8')
    for phrase in phrases:
        assert phrase in audit_text, 'Pre-PH08 audit drift: %s missing %s' % (audit_path, phrase)
assert 'Stage 5B original physics rerun remains NOT RUN' not in Path('docs/development-history.md').read_text(encoding='utf-8'), 'Old original Stage5B replay status'
print('Pre-PH08 docs and option-B provenance consistency PASS, not original HMM certification')
