"""Mutation tests ensure missing/false M0 acceptance evidence fails CI."""
import copy
import json
from pathlib import Path
import unittest

from m0_case_evidence_check import validate, LEDGER

class CaseEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.original = json.loads(LEDGER.read_text(encoding="utf-8"))

    def test_baseline_is_structurally_valid(self):
        self.assertTrue(validate(self.original))

    def mutation_fails(self, mutate, reason):
        obj = copy.deepcopy(self.original)
        mutate(obj)
        with self.assertRaisesRegex(ValueError, reason):
            validate(obj)

    def test_missing_case_is_rejected(self):
        self.mutation_fails(lambda x: x["cases"].pop(), "Exactly ten")

    def test_duplicate_id_is_rejected(self):
        self.mutation_fails(lambda x: x["cases"][1].update(id="T10"), "duplicate")

    def test_false_full_acceptance_is_rejected(self):
        self.mutation_fails(lambda x: x["cases"][0].update(full_acceptance="PASS"),
                            "full acceptance")

    def test_false_case_reviewer_is_rejected(self):
        self.mutation_fails(lambda x: x["cases"][0].update(case_specific_acceptance_review="PASS"),
                            "reviewer")

    def test_false_prototype_pass_is_rejected(self):
        self.mutation_fails(lambda x: x["cases"][0].update(prototype_status="PASS"),
                            "Prototype status")

    def test_partial_case_requires_real_link(self):
        self.mutation_fails(lambda x: x["cases"][0].update(evidence_urls=[]),
                            "pinned provenance")

    def test_unrun_case_cannot_embed_fake_link(self):
        self.mutation_fails(lambda x: x["cases"][1].update(evidence_urls=[
            "https://github.com/july-rust/BoardSI/actions/runs/1"
        ]), "pinned provenance")

    def test_external_or_placeholder_evidence_rejected(self):
        self.mutation_fails(lambda x: x["cases"][0].update(evidence_urls=[
            "https://evil.example/BoardSI/actions/runs/39391822772"
        ]), "pinned provenance")

    def test_source_url_mismatch_rejected(self):
        self.mutation_fails(lambda x: x["source_commit"] == "0"*40 or x.update(
            source_commit="0"*40), "pinned baseline")

    def test_fabricated_valid_same_repo_run_is_rejected(self):
        self.mutation_fails(
            lambda x: x["cases"][0].update(
                evidence_urls=["https://github.com/july-rust/BoardSI/actions/runs/1"]
            ), "case-specific pinned provenance"
        )

    def test_coordinated_source_and_self_consistent_url_is_rejected(self):
        def falsify(x):
            x["source_commit"] = "0" * 40
            x["source_provenance"]["main_url"] = (
                "https://github.com/july-rust/BoardSI/commit/" + "0" * 40
            )
        self.mutation_fails(falsify, "pinned baseline")

    def test_fabricated_provenance_run_is_rejected(self):
        self.mutation_fails(
            lambda x: x["source_provenance"].update(
                run_url="https://github.com/july-rust/BoardSI/actions/runs/1"
            ), "pinned successful docs"
        )

    def test_top_level_full_acceptance_claim_is_rejected(self):
        self.mutation_fails(
            lambda x: x["semantics"].update(
                full_acceptance="All full T tests are independently accepted and approved"
            ), "pinned no-acceptance contract"
        )

    def test_top_level_reviewer_claim_is_rejected(self):
        self.mutation_fails(
            lambda x: x["semantics"].update(
                review_scope="Every independent acceptance reviewer signed off on M0"
            ), "pinned no-acceptance contract"
        )

    def test_missing_limitations_rejected(self):
        self.mutation_fails(lambda x: x["cases"][3].update(limitations="none"),
                            "limits required")

if __name__ == "__main__":
    unittest.main()
