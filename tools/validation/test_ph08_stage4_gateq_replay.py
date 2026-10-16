"""Self-contained positive/negative tests; private Stage 4 archive not in CI."""
import unittest
from ph08_stage4_gateq_replay import compare_rows, check_historical_gate


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


if __name__ == '__main__':
    unittest.main(verbosity=2)
