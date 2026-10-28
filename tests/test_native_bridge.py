"""Native transport tests: strict JSON, persistence, errors and EOF."""
import json
from pathlib import Path
import subprocess
import sys
import unittest
import importlib.util

BRIDGE = Path(__file__).resolve().parents[1] / 'src/python/native_bridge.py'

class NativeBridgeTests(unittest.TestCase):
    def exchange(self, requests):
        return self.exchange_raw(''.join(json.dumps(r) + '\n' for r in requests))

    def exchange_raw(self, source):
        process = subprocess.run([sys.executable, str(BRIDGE)],
            input=source, text=True, capture_output=True, timeout=90)
        self.assertEqual(process.returncode, 0, process.stderr)
        def invalid_constant(value):
            raise AssertionError(f'Non-strict JSON constant: {value}')
        return [json.loads(line, parse_constant=invalid_constant)
                for line in process.stdout.splitlines()]

    def test_malformed_requests_do_not_desynchronize_the_stream(self):
        source = '{broken\n[]\nnull\n{"action":"preview","expr":NaN}\n'
        source += json.dumps({'action': 'preview', 'expr': '3/(s+1)'}) + '\n'
        replies = self.exchange_raw(source)
        self.assertEqual(len(replies), 5)
        self.assertTrue(all(not item['success'] for item in replies[:4]))
        self.assertEqual(replies[-1]['num'], [3.0])

    def test_oversized_request_is_drained_before_the_next_line(self):
        replies = self.exchange_raw(' ' * (8 * 1024 * 1024 + 1) + '\n'
                                    + '{"action":"presets"}\n')
        self.assertEqual(len(replies), 2)
        self.assertFalse(replies[0]['success'])
        self.assertIn('8 MiB', replies[0]['error'])
        self.assertTrue(replies[1]['success'])

    def test_scientific_values_are_normalized_without_android_import(self):
        import numpy as np
        android_before = sys.modules.get('android_bridge')
        spec = importlib.util.spec_from_file_location('native_transport_test', BRIDGE)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        normalized = module._json_safe({
            'nan': np.float64('nan'), 'inf': np.float64('inf'),
            'negative_inf': float('-inf'), 'matrix': np.array([[1, 2]]),
            'scalar_array': np.array(3), 'complex': np.complex128(1+2j),
            'bool': np.bool_(True), 'int': np.int64(7),
        })
        self.assertEqual(normalized, {
            'nan': None, 'inf': '∞', 'negative_inf': '-∞', 'matrix': [[1, 2]],
            'scalar_array': 3, 'complex': {'re': 1.0, 'im': 2.0},
            'bool': True, 'int': 7,
        })
        json.dumps(normalized, allow_nan=False)
        self.assertIs(sys.modules.get('android_bridge'), android_before)

    def test_engine_and_shutdown_output_cannot_pollute_protocol(self):
        code = '''
import atexit, sys, types
sys.path.insert(0, sys.argv[1])
import native_bridge
engine = types.ModuleType('python_bridge')
def dispatch(data):
    print('engine diagnostic')
    return {'success': True, 'value': data['value']}
engine.dispatch = dispatch
sys.modules['python_bridge'] = engine
atexit.register(lambda: print('shutdown diagnostic'))
native_bridge.main()
'''
        process = subprocess.run([sys.executable, '-c', code, str(BRIDGE.parent)],
                                 input='{"value":7}\n{"value":9}\n', text=True,
                                 capture_output=True, timeout=15)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual([json.loads(line) for line in process.stdout.splitlines()],
                         [{'success': True, 'value': 7}, {'success': True, 'value': 9}])
        self.assertIn('engine diagnostic', process.stderr)
        self.assertIn('shutdown diagnostic', process.stderr)

    def test_requests_survive_errors_and_return_strict_json(self):
        replies = self.exchange([{'action':'preview','expr':'1/(s+1)'},
            {'action':'does_not_exist'}, {'action':'preview','expr':'2/(s+2)'}])
        self.assertEqual(len(replies), 3)
        self.assertTrue(replies[0]['success'])
        self.assertFalse(replies[1]['success'])
        self.assertEqual(replies[2]['num'], [2.0])

    def test_script_session_is_persistent(self):
        replies = self.exchange([{'action':'script_request','operation':'execute','source':'a=7;'},
            {'action':'script_request','operation':'execute','source':'a+2'}])
        self.assertTrue(replies[0]['success'], replies[0])
        self.assertTrue(replies[1]['success'], replies[1])
        self.assertIn('9', replies[1]['output'])

    def test_all_six_engines_return_plots_and_recover_from_invalid_input(self):
        requests = [
            {'action': 'calculate', 'expr': '1/(s*(s+1))'},
            {'action': 'routh_hurwitz', 'expr': 's^3+3*s^2+2*s+1', 'has_k': False},
            {'action': 'time_response', 'expr': '4/(s^2+2*s+4)', 'points': 150},
            {'action': 'frequency_response', 'expr': '1/(s+1)', 'points': 150},
            {'action': 'controller_design', 'plant_expr': '5/(s*(s+1))',
             'controller_type': 'lead', 'ts': 2.0, 'xi': 0.5, 'points': 150},
            {'action': 'state_space', 'mode': 'matrices',
             'A': '[[0,1],[-2,-3]]', 'B': '[[0],[1]]', 'C': '[[1,0]]', 'D': '[[0]]'},
        ]
        sequence = []
        for request in requests:
            invalid = dict(request, expr=')', plant_expr=')', A='invalid matrix')
            sequence.extend([request, invalid])
        replies = self.exchange(sequence)
        self.assertEqual(len(replies), 12)
        for request, valid, invalid in zip(requests, replies[::2], replies[1::2]):
            with self.subTest(engine=request['action']):
                self.assertTrue(valid['success'], valid)
                self.assertTrue(valid.get('image') or valid.get('plots') or valid.get('plot_image'),
                                f"Scientific result missing for {request['action']}")
                self.assertFalse(invalid['success'], invalid)
                self.assertTrue(invalid.get('error'))

if __name__ == '__main__':
    unittest.main()
