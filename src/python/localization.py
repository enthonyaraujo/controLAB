"""Request-scoped localization; numerical data, paths and user code stay intact."""
import json
import re
from contextvars import ContextVar
from functools import wraps

_language = ContextVar('controlab_language', default='pt-BR')
from resource_paths import asset_path

_catalog_path = asset_path('locales', 'en.json')
_catalog = json.loads(_catalog_path.read_text(encoding='utf-8'))
_keys = sorted((k for k, v in _catalog.items() if k != v), key=len, reverse=True)
_pattern = re.compile(r'(?<![\w])(?:' + '|'.join(re.escape(k) for k in _keys) + r')(?![\w])')


def translate(text):
    if _language.get() != 'en' or not isinstance(text, str):
        return text
    return _catalog.get(text) or _pattern.sub(lambda m: _catalog[m.group()], text)


# Structural identifiers and user-owned strings must never be localized.
_PROTECTED = {'source', 'code', 'python_code', 'command', 'output', 'stdout', 'stderr',
              'path', 'folder', 'name', 'filename', 'value', 'expression', 'expr',
              'type', 'action', 'session', 'operation', 'image', 'plot_image', 'svg',
              'plot_svg', 'base64', 'variables', 'workspace', 'files', 'recent'}


def translate_result(value):
    if isinstance(value, str):
        return translate(value)
    if isinstance(value, list):
        return [translate_result(item) for item in value]
    if isinstance(value, dict):
        return {key: item if key in _PROTECTED else translate_result(item)
                for key, item in value.items()}
    return value


def localized_request(function):
    @wraps(function)
    def wrapped(data):
        token = _language.set('en' if data.get('language') == 'en' else 'pt-BR')
        try:
            result = function(data)
            # Script output and script figures belong to the author.
            return result if data.get('action') == 'script_request' else translate_result(result)
        except Exception as error:
            error.args = (translate(str(error)),)
            raise
        finally:
            _language.reset(token)
    return wrapped


def localize_figure(figure):
    from matplotlib.text import Text
    for text in figure.findobj(Text):
        text.set_text(translate(text.get_text()))
