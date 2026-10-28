#!/usr/bin/env python3
"""Print or set the application version (native/Cargo.toml and its Cargo.lock entry).

native/Cargo.toml is the single source: the Rust UI (CARGO_PKG_VERSION), the
Gradle project and the packaging scripts all read the version from it.
"""
import argparse
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'native/Cargo.toml'
LOCK = ROOT / 'Cargo.lock'
PACKAGE = 'controlab-native'
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
    lock, count = re.subn(rf'(\[\[package\]\]\nname = "{PACKAGE}"\nversion = )"[^"]+"', rf'\g<1>"{version}"', lock)
    if count != 1:
        raise RuntimeError(f'Pacote {PACKAGE} ausente em {LOCK}')
    MANIFEST.write_text(manifest, encoding='utf-8')
    LOCK.write_text(lock, encoding='utf-8')


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
