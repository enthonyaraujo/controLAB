#!/usr/bin/env python3
"""Print or set the application version.

The workspace Cargo.toml ([workspace.package]) is the source: every crate,
the Rust UI (CARGO_PKG_VERSION), the Gradle project and the packaging scripts
read it. Setting a version also updates the workspace entries in Cargo.lock
and the Python engine's copy in src/python/app_version.py.
"""
import argparse
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'Cargo.toml'
LOCK = ROOT / 'Cargo.lock'
PYTHON = ROOT / 'src/python/app_version.py'
PACKAGES = ('controlab-native', 'mscript')
SEMVER = re.compile(r'^\d+\.\d+\.\d+$')


def current_version():
    match = re.search(r'^version\s*=\s*"([^"]+)"', MANIFEST.read_text(encoding='utf-8'), re.M)
    if not match:
        raise RuntimeError(f'Versão ausente em {MANIFEST}')
    return match.group(1)


def set_version(version):
    manifest = MANIFEST.read_text(encoding='utf-8')
    manifest, count = re.subn(r'^(version\s*=\s*)"[^"]+"', rf'\g<1>"{version}"', manifest, count=1, flags=re.M)
    if count != 1:
        raise RuntimeError(f'Versão ausente em {MANIFEST}')
    lock = LOCK.read_text(encoding='utf-8')
    for package in PACKAGES:
        lock, count = re.subn(rf'(\[\[package\]\]\nname = "{package}"\nversion = )"[^"]+"', rf'\g<1>"{version}"', lock)
        if count != 1:
            raise RuntimeError(f'Pacote {package} ausente em {LOCK}')
    python, count = re.subn(r'^VERSION = "[^"]+"', f'VERSION = "{version}"', PYTHON.read_text(encoding='utf-8'), flags=re.M)
    if count != 1:
        raise RuntimeError(f'VERSION ausente em {PYTHON}')
    MANIFEST.write_text(manifest, encoding='utf-8')
    LOCK.write_text(lock, encoding='utf-8')
    PYTHON.write_text(python, encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('version', nargs='?', help='X.Y.Z; a leading "v" (Git tag) is accepted')
    args = parser.parse_args()
    if args.version is None:
        print(current_version())
        sys.exit(0)
    version = args.version.removeprefix('v')
    if not SEMVER.match(version):
        parser.error(f'Versão inválida (esperado X.Y.Z): {args.version}')
    set_version(version)
    print(version)
