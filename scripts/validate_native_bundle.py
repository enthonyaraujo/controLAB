#!/usr/bin/env python3
"""Check a native distribution from an unrelated directory without source/Python fallbacks."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile


def validate(bundle, ui=False):
    metadata = json.loads((bundle / 'bundle.json').read_text())
    resources = bundle / metadata['resources']
    manifest = json.loads((resources / 'manifest.json').read_text())
    for name, digest in manifest['files'].items():
        assert hashlib.sha256((resources / name).read_bytes()).hexdigest() == digest, name
    for item in bundle.rglob('*'):
        assert item.suffix.lower() not in {'.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.html', '.htm', '.vue', '.svelte', '.webmanifest'}, f'Web resource in native bundle: {item}'
    requests = [
        {'action': 'preview', 'expr': '1/(s+1)'},
        {'action': 'time_response', 'expr': '1/(s+1)', 'language': 'en', 'points': 60},
        {'action': 'render_formulas', 'sources': [r'\frac{1}{s+1}']},
        {'action': 'calculate', 'expr': '1/(s*(s+1))'},
        {'action': 'routh_hurwitz', 'expr': 's^3+3*s^2+2*s+1', 'has_k': False},
        {'action': 'frequency_response', 'expr': '1/(s+1)', 'points': 150},
        {'action': 'controller_design', 'plant_expr': '5/(s*(s+1))', 'controller_type': 'lead', 'ts': 2.0, 'xi': 0.5, 'points': 150},
        {'action': 'state_space', 'mode': 'matrices', 'A': '[[0,1],[-2,-3]]', 'B': '[[0],[1]]', 'C': '[[1,0]]', 'D': '[[0]]'},
    ]
    with tempfile.TemporaryDirectory(prefix='controlab-bundle-check-') as temp:
        env = dict(os.environ)
        env.pop('PYTHONPATH', None)
        env.pop('CONTROLAB_ASSETS', None)
        env.pop('CONTROLAB_ENGINE', None)
        env['CONTROLAB_PYTHON'] = str(Path(temp) / 'no-python')
        env['CONTROLAB_NATIVE_BRIDGE'] = str(Path(temp) / 'no-source-bridge')
        # Same protocol encoding the UI sets; Windows pipes otherwise use the ANSI code page.
        env['PYTHONIOENCODING'] = 'utf-8'
        completed = subprocess.run([str(bundle / metadata['engine'])],
            input=''.join(json.dumps(request) + '\n' for request in requests),
            text=True, encoding='utf-8', capture_output=True, cwd=temp, env=env, timeout=90)
        assert completed.returncode == 0, completed.stderr
        replies = [json.loads(line) for line in completed.stdout.splitlines()]
        assert len(replies) == len(requests), completed.stderr
        assert all(reply.get('success') for reply in replies), replies
        assert 'Time' in replies[1]['title']
        assert replies[2]['images'][0]
        if ui:
            screenshot = bundle / 'validation-ui.png'
            env.update(CONTROLAB_SCREENSHOT=str(screenshot), CONTROLAB_SCREENSHOT_PAGE='time',
                CONTROLAB_SCREENSHOT_SIZE='1024x768', CONTROLAB_SCREENSHOT_THEME='dark')
            completed = subprocess.run([str(bundle / metadata['ui'])], cwd=temp, env=env,
                text=True, encoding='utf-8', errors='replace', capture_output=True, timeout=60)
            assert completed.returncode == 0, completed.stderr
            assert screenshot.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
            # .m scripts run inside the UI process: they must work with no engine at all.
            script = Path(temp) / 'validation-script.png'
            env.update(CONTROLAB_SCREENSHOT=str(script), CONTROLAB_ENGINE=str(Path(temp) / 'no-engine'),
                CONTROLAB_SCREENSHOT_SCRIPT='A=[1 2;3 4];\ndisp(A*A)\nstep(tf([1],[1 2 1]))')
            env.pop('CONTROLAB_SCREENSHOT_PAGE')
            completed = subprocess.run([str(bundle / metadata['ui'])], cwd=temp, env=env,
                text=True, encoding='utf-8', errors='replace', capture_output=True, timeout=60)
            assert completed.returncode == 0, completed.stderr
            assert script.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
    print('Bundle validated: independent resources, all six scientific engines and formulas' +
          (', native UI and in-process .m scripts' if ui else ''))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--ui', action='store_true')
    args = parser.parse_args()
    validate(args.bundle.resolve(), args.ui)
