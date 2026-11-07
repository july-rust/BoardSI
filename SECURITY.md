# BoardSI Security Policy (pre-release)

[← Project](README.md) · [Documentation](docs/README.md) · [Contributing](CONTRIBUTING.md) · [Developer Quickstart](docs/developer-quickstart.md)

> **Status (2026-11-07 KST):** BoardSI is a limited source-built developer Preview, **not** a security-audited general-purpose CAD importer, production SI analyzer or redistributable release. No product release support commitments or guaranteed vulnerability response times are established here.

## Reporting a potential vulnerability

**Do not disclose exploitation details, confidential board files, token values or internal paths in public GitHub Issues, PR comments or Actions logs.** If GitHub private vulnerability reporting is enabled for this repository, use the **Security → Report a vulnerability** workflow. Otherwise contact the repository owner through an **already trusted private communication channel**; do not assume an unpublished contact address.

A useful report includes:

- Component and affected exact commit/PR or locally built binary SHA, OS/architecture and compiler if relevant.
- Minimal **synthetic/public** reproduction steps, expected behavior versus observed behavior.
- Input-format/parser or IPC boundary, whether any data escaped the local machine, and why the issue may be exploitable.
- Affected trust boundary and an example **redacted** diagnostic. Do not attach the original employer PCB or secret.

Until there is a verified mitigation, avoid publishing a working exploit against private board data.

## Data handling and confidentiality

BoardSI's intended processing is **local/offline**. Never upload company PCB artwork, PADS/ODB++ exports, layer stacks, nets, Touchstone/SNP, measurements, original private PH-08 archives, license secrets or private file paths to third-party services or public CI. Only publicly redistributable or fully synthetic sample data belong in the repository and automated test fixtures.

Security requires more than a \`.gitignore\`, secret scan or “offline” label: check that errors, exported artifacts, crash dumps, command-line argument logs, test receipts, viewer diagnostics and GitHub Actions never leak source content or Net/Pad identifiers from confidential designs.

## Trust boundaries and known unqualified areas

| Boundary | Intended control or test concern | Current limit |
| :--- | :--- | :--- |
| Input files | Strict supported-version grammar, object bounds, coordinate overflow, file size and malformed metadata handling | General PADS/ODB++ parser security has **not** passed a production audit |
| Copper geometry | Source ownership, layer/Net provenance, uncertain tangency fail-closed and drill-vs-annulus distinction | Local contact candidate **is not** proof of physical electrical Open/Short |
| Workspace/cache | Prevent directory traversal, symlink escapes, stale source fingerprints and cross-project reuse | Full production disk cache/recovery has **not** been accepted |
| Local IPC / Worker | Bounded messages, version/sequence/identity validation, cancel/timeout and process isolation | Production solver interactions and hostile-OS hardening remain unqualified |
| Qt runtime | Verify actual renderer/worker path, cleanup and native deployment | CI prototype PASS is not signed installer or clean-system security certification |
| Third-party components | Qt, C/C++ runtime, CAD parser dependencies, licensing and redistribution | Rights/notices/dependency security review required before distribution |

This is a **risk register**, not proof every mitigator above is implemented. [Architecture](docs/architecture/README.md), [validation status](docs/validation/README.md), [single checklist](docs/development-checklist.md).

## Fix and verification policy

Use a focused PR with minimal public synthetic reproductions, negative tests and a precise supported scope. Record exact source SHA, operating system, observed failure, fix and regression result. Keep the canonical checklist consistent; do not change historical physical-model acceptance conclusions or call a green formatter/security scan evidence of PCB SI accuracy. Native test success and legal release permission remain separate gates.

**Private vulnerability disclosure, product release eligibility and the ability to protect real proprietary board data are not guaranteed by this pre-release policy.**
