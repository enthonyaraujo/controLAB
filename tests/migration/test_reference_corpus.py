"""Integrity checks for the immutable Python migration oracle (no Python engine import)."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
REFERENCE = ROOT / 'tests/reference/python'


class ReferenceCorpusTests(unittest.TestCase):
    def test_manifest_covers_all_modules_and_presets(self):
        manifest = json.loads((REFERENCE / 'manifest.json').read_text())
        self.assertEqual(manifest['schema_version'], 1)
        self.assertEqual(set(manifest['modules']), {'lgr', 'routh', 'time', 'frequency', 'controller', 'state_space'})
        self.assertTrue(manifest['source_commit'])
        for module in manifest['modules'].values():
            self.assertTrue(module['preset_ids'])
            self.assertEqual(set(module['preset_ids']), set(module['covered_preset_ids']))
        self.assertGreaterEqual(len(manifest['cases']), 80)
        for case in manifest['cases']:
            self.assertEqual(hashlib.sha256((REFERENCE / case['path']).read_bytes()).hexdigest(), case['sha256'])

    def test_outputs_and_separate_assets_are_complete(self):
        manifest = json.loads((REFERENCE / 'manifest.json').read_text())
        def visit(value):
            if isinstance(value, dict):
                if '$asset' in value:
                    data = (REFERENCE / value['$asset']).read_bytes()
                    self.assertEqual(hashlib.sha256(data).hexdigest(), value['sha256'])
                    self.assertEqual(len(data), value['bytes'])
                else:
                    for item in value.values():
                        visit(item)
            elif isinstance(value, list):
                for item in value:
                    visit(item)
        arrays = {'time': 0, 'frequency': 0}
        for entry in manifest['cases']:
            case = json.loads((REFERENCE / entry['path']).read_text())
            self.assertIn('input', case)
            self.assertIn('output', case)
            visit(case['output'])
            if 'numerical' in case:
                kind = case['numerical']['kind']
                arrays[kind] += 1
                key = 'time' if kind == 'time' else 'omega'
                self.assertGreaterEqual(len(case['numerical'][key]), 200)
        self.assertGreaterEqual(arrays['time'], 5)
        self.assertGreaterEqual(arrays['frequency'], 4)


if __name__ == '__main__':
    unittest.main()
