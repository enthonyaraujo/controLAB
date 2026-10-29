"""Persistent, bounded JSON-lines transport for the native ControLAB client.

Only protocol replies go to stdout. Scientific diagnostics, import-time output
and runtime shutdown messages go to stderr, including during interpreter exit.
"""

import json
import math
import sys

MAX_REQUEST_BYTES = 8 * 1024 * 1024
MAX_RESPONSE_BYTES = 32 * 1024 * 1024


def _json_safe(value):
    """Normalize scientific values without loading the Android compatibility shim."""
    import numpy as np

    if value is None or isinstance(value, (str, bool)):
        return value
    if isinstance(value, np.bool_):
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
        return _json_safe(value.tolist())
    if isinstance(value, dict):
        return {str(key): _json_safe(item) for key, item in value.items()}
    if isinstance(value, (list, tuple, set)):
        return [_json_safe(item) for item in value]
    return str(value)


def _reject_constant(value):
    raise ValueError(f"Constante JSON inválida: {value}")


def main():
    # The protocol is UTF-8 bytes whatever the console code page: frozen engines
    # ignore PYTHONIOENCODING, and Windows would otherwise encode as cp1252.
    protocol_stdout = sys.stdout.buffer
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="backslashreplace")
    sys.stdout = sys.stderr
    from python_bridge import dispatch

    source = sys.stdin.buffer
    while True:
        raw = source.readline(MAX_REQUEST_BYTES + 1)
        if not raw:
            break
        try:
            if len(raw) > MAX_REQUEST_BYTES:
                # Drain the rejected line so the next complete request can recover.
                while raw and not raw.endswith(b"\n"):
                    raw = source.readline(MAX_REQUEST_BYTES + 1)
                raise ValueError("Requisição excede o limite de 8 MiB.")
            data = json.loads(raw, parse_constant=_reject_constant)
            if not isinstance(data, dict):
                raise ValueError("A requisição deve ser um objeto JSON.")
            from native_presentation import dispatch_native
            result = dispatch_native(data, dispatch)
            response = json.dumps(_json_safe(result), ensure_ascii=False, allow_nan=False)
            if len(response.encode("utf-8")) > MAX_RESPONSE_BYTES:
                raise ValueError("Resposta excede o limite de 32 MiB.")
        except Exception as error:
            response = json.dumps(
                {"success": False, "error": f"{type(error).__name__}: {error}"},
                ensure_ascii=False, allow_nan=False,
            )
        protocol_stdout.write(response.encode("utf-8") + b"\n")
        protocol_stdout.flush()


if __name__ == "__main__":
    main()
