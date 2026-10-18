"""Self-contained positive/negative tests; private Stage 4 archive not in CI."""
import unittest
import io
import json
import tempfile
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch
from ph08_stage4_gateq_replay import (
    compare_rows, check_historical_gate, ReplayNotVerified,
    _approved_parent_runtime, _read_original_once, _verified_child_runtime, fail, main,
)


def reference():
    return {
        'status': 'FAIL',
        'failures': [['uniform','quadratic','bulk_gradient_rms','reduction',
                      [0.953506146487913,0.45916314091601546]]],
        'rows': [{'grid':g,'kind':k,'n':n,'bulk_phi_rms':0.01,
                  'bulk_gradient_rms':0.001,'linear_residual':1e-15}
                 for g in ('uniform','skew','graded')
                 for k in ('affine','quadratic','interface_affine','interface_quadratic')
                 for n in (8,16,32)]}


class ArchivedFailReplay(unittest.TestCase):
    def test_original_fail_can_be_reproduced_without_becoming_pass(self):
        x=reference()
        self.assertTrue(check_historical_gate(x))
        self.assertEqual(compare_rows(x,x),[])

    def test_mutated_gradient_detected(self):
        x=reference(); y=reference()
        y['rows'][6]['bulk_gradient_rms']+=1e-4
        self.assertTrue(any('bulk_gradient_rms' in s for s in compare_rows(y,x)))

    def test_historical_failure_cannot_be_silently_cleared(self):
        x=reference(); y=reference(); y['failures']=[]
        self.assertFalse(check_historical_gate(y))
        self.assertTrue(compare_rows(y,x))

    def test_false_physical_pass_rejected(self):
        x=reference(); x['status']='PASS'
        self.assertFalse(check_historical_gate(x))

    def test_wrong_row_count_rejected(self):
        x=reference(); x['rows'].pop()
        self.assertTrue(compare_rows(x,reference()))

    def test_nonfinite_gradient_rejected(self):
        x=reference(); x['rows'][0]['bulk_gradient_rms']=float('nan')
        self.assertTrue(compare_rows(x,reference()))

    def test_reduction_at_threshold_not_fail(self):
        x=reference();x['failures'][0][4]=[1.8,1.9]
        self.assertFalse(check_historical_gate(x))

    def test_row_id_mutation_rejected(self):
        x=reference(); x['rows'][0]['grid']='skew'
        self.assertTrue(compare_rows(x,reference()))



    def test_replay_child_observed_runtime_validation(self):
        expected = {'python': '3.13.5', 'numpy': '2.3.5', 'scipy': '1.17.0'}
        self.assertTrue(_verified_child_runtime(expected))
        for bad in (
            {**expected, 'numpy': '9.9.9'},
            {**expected, 'scipy': '0.0.0'},
            {**expected, 'python': '3.12.9'},
            {**expected, 'extra': 'unexpected'},
            {'python': '3.13.5', 'numpy': '2.3.5'},
            'not a runtime object',
        ):
            with self.subTest(bad=bad):
                self.assertFalse(_verified_child_runtime(bad))

    def test_private_archive_invalid_digest_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad_zip = Path(tmp) / 'private-company-path.zip'
            bad_zip.write_bytes(b'public controlled invalid archive')
            with self.assertRaises(ReplayNotVerified) as caught:
                _read_original_once(bad_zip)
            self.assertEqual(caught.exception.code, 'ORIGINAL_ARCHIVE_SHA256_MISMATCH')

    def test_private_archive_oversize_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            large_zip = Path(tmp) / 'large.zip'
            large_zip.write_bytes(b'x' * (8 * 1024 * 1024 + 1))
            with self.assertRaises(ReplayNotVerified) as caught:
                _read_original_once(large_zip)
            self.assertEqual(caught.exception.code, 'ARCHIVE_SIZE_LIMIT')

    def test_runtime_mismatch_fails_closed(self):
        with patch('ph08_stage4_gateq_replay.importlib.metadata.version',
                   return_value='unapproved'):
            self.assertFalse(_approved_parent_runtime())

    def test_public_sanitizer_never_includes_private_exception(self):
        secret = '/approved/private/company/raw-HMM-source / secret matrix 1234'
        out = io.StringIO()
        with patch('ph08_stage4_gateq_replay.rerun',
                   side_effect=RuntimeError(secret)), redirect_stdout(out):
            exit_code = main([secret])
        self.assertEqual(exit_code, 2)
        report = json.loads(out.getvalue())
        self.assertEqual(report['reason_code'], 'PRIVATE_REPLAY_INTERNAL_ERROR')
        self.assertNotIn(secret, out.getvalue())
        self.assertNotIn('matrix', out.getvalue())
        self.assertEqual(report['original_gate_q'], 'FAIL')
        self.assertEqual(report['physical_error_bound'], 'BOUND_NOT_CERTIFIED')

    def test_explicit_reason_code_is_only_reported(self):
        out = io.StringIO()
        with patch('ph08_stage4_gateq_replay.rerun',
                   side_effect=ReplayNotVerified('CHILD_RUNTIME_VERSION_UNAPPROVED')
                   ), redirect_stdout(out):
            self.assertEqual(main(['/approved/private.zip']), 2)
        report = json.loads(out.getvalue())
        self.assertEqual(report['reason_code'], 'CHILD_RUNTIME_VERSION_UNAPPROVED')
        self.assertNotIn('/approved/private.zip', out.getvalue())

    def test_unwritable_report_path_sanitized(self):
        out = io.StringIO()
        with tempfile.TemporaryDirectory() as tmp:
            with patch('ph08_stage4_gateq_replay.rerun',
                       return_value={'status': 'REPRODUCED_HISTORICAL_FAIL',
                                     'original_gate_q': 'FAIL'}), redirect_stdout(out):
                self.assertEqual(main(['/approved/private.zip', '--output-json', tmp]), 2)
        report = json.loads(out.getvalue())
        self.assertEqual(report['reason_code'], 'SANITIZED_REPORT_WRITE_FAILED')

    def test_success_does_not_promote_to_physical_acceptance(self):
        report = fail('MISSING_OR_UNREADABLE_PRIVATE_ARCHIVE')
        self.assertEqual(report['rust_fem'], 'NO_GO')
        self.assertEqual(report['Gate_M'], 'NOT_VERIFIED')
        self.assertEqual(report['contract'], 'NOT_FROZEN')


if __name__ == '__main__':
    unittest.main(verbosity=2)
