"""Keep every copy of the application version equal to the workspace Cargo.toml."""
from pathlib import Path
import re
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))

from app_version import VERSION  # noqa: E402
from set_version import current_version  # noqa: E402


class AppVersionTests(unittest.TestCase):
    def test_python_engine_matches_cargo_manifest(self):
        self.assertEqual(VERSION, current_version())

    def test_cargo_lock_matches_cargo_manifest(self):
        lock = (ROOT / 'Cargo.lock').read_text(encoding='utf-8')
        for package in ('controlab-native', 'mscript'):
            match = re.search(rf'name = "{package}"\nversion = "([^"]+)"', lock)
            self.assertEqual(match.group(1), current_version(), package)

    def test_sources_do_not_hardcode_the_version(self):
        pattern = re.compile(r'v' + re.escape(current_version()) + r'\b')
        sources = [*(ROOT / 'native/src').glob('*.rs'), *(ROOT / 'crates').rglob('*.rs'), *(ROOT / 'src/python').glob('*.py')]
        offenders = [p.name for p in sources if p.name != 'app_version.py' and pattern.search(p.read_text(encoding='utf-8'))]
        self.assertEqual(offenders, [])


if __name__ == '__main__':
    unittest.main()
