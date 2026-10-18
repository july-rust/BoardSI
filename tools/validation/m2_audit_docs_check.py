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
    'docs/branding/boardsi-icon-decision.md': ('icon B', 'DESIGN_APPROVED', 'ORIGINAL_PNG_BYTE_IDENTITY_NOT_VERIFIED', 'RELEASE_PACKAGING_NOT_VERIFIED'),
    'assets/branding/boardsi-option-b-reconstruction.svg': ('RECONSTRUCTION_ONLY', '<svg', '</svg>'),
}
for audit_path, phrases in PRE_PH08_STATUS.items():
    audit_text = Path(audit_path).read_text(encoding='utf-8')
    for phrase in phrases:
        assert phrase in audit_text, 'Pre-PH08 audit drift: %s missing %s' % (audit_path, phrase)
assert 'Stage 5B original physics rerun remains NOT RUN' not in Path('docs/development-history.md').read_text(encoding='utf-8'), 'Old original Stage5B replay status'
print('Pre-PH08 docs and option-B provenance consistency PASS, not original HMM certification')

# Fail-closed static SVG allowlist: XML well-formedness alone is NOT a security check.
# Deliberately narrow to the approved artwork's actual static SVG vocabulary.
import re
import xml.etree.ElementTree as ET

SVG_NS = '{http://www.w3.org/2000/svg}'
STATIC_TAGS = {'svg', 'defs', 'linearGradient', 'stop', 'rect', 'g', 'path', 'circle'}
STATIC_ATTRIBUTES = {
    'width', 'height', 'viewBox', 'role', 'aria-label', 'id',
    'x1', 'x2', 'y1', 'y2', 'offset', 'stop-color', 'stop-opacity',
    'x', 'y', 'rx', 'd', 'cx', 'cy', 'r', 'fill',
    'stroke', 'stroke-width', 'stroke-linecap', 'stroke-linejoin', 'opacity',
}
LOCAL_GRADIENT = re.compile(r'url\(#([A-Za-z_][A-Za-z0-9_.-]*)\)\Z')
SCHEME = re.compile(r'(?i)(?:https?:|file:|data:|javascript:|vbscript:|@import|\\\\|//|url\s*\()')

def validate_offline_static_svg(svg_bytes):
    if len(svg_bytes) > 256 * 1024:
        raise ValueError('SVG too large')
    # Interpret only UTF-8. ElementTree drops stylesheet processing
    # instructions but SVG renderers can load them; reject before parsing.
    raw = svg_bytes.decode('utf-8', errors='strict')
    without_declaration = re.sub(r'\A\s*<\?xml\s+[^?]*\?>', '', raw, count=1, flags=re.I)
    if '<?' in without_declaration:
        raise ValueError('XML processing instructions forbidden')
    if re.search(r'(?i)<!DOCTYPE|<!ENTITY', raw):
        raise ValueError('DTD and entities forbidden')
    root = ET.fromstring(svg_bytes)
    if root.tag != SVG_NS + 'svg':
        raise ValueError('not an SVG document')
    ids = set()
    for element in root.iter():
        if not isinstance(element.tag, str) or not element.tag.startswith(SVG_NS):
            raise ValueError('foreign namespace / non-element node')
        tag = element.tag[len(SVG_NS):]
        if tag not in STATIC_TAGS:
            raise ValueError('active or unsupported SVG element: ' + tag)
        for key, value in element.attrib.items():
            # Namespaced attributes (xlink:href, xml:base) and CSS 'style' are
            # never allowed, even if the address is encoded as XML entities.
            if key not in STATIC_ATTRIBUTES:
                raise ValueError('unsafe SVG attribute: ' + key)
            if key == 'id':
                if not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_.-]*', value) or value in ids:
                    raise ValueError('invalid or duplicate SVG id')
                ids.add(value)
            if '\\' in value:
                raise ValueError('CSS escape / backslash forbidden in static SVG attributes')
            if key in ('fill', 'stroke') and value.startswith('url('):
                # Only same-document gradients, verified in second pass.
                if not LOCAL_GRADIENT.fullmatch(value):
                    raise ValueError('non-local SVG paint reference')
            elif key in ('fill', 'stroke', 'stop-color'):
                # Explicit paint grammar rejects CSS escapes, functions and URLs.
                if not re.fullmatch(r'#[0-9a-fA-F]{3,8}|none', value):
                    raise ValueError('unsupported static paint value')
            elif SCHEME.search(value):
                raise ValueError('URI/CSS directive is forbidden')
    for element in root.iter():
        for key in ('fill', 'stroke'):
            value = element.attrib.get(key, '')
            match = LOCAL_GRADIENT.fullmatch(value)
            if match and match.group(1) not in ids:
                raise ValueError('dangling SVG local reference')
    # Only gradients may be paint references (not references to script/other IDs).
    gradients = {e.attrib.get('id') for e in root.iter()
                 if e.tag == SVG_NS + 'linearGradient'}
    for element in root.iter():
        for key in ('fill', 'stroke'):
            match = LOCAL_GRADIENT.fullmatch(element.attrib.get(key, ''))
            if match and match.group(1) not in gradients:
                raise ValueError('non-gradient SVG paint reference')
    return root

ICON_PATH = 'assets/branding/boardsi-option-b-reconstruction.svg'
# Exact owner-approved design master; changing even static geometry or colors
# requires explicit fresh owner approval and checksum review, not silent drift.
APPROVED_B_SVG_SHA256 = '92ac87c323f49dcb3a15e88057d03357b39c9f5d1d8d8b4e1b1702b96b486406'
original_svg = Path(ICON_PATH).read_bytes()
assert hashlib.sha256(original_svg).hexdigest() == APPROVED_B_SVG_SHA256, (
    'Owner-approved SVG bytes changed without a documented new approval'
)
validate_offline_static_svg(original_svg)
tampered_svg = original_svg.replace(b'width="1024"', b'width="1025"', 1)
assert tampered_svg != original_svg, 'SVG tamper regression fixture missing'
assert hashlib.sha256(tampered_svg).hexdigest() != APPROVED_B_SVG_SHA256, (
    'SVG visual change should not match approved SHA-256'
)
# Execute mutation tests in the same required docs job; one error is enough
# to fail CI rather than giving false assurances of offline/static safety.
def _must_reject(fragment):
    try:
        validate_offline_static_svg(fragment)
    except (ValueError, ET.ParseError, UnicodeDecodeError):
        return
    raise AssertionError('Unsafe SVG accepted: ' + fragment[:180].decode('utf-8', 'replace'))
svg_start = b'<svg xmlns="http://www.w3.org/2000/svg">'
svg_end = b'</svg>'
for bad in (
    b'<script>alert(1)</script>',
    b'<foreignObject><div xmlns="http://www.w3.org/1999/xhtml">x</div></foreignObject>',
    b'<style>@import url(https://example.invalid/bad.css)</style>',
    b'<rect style="fill:url(https://example.invalid/x)" />',
    b'<rect fill="url(https://example.invalid/image.svg)" />',
    b'<rect fill="url(#unknown)" />',
    b'<rect onclick="alert(1)" />',
    b'<image href="https://example.invalid/image.svg" />',
    b'<rect xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="https://example.invalid/a" />',
    b'<animate attributeName="opacity" values="0;1" />',
    b'<rect fill="url(&#104;ttps://example.invalid/x)" />',
    br'<rect fill="u\72 l(\68\74\74\70\73\3a\2f\2f example.invalid\2f x)" />',
    b'<?xml-stylesheet type="text/css" href="https://example.invalid/a.css"?>',
):
    _must_reject(svg_start + bad + svg_end)
_must_reject(b'<?xml-stylesheet type="text/css" href="https://example.invalid/a.css"?>' + svg_start + svg_end)
_must_reject(b'<!DOCTYPE svg [<!ENTITY x SYSTEM "file:///private">]>' + svg_start + svg_end)
print('B DESIGN_APPROVED static SVG safety PASS: immutable owner-approved SHA-256, local-gradient allowlist and 15 attack mutations; packaging NOT VERIFIED')

# Historical nested-archive checksum inventories must never be conflated.
stage5d = Path('docs/validation/m2-g2-ph08-stage5d-evidence.md').read_text(encoding='utf-8')
assert 'Stage 5B 원시 338개 hash 및 source 20개' not in stage5d, 'Stage 4 inventory misattributed to Stage 5B'
for marker in ('Stage 5B 외부 ZIP 자체의 36개 checksum', 'Stage 4의 338개 checksum', 'Stage 5B HMM', 'Gate Q **FAIL**'):
    assert marker in stage5d, 'Stage 5D time-scope/inventory drift: ' + marker
print('Historical Stage5B 36/13 versus nested Stage4 338/20 evidence classification PASS')
