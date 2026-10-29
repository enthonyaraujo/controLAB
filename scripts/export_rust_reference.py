#!/usr/bin/env python3
"""Snapshot the Python scientific protocol before Rust migration; never import user scripts.

Run with the existing .venv. Scientific source and installed dependencies are unchanged.
"""
from __future__ import annotations

import argparse
import ast
import base64
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/python'
sys.path.insert(0, str(SOURCE))
os.environ['MPLBACKEND'] = 'Agg'
# Matplotlib honors this for SVG metadata; stable salt prevents random SVG IDs.
os.environ['SOURCE_DATE_EPOCH'] = '0'


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, indent=2) + '\n').encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def cases_and_presets():
    from python_bridge import PRESETS
    from routh_hurwitz import ROUTH_PRESETS
    from time_response import TIME_RESPONSE_PRESETS
    from frequency_response import FREQUENCY_PRESETS
    from controller_design import CONTROLLER_PRESETS
    from state_space import STATE_SPACE_PRESETS
    cases, modules = [], {}
    def add(name, payload, **metadata):
        cases.append({'id': name, 'input': payload, **metadata})
    groups = [
        ('lgr', 'presets', 'calculate', PRESETS),
        ('routh', 'routh_presets', 'routh_hurwitz', ROUTH_PRESETS),
        ('time', 'time_response_presets', 'time_response', TIME_RESPONSE_PRESETS),
        ('frequency', 'frequency_response_presets', 'frequency_response', FREQUENCY_PRESETS),
        ('controller', 'controller_presets', 'controller_design', CONTROLLER_PRESETS),
        ('state_space', 'state_space_presets', 'state_space', STATE_SPACE_PRESETS),
    ]
    for module, catalog_action, action, presets in groups:
        add(module + '_catalog', {'action': catalog_action}, module=module)
        modules[module] = {'preset_ids': list(presets), 'covered_preset_ids': list(presets)}
        for index, (name, config) in enumerate(presets.items()):
            payload = {'action': action}
            if module == 'lgr':
                payload.update(mode='preset', preset_key=name)
            elif module == 'routh':
                payload.update(expr=config['expr'])
            elif module in ('time', 'frequency'):
                payload.update(expr=config, points=250)
            elif module == 'controller':
                payload.update(config, points=250)
            else:
                payload.update(config)
            add(f'{module}_preset_{index:02}', payload, module=module, preset_id=name)

    # Only AST literals are read from reviewed repository tests. No test or user code is executed.
    for file in ('test_controller_design.py', 'test_time_response.py', 'test_frequency_response.py', 'test_state_space.py', 'test_routh_hurwitz.py'):
        tree = ast.parse((ROOT / 'tests' / file).read_text())
        module = {'test_controller_design.py': 'controller', 'test_time_response.py': 'time', 'test_frequency_response.py': 'frequency', 'test_state_space.py': 'state_space', 'test_routh_hurwitz.py': 'routh'}[file]
        for function in ast.walk(tree):
            if not isinstance(function, ast.FunctionDef) or not function.name.startswith('test_'):
                continue
            counter = 0
            for call in ast.walk(function):
                if not isinstance(call, ast.Call) or not isinstance(call.func, ast.Name):
                    continue
                name = call.func.id
                try:
                    if name == 'dispatch' and call.args:
                        payload = ast.literal_eval(call.args[0])
                    elif name in ('design_controller', 'analyze_time_response', 'analyze_frequency_response', 'analyze_routh_hurwitz', 'analyze_state_space'):
                        payload = {kw.arg: ast.literal_eval(kw.value) for kw in call.keywords}
                        payload['action'] = {'design_controller':'controller_design', 'analyze_time_response':'time_response', 'analyze_frequency_response':'frequency_response', 'analyze_routh_hurwitz':'routh_hurwitz', 'analyze_state_space':'state_space'}[name]
                        if call.args:
                            payload['plant_expr' if name == 'design_controller' else 'expr'] = ast.literal_eval(call.args[0])
                        for old, new in [('expression','expr'), ('a','A'), ('b','B'), ('c','C'), ('d','D')]:
                            if old in payload:
                                payload[new] = payload.pop(old)
                    else:
                        continue
                except (ValueError, TypeError):
                    continue
                if payload.get('action') in ('time_response', 'frequency_response', 'controller_design'):
                    payload.setdefault('points', 250)
                add(f'{module}_{function.name}_{counter}', payload, module=module, source=f'tests/{file}:{call.lineno}')
                counter += 1

    extra = [
        ('preview_unicode', {'action':'preview','expr':'G(s) = (s + 2) ÷ (s³ + 5s² + 4s)'}, 'lgr'),
        ('preview_scientific', {'action':'preview','expr':'1e-3(s + 2)/(s(s + 1))'}, 'lgr'),
        ('preview_parts', {'action':'preview','mode':'parts','numerator':'s+2','denominator':'s(s+1)(s+4)'}, 'lgr'),
        ('preview_coeffs', {'action':'preview','mode':'coeffs','num':'1, 2','den':'1, 5, 4, 0'}, 'lgr'),
        ('preview_zpk', {'action':'preview','mode':'zpk','zeros':'-2','poles':'0, -1, -4','k':1}, 'lgr'),
        ('preview_unknown_mode', {'action':'preview','mode':'unknown'}, 'lgr'),
        ('preview_invalid_symbol', {'action':'preview','expr':'unsupported(s)'}, 'lgr'),
        ('preview_empty', {'action':'preview','expr':''}, 'lgr'),
        ('preview_zero_denominator', {'action':'preview','expr':'1/0'}, 'lgr'),
        ('lgr_complex_zero', {'action':'calculate','expr':'(s^2+6s+12)/(s*(s+2)^2)'}, 'lgr'),
        ('time_ramp_light', {'action':'time_response','expr':'1/(s+1)','response_type':'ramp','theme':'light','width':640,'dpi':100,'points':250,'settling_threshold':0.05}, 'time'),
        ('time_impulse', {'action':'time_response','expr':'1/(s+1)','response_type':'impulse','points':250}, 'time'),
        ('time_unstable', {'action':'time_response','expr':'1/(s-1)','final_time':3,'points':250}, 'time'),
        ('time_invalid_threshold', {'action':'time_response','settling_threshold':0.1}, 'time'),
        ('time_invalid_window', {'action':'time_response','final_time':-1}, 'time'),
        ('frequency_invalid_range', {'action':'frequency_response','omega_min':0}, 'frequency'),
        ('frequency_too_many_decades', {'action':'frequency_response','omega_min':1e-8,'omega_max':1e8}, 'frequency'),
        ('frequency_unstable', {'action':'frequency_response','expr':'1/(s-1)','points':250}, 'frequency'),
        ('state_observable', {'action':'state_space','mode':'transfer','expr':'1/(s^2+3s+2)','canonical_form':'observable'}, 'state_space'),
        ('state_invalid_dimension', {'action':'state_space','A':'[[1,2,3]]'}, 'state_space'),
        ('state_invalid_poles', {'action':'state_space','desired_poles':'-1'}, 'state_space'),
        ('state_uncontrollable', {'action':'state_space','A':'[[-1,0],[0,-2]]','B':'[[1],[0]]','C':'[[1,0]]','D':'[[0]]','desired_poles':'-3,-4'}, 'state_space'),
        ('routh_invalid_expression', {'action':'routh_hurwitz','expr':'x+1'}, 'routh'),
        ('controller_inner_loop_desired_pole', {'action':'controller_design','plant_expr':'18/((2s+11)(2s+1)) / (1 + 0.5*18/((2s+11)(2s+1))) * 1/(30s)','controller_type':'PD','desired_pole':'-1+1j','points':200}, 'controller'),
        ('controller_invalid_type', {'action':'controller_design','controller_type':'unknown'}, 'controller'),
        ('controller_compensator', {'action':'controller_design','plant_expr':'1/(s+1)','design_type':'compensator','points':250}, 'controller'),
        ('formulas', {'action':'render_formulas','sources':[r'\frac{1}{s+1}',r'\begin{bmatrix}1&0\\0&1\end{bmatrix}',r'\unknowncommand{x}','',7]}, 'presentation'),
        ('formulas_invalid_sources', {'action':'render_formulas','sources':'x'}, 'presentation'),
        ('unknown_action', {'action':'unknown'}, 'protocol'),
    ]
    for method in ('zn_reaction','zn_critical','zn_critical_manual','cohen_coon','chr'):
        for controller in ('P','PI','PD','PID'):
            extra.append((f'legacy_{method}_{controller}', {'action':'controller_design','plant_expr':'1/(s*(s+1)*(s+5))','method':method,'controller_type':controller,'points':250}, 'controller'))
    for controller in ('P','PI','PD','PID'):
        extra.append((f'legacy_chr20_{controller}', {'action':'controller_design','plant_expr':'1/(s*(s+1)*(s+5))','method':'chr','controller_type':controller,'chr_response':'20','points':250}, 'controller'))
    for kind in ('lead','lag'):
        for domain in ('frequency','time'):
            extra.append((f'lead_lag_{kind}_{domain}', {'action':'controller_design','plant_expr':'1/(s+1)','design_type':'lead_lag','compensator_type':kind,'design_domain':domain,'points':250}, 'controller'))
    for command in ('', 'help', 'ajuda', 'clear', 'clc', 'whos'):
        extra.append(('console_' + (command or 'empty'), {'action':'console_exec','command':command}, 'tools'))
    for index, value in enumerate(('-1+1j', '-1 ± j', '-1-1j', '-1+sqrt(3)j', 's = -1 ± √3 i', '-2 + 3,5j', '', '1+1j', '-1')):
        extra.append((f'parse_desired_pole_{index}', {'action':'__core__','method':'parse_desired_pole','args':[value]}, 'controller'))
    for index, value in enumerate(('[1, 2, 3]', '1, 2; 3, 4', '[[0,1],[-2,-3]]', 'bad')):
        extra.append((f'parse_matrix_{index}', {'action':'__core__','method':'parse_matrix','args':[value,'A']}, 'state_space'))
    for index, value in enumerate(('1, 2, 3', '[1 2 3]', '1e-3, -2, 0', '', 'bad')):
        extra.append((f'parse_coeffs_{index}', {'action':'__core__','method':'parse_coeffs_str','args':[value]}, 'lgr'))
    for name, payload, module in extra:
        add(name, payload, module=module)
    # Identical requests are represented once; provenance for the first occurrence is retained.
    unique, seen = [], set()
    for case in cases:
        key = encoded(case['input'])
        if key not in seen:
            seen.add(key)
            unique.append(case)
    return unique, modules


def extract_assets(value, output, key=''):
    if isinstance(value, dict):
        return {k: extract_assets(v, output, k) for k, v in value.items()}
    if isinstance(value, list):
        return [extract_assets(v, output, key) for v in value]
    if not isinstance(value, str):
        return value
    if '<svg' in value[:1000]:
        data, extension, encoding = value.encode(), 'svg', 'utf-8'
    elif value.startswith('data:image/png;base64,'):
        data, extension, encoding = base64.b64decode(value.split(',', 1)[1]), 'png', 'data:image/png;base64'
    elif key in ('images', 'image') and value.startswith('iVBOR'):
        data, extension, encoding = base64.b64decode(value), 'png', 'base64'
    else:
        return value
    sha = digest(data)
    path = f'assets/{sha}.{extension}'
    (output / path).write_bytes(data)
    return {'$asset': path, 'sha256': sha, 'bytes': len(data), 'encoding': encoding}


def numerical(payload):
    """Capture actual engine grids and full response vectors, not samples or plot pixels."""
    import control as ct
    import numpy as np
    from control_utils import parse_siso_transfer_function, recommended_time_vector, response_arrays
    action = payload.get('action')
    if action not in ('time_response', 'frequency_response'):
        return None
    plant, num, den, *_ = parse_siso_transfer_function(payload.get('expr') or ('4/(s^2+2s+4)' if action == 'time_response' else '10/(s*(s+2)*(s+5))'))
    if action == 'time_response':
        time = recommended_time_vector(plant, payload.get('final_time'), payload.get('points', 900))
        return {'kind':'time', 'time':time, 'step':response_arrays(ct.step_response(plant,timepts=time))[1], 'impulse':response_arrays(ct.impulse_response(plant,timepts=time))[1], 'ramp':response_arrays(ct.forced_response(plant,timepts=time,inputs=time))[1]}
    from frequency_response import _asymptotic_magnitude
    omega = np.logspace(np.log10(float(payload.get('omega_min',0.01))), np.log10(float(payload.get('omega_max',100))), int(np.clip(int(payload.get('points',900) or 900),200,4000)))
    response = ct.frequency_response(plant,omega)
    magnitude = np.asarray(response.magnitude).squeeze()
    phase = np.unwrap(np.asarray(response.phase).squeeze()) * 180 / np.pi
    db = 20*np.log10(np.maximum(magnitude,np.finfo(float).tiny))
    return {'kind':'frequency', 'omega':omega, 'magnitude':magnitude, 'phase_deg':phase, 'magnitude_db':db, 'asymptote_db':_asymptotic_magnitude(num,den,omega,db), 'complex_response':magnitude*np.exp(1j*np.deg2rad(phase)), 'closed_loop_magnitude':np.asarray(ct.frequency_response(ct.feedback(plant,1),omega).magnitude).squeeze()}


def export(output, only=None):
    import matplotlib
    matplotlib.rcParams['svg.hashsalt'] = 'controlab-python-reference-v1'
    import matplotlib.pyplot as plt
    from native_bridge import _json_safe
    from native_presentation import dispatch_native
    cases, modules = cases_and_presets()
    if only:
        missing = set(only) - {case['id'] for case in cases}
        if missing:
            raise SystemExit('Unknown case IDs: ' + ', '.join(sorted(missing)))
    output.mkdir(parents=True, exist_ok=True)
    (output / 'cases').mkdir(exist_ok=True)
    (output / 'assets').mkdir(exist_ok=True)
    manifest = {'schema_version':1, 'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(), 'source_sha256':{str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in sorted(SOURCE.glob('*.py'))}, 'fixture_sources_sha256':{str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in sorted((ROOT/'tests').glob('test_*.py'))}, 'generator_sha256':digest(Path(__file__).read_bytes()), 'python':sys.version.split()[0], 'packages':{name:importlib.metadata.version(name) for name in ('numpy','scipy','sympy','control','matplotlib','Pillow')}, 'determinism':{'SOURCE_DATE_EPOCH':'0','svg.hashsalt':'controlab-python-reference-v1'}, 'modules':modules, 'cases':[]}
    for case in cases:
        if only and case['id'] not in only:
            continue
        payload = case['input']
        try:
            if payload.get('action') == '__core__':
                from controller_design import parse_desired_pole
                from state_space import parse_matrix
                from lgr_engine import parse_coeffs_str
                helpers = {'parse_desired_pole':parse_desired_pole, 'parse_matrix':parse_matrix, 'parse_coeffs_str':parse_coeffs_str}
                result = {'success':True,'value':helpers[payload['method']](*payload['args'])}
            else:
                result = dispatch_native(payload)
        except Exception as error:
            result = {'success':False,'error':f'{type(error).__name__}: {error}'}
        record = {**case, 'output':extract_assets(_json_safe(result),output)}
        if result.get('success'):
            arrays = numerical(payload)
            if arrays is not None:
                record['numerical'] = _json_safe(arrays)
        data = encoded(record)
        path = f"cases/{case['id']}.json"
        (output/path).write_bytes(data)
        manifest['cases'].append({'id':case['id'],'module':case['module'],'action':payload['action'],'success':result.get('success',False),'path':path,'sha256':digest(data)})
        plt.close('all')
        print(f"{case['id']}: {'ok' if result.get('success') else result.get('error')}",flush=True)
    (output/'manifest.json').write_bytes(encoded(manifest))
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'tests/reference/python')
    parser.add_argument('--check',action='store_true',help='Regenerate in a temporary directory and compare; never overwrite the oracle.')
    parser.add_argument('--case',action='append',help='Select a case ID (repeatable). Best used with --check.')
    args = parser.parse_args()
    if args.check:
        with tempfile.TemporaryDirectory(prefix='controlab-reference-') as temp:
            generated = Path(temp)
            manifest = export(generated,args.case)
            changed = []
            if not args.case:
                original = json.loads((args.output/'manifest.json').read_text())
                if {c['id'] for c in original['cases']} != {c['id'] for c in manifest['cases']}:
                    changed.append('case inventory')
            for entry in manifest['cases']:
                path = entry['path']
                if not (args.output/path).is_file() or (args.output/path).read_bytes() != (generated/path).read_bytes():
                    changed.append(path)
            if changed:
                raise SystemExit('Reference differences: ' + ', '.join(changed))
            print(f"Verified {len(manifest['cases'])} complete cases including asset hashes.")
    else:
        export(args.output,args.case)


if __name__ == '__main__':
    main()
