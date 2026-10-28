"""Native presentation adapters; no scientific computations or browser renderer."""
import base64
import io
import re
from pathlib import Path

MAX_FORMULAS = 200
MAX_FORMULA_CHARS = 4096
# MathText lacks these AMS arrows; the substitutes keep the same meaning and spacing.
_MATHTEXT_SUBSTITUTES = (
    (r'\displaystyle', ''),
    (r'\dfrac', r'\frac'),
    (r'\tfrac', r'\frac'),
    (r'\implies', r'\;\Longrightarrow\;'),
    (r'\iff', r'\;\Longleftrightarrow\;'),
)


# Formulas are typeset at a 48 px em in white; the client scales and tints them.
FORMULA_EM_PX = 48
_KATEX_FONTS = Path(__file__).resolve().parents[1] / 'renderer' / 'vendor' / 'katex' / 'fonts'
_fontset = None


def _katex_fontset():
    """MathText settings using the KaTeX fonts the previous renderer displayed."""
    global _fontset
    if _fontset is None:
        from matplotlib import font_manager
        files = sorted(_KATEX_FONTS.glob('KaTeX_*.ttf')) if _KATEX_FONTS.is_dir() else []
        for file in files:
            font_manager.fontManager.addfont(str(file))
        _fontset = {'mathtext.fontset': 'cm'}
        if files:
            _fontset = {
                'mathtext.fontset': 'custom',
                'mathtext.rm': 'KaTeX_Main',
                'mathtext.it': 'KaTeX_Math:italic',
                'mathtext.bf': 'KaTeX_Main:bold',
                'mathtext.sf': 'KaTeX_SansSerif',
                'mathtext.cal': 'KaTeX_Caligraphic',
                'mathtext.tt': 'DejaVu Sans Mono',
                'mathtext.fallback': 'cm',
            }
    return _fontset


_MATRIX = re.compile(r'\\begin\{([pbv]?matrix)\}(.*?)\\end\{\1\}', re.S)


def _stack(cells):
    """Vertical stack of cells with \\genfrac, MathText's only multi-row construct."""
    if len(cells) == 1:
        return cells[0]
    head, rest = cells[0], _stack(cells[1:])
    return r'\genfrac{}{}{0}{0}{%s}{%s}' % (head, rest)


def _matrices(expression):
    """Rewrite matrix environments, which MathText lacks, as columns of stacked cells."""
    def replace(match):
        kind, body = match.group(1), match.group(2)
        rows = [[cell.strip() for cell in row.split('&')] for row in re.split(r'\\\\', body) if row.strip()]
        width = max(len(row) for row in rows)
        columns = [_stack([row[j] if j < len(row) else '' for row in rows]) for j in range(width)]
        inner = r'\quad '.join('{%s}' % column for column in columns)
        fences = {'pmatrix': ('(', ')'), 'bmatrix': ('[', ']'), 'vmatrix': ('|', '|')}.get(kind)
        return r'\left%s %s \right%s' % (fences[0], inner, fences[1]) if fences else inner
    return _MATRIX.sub(replace, expression)


_INNER_BEFORE = re.compile(r'(?<=[A-Za-z0-9})])\s*\\left(?=[(\[|])')
_INNER_AFTER = re.compile(r'(\\right[)\]|])\s*(?=[A-Za-z0-9]|\\left|\\[a-zA-Z])')


def _inner_spacing(expression):
    """KaTeX spaces `\\left…\\right` groups from adjacent symbols; MathText does not."""
    expression = _INNER_BEFORE.sub(r'\\,\\left', expression)
    return _INNER_AFTER.sub(lambda m: m.group(1) + r'\,', expression)


def _display_style(expression):
    """KaTeX display mode: top-level fractions are full size, nested ones stay compact."""
    out, depth, i = [], 0, 0
    while i < len(expression):
        if expression.startswith('\\frac', i) and depth == 0:
            out.append('\\dfrac')
            i += 5
            continue
        char = expression[i]
        if char == '\\' and i + 1 < len(expression):
            out.append(expression[i:i + 2])
            i += 2
            continue
        depth += char == '{'
        depth -= char == '}'
        out.append(char)
        i += 1
    return ''.join(out)


def render_formulas(sources, theme='dark'):
    """Typeset each TeX source with Matplotlib MathText, in request order.

    Unsupported or oversized sources yield None so the client keeps the TeX text.
    The renderer uses MathText's internal parser, never a shell or external LaTeX.
    `theme` is accepted for compatibility; images are white and tinted by the client.
    Sources prefixed with `\\textstyle` are inline math; others use KaTeX display mode.
    """
    if not isinstance(sources, list):
        raise ValueError('sources deve ser uma lista de fórmulas TeX.')
    import warnings
    import matplotlib
    from matplotlib.figure import Figure
    from matplotlib.font_manager import FontProperties
    from matplotlib.mathtext import MathTextParser
    import logging
    images, depths = [], []
    # Sized delimiters fall back to Computer Modern; that is expected, not worth a log line.
    logging.getLogger('matplotlib.mathtext').setLevel(logging.ERROR)
    with matplotlib.rc_context(_katex_fontset()), warnings.catch_warnings():
        warnings.simplefilter('ignore')
        # FontProperties captures mathtext.fontset when created, so build it in the context.
        prop = FontProperties(size=FORMULA_EM_PX / 2)
        for source in sources[:MAX_FORMULAS]:
            if not isinstance(source, str) or not source.strip() or len(source) > MAX_FORMULA_CHARS:
                images.append(None)
                depths.append(0)
                continue
            expression = source.strip().strip('$')
            inline = expression.startswith('\\textstyle')
            if inline:
                expression = expression[len('\\textstyle'):]
            for command, substitute in _MATHTEXT_SUBSTITUTES:
                expression = expression.replace(command, substitute)
            expression = _inner_spacing(_matrices(expression))
            if not inline:
                expression = _display_style(expression)
            try:
                # math_to_image with a transparent figure; depth locates the baseline.
                text = '$' + expression + '$'
                width, height, depth, _, _ = MathTextParser('path').parse(text, dpi=72, prop=prop)
                if width <= 0 or height <= 0:
                    raise ValueError('fórmula vazia')
                figure = Figure(figsize=(width / 72, height / 72))
                figure.text(0, depth / height, text, fontproperties=prop, color='#FFFFFF')
                output = io.BytesIO()
                figure.savefig(output, dpi=144, format='png', transparent=True)
                images.append(base64.b64encode(output.getvalue()).decode('ascii'))
                depths.append(depth / height)
            except (ValueError, RuntimeError):
                images.append(None)
                depths.append(0)
    return {'success': True, 'images': images, 'depths': depths, 'em': FORMULA_EM_PX}


GITHUB_RELEASES_API = 'https://api.github.com/repos/enthonyaraujo/controlab/releases/latest'


def check_updates():
    """Latest GitHub release, as the previous settings dialog queried it."""
    import json
    import urllib.error
    import urllib.request
    request = urllib.request.Request(GITHUB_RELEASES_API, headers={
        'Accept': 'application/vnd.github.v3+json', 'User-Agent': 'ControLAB'})
    try:
        with urllib.request.urlopen(request, timeout=15) as response:
            release = json.loads(response.read(1024 * 1024).decode('utf-8'))
    except urllib.error.HTTPError as error:
        if error.code == 403:
            message = ('Limite de requisições da API do GitHub atingido temporariamente. '
                       'Tente novamente mais tarde.')
        else:
            message = f'O GitHub retornou HTTP {error.code}'
        return {'success': False, 'error': message}
    except (OSError, ValueError):
        return {'success': False, 'error': 'Verifique sua conexão à internet.'}
    return {'success': True, 'tag_name': release.get('tag_name', ''),
            'html_url': release.get('html_url', ''), 'published_at': release.get('published_at', '')}


def dispatch_native(data, dispatch=None):
    """Route presentation requests locally; everything else goes to the scientific engine."""
    if data.get('action') == 'render_formulas':
        return render_formulas(data.get('sources'), data.get('theme', 'dark'))
    if data.get('action') == 'check_updates':
        return check_updates()
    if dispatch is None:
        from python_bridge import dispatch
    return dispatch(data)


def dispatch_android(payload_json):
    """Retain Chaquopy's Android-specific Matplotlib compatibility setup."""
    import json
    from android_bridge import dispatch_json
    data = json.loads(payload_json)
    if data.get('action') == 'render_formulas':
        return json.dumps(render_formulas(data.get('sources'), data.get('theme', 'dark')),
                          ensure_ascii=False, allow_nan=False)
    if data.get('action') == 'check_updates':
        return json.dumps(check_updates(), ensure_ascii=False)
    return dispatch_json(payload_json)
