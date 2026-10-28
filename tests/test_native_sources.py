"""Prevent the removed frontend and Node build system from returning unnoticed."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CHECK = ROOT / 'scripts/check_native_sources.py'


class NativeSourceTests(unittest.TestCase):
    def test_project_contains_no_frontend_sources_or_node_dependencies(self):
        result = subprocess.run([sys.executable, str(CHECK)], text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((ROOT / 'src/renderer').exists())
        self.assertFalse((ROOT / 'src/main').exists())
        self.assertFalse((ROOT / 'src/preload').exists())
        self.assertFalse((ROOT / 'android').exists())
        self.assertFalse((ROOT / 'src/python/web_server.py').exists())

    def test_guard_rejects_web_sources_and_node_modules_but_ignores_build_caches(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'target').mkdir()
            (root / 'target/generated.js').write_text('cache')
            clean = subprocess.run([sys.executable, str(CHECK), '--root', directory], capture_output=True)
            self.assertEqual(clean.returncode, 0)
            (root / 'view.css').write_text('legacy')
            (root / 'node_modules').mkdir()
            rejected = subprocess.run([sys.executable, str(CHECK), '--root', directory], text=True, capture_output=True)
            self.assertEqual(rejected.returncode, 1)
            self.assertIn('view.css', rejected.stderr)
            self.assertIn('node_modules', rejected.stderr)


if __name__ == '__main__':
    unittest.main()
