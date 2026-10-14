# BoardSI Project Management

Source of truth: [Development Specification v1.0](specification/BoardSI_Development_Spec_v1.0.md). Requirements U01–U48; acceptance tests T01–T54. A document's completion is NOT proof of program correctness.

## Milestones and labels

[Project plan JSON](project-plan.json) defines 11 development milestone stages, 22 labels and 24 planning Issues. The workflow [bootstrap-management.yml](../.github/workflows/bootstrap-management.yml) creates missing GitHub Milestones/Labels and links existing Issues by their exact titles. Trigger: merge of setup PR into main, or Actions → Bootstrap GitHub Milestones and Labels → Run workflow. It is idempotent; it must show a successful Actions run before considering Milestones created.

## GitHub Projects v2

The connected GitHub service does not offer a direct Projects creation action. User: create a Board project **BoardSI Development** at https://github.com/users/july-rust/projects/new and link the BoardSI repository. Recommended status columns: Backlog, Ready, In Progress, In Review, Blocked, Done. Add Priority (P0/P1/P2), Area and Requirement/Test fields when supported. Issues and Milestones remain the durable task records.

## Development policy

- M0: Qt/Rust binding, versioned IPC, canonical geometry model, native OS builds, public fixtures and offline confidentiality.
- M1/M1a/M1b: import, artwork, substrate geometry, port configuration and validation.
- M2/M2a/M2b: validated RLGC/direct S, progressive sweep, live statuses, bounded RAM/SSD cache and recovery — initial release gates.
- M2c/M3/M4/M5: **future scope only**; never imply implemented.
- All work on focused branches; merge only after tests and written evidence.
- No real company PCB/stackup/netlist/Touchstone data in repository, PRs, Issues, CI or artifacts.
- Native compilation and actual runtime acceptance are distinct; the docs-only CI intentionally checks neither.
