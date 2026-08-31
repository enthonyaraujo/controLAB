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

from python_bridge import dispatch  # noqa: E402


def dispatch_json(payload_json):
    """Recebe e devolve JSON para evitar conversões ambíguas entre Java e Python."""
    try:
        result = dispatch(json.loads(str(payload_json)))
    except Exception as exc:
        result = {"success": False, "error": str(exc)}
    return json.dumps(result)
