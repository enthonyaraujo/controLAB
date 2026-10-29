#!/usr/bin/env python3
"""Fail if frontend/Node source files reappear in the native project tree."""
import argparse
import os
from pathlib import Path

FORBIDDEN = {'.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.html', '.htm', '.vue', '.svelte', '.webmanifest'}
GENERATED = {'.git', '.venv', '.pytest_cache', '__pycache__', 'target', 'build', 'dist', 'graphify-out'}
CONFIGS = {'package.json', 'package-lock.json', 'yarn.lock', 'pnpm-lock.yaml', 'capacitor.config.json'}


def forbidden_files(root):
    result = []
    for directory, folders, files in os.walk(root):
        folders[:] = [name for name in folders if name not in GENERATED]
        if 'node_modules' in folders:
            result.append(Path(directory) / 'node_modules')
            folders.remove('node_modules')
        for name in files:
            if Path(name).suffix.lower() in FORBIDDEN or name in CONFIGS:
                result.append(Path(directory) / name)
    return sorted(result)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    root = parser.parse_args().root.resolve()
    files = forbidden_files(root)
    if files:
        parser.exit(1, 'Arquivos de frontend/Node encontrados:\n' + '\n'.join(str(p.relative_to(root)) for p in files) + '\n')
    print('Native source tree validated: no JavaScript, TypeScript, CSS, HTML or Node dependencies.')
