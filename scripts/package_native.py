#!/usr/bin/env python3
"""Package the Cargo application and an independent scientific Python engine.

No dependency installation, Electron, JavaScript or web assets are involved.
Build Python packages must already be installed in the selected interpreter.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def pyinstaller_command(work, output):
    return [sys.executable, '-m', 'PyInstaller', '--noconfirm', '--clean', '--onedir',
        '--name', 'controlab-engine', '--distpath', str(output),
        '--workpath', str(work / 'work'), '--specpath', str(work / 'spec'),
        '--collect-data', 'matplotlib', '--collect-data', 'control',
        '--exclude-module', 'matplotlib.backends.backend_webagg',
        '--exclude-module', 'matplotlib.backends.backend_webagg_core',
        '--exclude-module', 'matplotlib.backends.backend_nbagg',
        '--exclude-module', 'tkinter', '--exclude-module', 'PyQt5',
        '--exclude-module', 'PyQt6', '--exclude-module', 'PySide6',
        '--exclude-module', 'pyarrow', '--exclude-module', 'IPython',
        '--hidden-import', 'matplotlib.backends.backend_svg',
        '--hidden-import', 'matplotlib.backends.backend_agg',
        '--hidden-import', 'matplotlib.backends.backend_pdf',
        '--add-data', str(ROOT / 'assets') + os.pathsep + 'assets',
        '--paths', str(ROOT / 'src/python'), str(ROOT / 'src/python/native_bridge.py')]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--offline', action='store_true', help='Cargo cached dependencies only')
    parser.add_argument('--skip-build', action='store_true', help='Use an already built Cargo workspace')
    parser.add_argument('--output', type=Path, default=ROOT / 'dist/native/controlab')
    args = parser.parse_args()
    if importlib.util.find_spec('PyInstaller') is None:
        parser.error('PyInstaller ausente. Prepare requirements-build.txt no ambiente Python de build.')
    output = args.output.resolve()
    if output.exists() and any(output.iterdir()):
        parser.error(f'A pasta de saída deve estar vazia para preservar arquivos existentes: {output}')
    if output == ROOT or ROOT.is_relative_to(output):
        parser.error('A saída não pode conter o repositório de origem.')
    if not args.skip_build:
        subprocess.run(['cargo', 'build', '--locked', '--workspace'] +
            (['--release'] if args.release else []) + (['--offline'] if args.offline else []), cwd=ROOT, check=True)
    target = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target'))
    if not target.is_absolute():
        target = ROOT / target
    profile = target / ('release' if args.release else 'debug')
    suffix = '.exe' if sys.platform == 'win32' else ''
    binaries = [profile / ('controlab' + suffix), profile / ('controlab-runtime' + suffix)]
    for binary in binaries:
        if not binary.is_file():
            parser.error(f'Executável ausente: {binary}. Compile o workspace Cargo primeiro.')
    work = ROOT / 'build/native-package'
    subprocess.run(pyinstaller_command(work, work / 'dist'), cwd=ROOT, check=True)
    output.mkdir(parents=True, exist_ok=True)
    for binary in binaries:
        shutil.copy2(binary, output / binary.name)
    shutil.copytree(work / 'dist/controlab-engine', output / 'engine')
    # Matplotlib's scientific Agg/SVG/PDF backends need no WebAgg frontend.
    for payload in (output/'engine', work/'dist/controlab-engine'):
        for relative in ('matplotlib/mpl-data/web_backend', 'matplotlib/backends/web_backend', 'matplotlib/mpl-data/plot_directive'):
            web_backend=payload/'_internal'/relative
            if web_backend.exists():
                shutil.rmtree(web_backend)
    shutil.copy2(ROOT / 'LICENSE', output / 'LICENSE')
    (output/'licenses').mkdir(exist_ok=True)
    for name in ('LICENSE', 'THIRD_PARTY_NOTICES.txt'):
        shutil.copy2(ROOT/'vendor/openmat'/name,output/'licenses'/('openmat-'+name))
    manifest = {'version': '5.0.0', 'platform': sys.platform, 'profile': profile.name,
        'ui': binaries[0].name, 'runtime': binaries[1].name,
        'engine': 'engine/controlab-engine' + suffix,
        'resources': 'engine/_internal/assets'}
    (output / 'bundle.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(output)


if __name__ == '__main__':
    main()
