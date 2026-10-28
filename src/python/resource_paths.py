"""Resolve canonical shared data in source checkouts, desktop bundles and Android.

No renderer or Node modules are consulted. Fonts must be ordinary filesystem
files for Matplotlib, so Android extracts the generated controlab_assets package.
"""
import importlib
import os
from pathlib import Path
import sys


def assets_root():
    override = os.environ.get('CONTROLAB_ASSETS')
    if override:
        root = Path(override).expanduser().resolve()
        if not (root / 'manifest.json').is_file():
            raise FileNotFoundError(f'Recursos do ControLAB ausentes em {root}')
        return root
    frozen_root = getattr(sys, '_MEIPASS', None)
    if frozen_root:
        root = Path(frozen_root) / 'assets'
        if (root / 'manifest.json').is_file():
            return root
        raise FileNotFoundError('Pacote nativo sem os recursos compartilhados assets/')
    try:
        package = importlib.import_module('controlab_assets')
    except ModuleNotFoundError as error:
        if error.name != 'controlab_assets':
            raise
    else:
        root = Path(package.__file__).resolve().parent
        if (root / 'manifest.json').is_file():
            return root
        raise FileNotFoundError('Pacote Android controlab_assets incompleto')
    root = Path(__file__).resolve().parents[2] / 'assets'
    if not (root / 'manifest.json').is_file():
        raise FileNotFoundError('Recursos do ControLAB ausentes: configure CONTROLAB_ASSETS')
    return root


def asset_path(*parts):
    root = assets_root().resolve()
    result = root.joinpath(*parts).resolve()
    if not result.is_relative_to(root):
        raise ValueError('O recurso deve permanecer dentro de assets/')
    return result
