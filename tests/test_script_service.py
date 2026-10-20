"""Behavioral checks for the copied .m engine and persistent Python transport."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from script_service import ScriptService


class ScriptRuntimeTests(unittest.TestCase):
    def setUp(self):
        self.service = ScriptService()
        if not self.service.runtime_path().is_file():
            self.skipTest('Compile o motor com npm run build:openmat antes destes testes.')
        self.addCleanup(self.service.close)

    def request(self, operation, **data):
        return self.service.request({'session': 'test', 'operation': operation, **data})

    def test_matrix_script_and_persistent_console_share_workspace(self):
        result = self.request('execute', source='A=[1 2;3 4]; B=[5 6;7 8]; C=A*B; disp(C);')
        self.assertTrue(result['success'], result)
        self.assertIn('19  22', result['output'])
        self.assertIn('43  50', result['output'])
        self.assertEqual({v['name'] for v in result['workspace']}, {'A', 'B', 'C'})
        result = self.request('execute', source='det(A)')
        self.assertTrue(result['success'], result)
        self.assertIn('-2', result['output'])
        self.assertEqual(self.request('clear')['workspace'], [])

    def test_sessions_are_isolated_and_failure_keeps_completed_assignments(self):
        result = self.request('execute', source='before=42; missing_function(1)')
        self.assertFalse(result['success'])
        self.assertGreaterEqual(result['diagnostic']['line'], 1)
        self.assertIn('before', {v['name'] for v in result['workspace']})
        other = self.service.request({'session': 'other', 'operation': 'state'})
        self.assertEqual(other['workspace'], [])
        self.assertIn('42', self.request('execute', source='before')['output'])

    def test_plot_and_control_model_return_structured_points(self):
        result = self.request('execute', source='x=0:0.1:1; plot(x,x.^2);')
        self.assertTrue(result['success'], result)
        self.assertEqual(len(result['plots']), 1)
        self.assertEqual(result['plots'][0]['series'][0]['points'][-1], [1.0, 1.0])
        result = self.request('execute', source='G=tf([1],[1 2 1]); step(G);')
        self.assertTrue(result['success'], result)
        self.assertGreater(len(result['plots'][0]['series'][0]['points']), 2)

    def test_file_roundtrip_and_failed_save_preserve_existing_data(self):
        with tempfile.TemporaryDirectory() as temp:
            file = str(Path(temp) / 'matrizes.m')
            result = self.request('save', path=file, source='A=[1 2;3 4];')
            self.assertTrue(result['success'], result)
            self.assertEqual(self.request('read', path=file)['source'], 'A=[1 2;3 4];')
            collision = self.request('save', path=file, source='new')
            self.assertFalse(collision['success'])
            self.assertTrue(collision['exists'])
            self.assertEqual(Path(file).read_text(), 'A=[1 2;3 4];')
            self.assertTrue(self.request('save', path=file, source='changed', overwrite=True)['success'])
            self.assertEqual(Path(file).read_text(), 'changed')
            listing = self.request('list', path=temp)
            self.assertEqual(listing['files'][0]['name'], 'matrizes.m')
            failed = self.request('save', path=str(Path(temp) / 'missing/a.m'), source='new')
            self.assertFalse(failed['success'])
            self.assertEqual(Path(file).read_text(), 'changed')
            self.assertFalse(self.request('read', path=str(Path(temp) / 'other.txt'))['success'])

    def test_resource_limit_does_not_break_the_next_request(self):
        result = self.request('execute', source='huge=zeros(10000000,1);')
        self.assertFalse(result['success'])
        self.assertTrue(self.request('execute', source='2+2')['success'])
        result = self.request('execute', source='x' * (1024 * 1024 + 1))
        self.assertFalse(result['success'])

    def test_persistent_electron_worker_protocol(self):
        import sys
        worker = Path(__file__).resolve().parents[1] / 'src/python/script_service.py'
        requests = [{'operation': 'execute', 'source': 'A=7;'}, {'operation': 'execute', 'source': 'A+1'}]
        process = subprocess.run([sys.executable, str(worker)], input=''.join(json.dumps(r) + '\n' for r in requests), capture_output=True, text=True, timeout=10, check=True)
        replies = [json.loads(line) for line in process.stdout.splitlines()]
        self.assertEqual(len(replies), 2)
        self.assertIn('8', replies[1]['output'])
