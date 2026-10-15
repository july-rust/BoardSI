"""Mutation tests run against REAL FEM-emitted JSON, before FVM invocation."""
import copy
import json
import pathlib
import sys
import unittest

from m2_ph07_inputs import expected_cases, validate_input


class InputMutation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root=pathlib.Path(sys.argv[1])

    def test_all_cases_and_single_field_mutations(self):
        count=0
        for name in expected_cases():
            data=json.loads((self.root/(name+'.json')).read_text())
            validate_input(name,data)
            paths=[('width_m',),('copper_thickness_m',),('B_m',),('H_m',),
                   ('air_epsilon_r',),('n',),('nx',),('nz',),('dx_m',),
                   ('nodes',),('triangles',),('bottom_ground_v',),('signal_v',),
                   ('top_boundary',),('side_boundary',),('case',),('model',)]
            for i in range(len(data['layers'])):
                paths.extend([('layers',i,'height_m'),('layers',i,'epsilon_r')])
            if data['mask'] is not None:
                paths.extend([('mask','thickness_m'),('mask','epsilon_r')])
            for path in paths:
                changed=copy.deepcopy(data)
                node=changed
                for key in path[:-1]:
                    node=node[key]
                value=node[path[-1]]
                node[path[-1]]=value+0.000001 if isinstance(value,(int,float)) else 'MUTATED'
                with self.subTest(case=name,path=path):
                    with self.assertRaises((ValueError,KeyError)):
                        validate_input(name,changed)
                count+=1
            for key,value in [('mask',None if data['mask'] else {'thickness_m':0.000025,'epsilon_r':3.3}),
                              ('layers',[]),('width_m',float('nan')),('width_m',float('inf'))]:
                changed=copy.deepcopy(data)
                changed[key]=value
                with self.assertRaises((ValueError,KeyError)):
                    validate_input(name,changed)
                count+=1
        print(f'PH07 input mutation rejections PASS: {count}; original cases: {len(expected_cases())}')

    def test_missing_and_unknown(self):
        d=json.loads((self.root/'ph07-A-n32.json').read_text())
        del d['top_boundary']
        with self.assertRaises(KeyError):
            validate_input('ph07-A-n32',d)
        with self.assertRaises(KeyError):
            validate_input('ph07-unknown',d)


if __name__=='__main__':
    # Keep evidence directory for setUpClass; unittest receives no extra argv.
    unittest.main(argv=[sys.argv[0]],verbosity=2)
