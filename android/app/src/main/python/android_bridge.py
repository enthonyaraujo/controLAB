"""Ponte do Capacitor/Chaquopy para o mesmo motor Python das interfaces desktop."""

import json
import os
import tempfile
from pathlib import Path

try:
    from java import jclass

    application = jclass("com.chaquo.python.Python").getPlatform().getApplication()
    cache_root = Path(str(application.getCacheDir().getAbsolutePath())) / "lgr-studio"
except ImportError:
    cache_root = Path(tempfile.gettempdir()) / "lgr-studio"

cache_root.mkdir(parents=True, exist_ok=True)
os.environ.setdefault("MPLCONFIGDIR", str(cache_root / "matplotlib"))

import matplotlib  # noqa: E402
matplotlib.use("Agg")
from matplotlib import font_manager  # noqa: E402


class _SafeWeightDict(dict):
    """Protege contra KeyError no backend_svg.py do matplotlib 3.6.0 no Android."""

    def __getitem__(self, key):
        try:
            return super().__getitem__(key)
        except KeyError:
            if isinstance(key, (int, float)):
                return int(key)
            try:
                return int(key)
            except (ValueError, TypeError):
                return 400

    def __missing__(self, key):
        return self.__getitem__(key)


if not isinstance(font_manager.weight_dict, _SafeWeightDict):
    font_manager.weight_dict = _SafeWeightDict(font_manager.weight_dict)
for _w in range(0, 1001, 50):
    font_manager.weight_dict[_w] = _w
    font_manager.weight_dict[str(_w)] = _w

from python_bridge import dispatch  # noqa: E402


def _json_safe(value):
    """Converte a resposta em JSON estrito.

    O org.json do Android lança JSONException para NaN/Infinity e o json do
    Python não serializa tipos NumPy; ambos derrubavam o cálculo no celular.
    """
    import math

    import numpy as np

    if value is None or isinstance(value, (bool, str)):
        return value
    if isinstance(value, (np.bool_,)):
        return bool(value)
    if isinstance(value, (int, np.integer)):
        return int(value)
    if isinstance(value, (float, np.floating)):
        number = float(value)
        if math.isnan(number):
            return None
        if math.isinf(number):
            return "∞" if number > 0 else "-∞"
        return number
    if isinstance(value, (complex, np.complexfloating)):
        return {"re": _json_safe(value.real), "im": _json_safe(value.imag)}
    if isinstance(value, np.ndarray):
        return [_json_safe(item) for item in value.tolist()]
    if isinstance(value, dict):
        return {str(key): _json_safe(item) for key, item in value.items()}
    if isinstance(value, (list, tuple, set)):
        return [_json_safe(item) for item in value]
    return str(value)


def dispatch_json(payload_json):
    """Recebe e devolve JSON para evitar conversões ambíguas entre Java e Python."""
    try:
        result = dispatch(json.loads(str(payload_json)))
    except Exception as exc:
        import traceback
        traceback.print_exc()
        result = {"success": False, "error": f"{type(exc).__name__}: {exc}"}
    try:
        return json.dumps(_json_safe(result), allow_nan=False)
    except Exception as exc:
        return json.dumps({"success": False, "error": f"Falha ao serializar resultado: {exc}"})
