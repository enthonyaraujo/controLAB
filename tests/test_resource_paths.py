"""Shared resources work without renderer, Node or the legacy Android project."""
import importlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

from resource_paths import asset_path, assets_root

ROOT = Path(__file__).resolve().parents[1]


class ResourcePathTests(unittest.TestCase):
    def test_canonical_assets_are_complete_and_match_the_manifest(self):
        import hashlib
        assets = ROOT / 'assets'
        manifest = json.loads((assets / 'manifest.json').read_text())
        self.assertGreater(len(manifest['files']), 20)
        for relative, digest in manifest['files'].items():
            path = assets / relative
            self.assertFalse(path.is_symlink())
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), digest, relative)
        self.assertFalse(any(p.suffix in {'.js', '.css', '.html', '.woff', '.woff2'} for p in assets.rglob('*')))
        self.assertEqual(assets_root(), assets)

    def test_resources_cannot_escape_assets_and_invalid_override_does_not_fallback(self):
        with self.assertRaises(ValueError):
            asset_path('..', 'requirements.txt')
        with tempfile.TemporaryDirectory() as temp, patch.dict(os.environ, {'CONTROLAB_ASSETS': temp}):
            with self.assertRaises(FileNotFoundError):
                assets_root()

    def test_frozen_bundle_and_android_package_have_explicit_roots(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            assets = root / 'assets'
            assets.mkdir()
            (assets / 'manifest.json').write_text('{}')
            with patch.dict(os.environ, {}, clear=True), patch.object(sys, '_MEIPASS', str(root), create=True):
                self.assertEqual(assets_root(), assets)
            package = types.ModuleType('controlab_assets')
            package.__file__ = str(assets / '__init__.py')
            with patch.dict(os.environ, {}, clear=True), patch.dict(sys.modules, {'controlab_assets': package}):
                self.assertEqual(assets_root(), assets)

    def test_scientific_engine_and_formulas_work_in_a_renderer_free_checkout(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            shutil.copytree(ROOT / 'src/python', root / 'src/python', ignore=shutil.ignore_patterns('__pycache__'))
            shutil.copytree(ROOT / 'assets', root / 'assets')
            self.assertFalse((root / 'src/renderer').exists())
            self.assertFalse((root / 'android').exists())
            requests = [
                {'action': 'preview', 'expr': '1/(s+1)'},
                {'action': 'time_response', 'expr': '1/(s+1)', 'language': 'en', 'points': 60},
                {'action': 'render_formulas', 'sources': [r'\frac{1}{s+1}']},
            ]
            env = dict(os.environ)
            env.pop('CONTROLAB_ASSETS', None)
            env['PYTHONPATH'] = str(root / 'src/python')
            completed = subprocess.run([sys.executable, str(root / 'src/python/native_bridge.py')],
                input=''.join(json.dumps(r) + '\n' for r in requests), text=True,
                capture_output=True, cwd=temp, env=env, timeout=90)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            replies = [json.loads(line) for line in completed.stdout.splitlines()]
            self.assertEqual(len(replies), 3)
            for reply in replies:
                self.assertTrue(reply['success'], reply)
            self.assertIn('Time', replies[1]['title'])
            self.assertTrue(replies[1]['image'].startswith('data:image/png;base64,'))
            self.assertTrue(replies[2]['images'][0])


if __name__ == '__main__':
    unittest.main()
