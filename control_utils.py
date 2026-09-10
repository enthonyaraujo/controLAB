"""Utilitários compartilhados pelos módulos científicos do ControLAB."""

from __future__ import annotations

import base64
import io
import math
from typing import Iterable

import control as ct
import matplotlib.pyplot as plt
import numpy as np

from lgr_engine import parse_tf_expression


def parse_siso_transfer_function(expression: str):
    """Converte uma expressão segura em uma função de transferência SISO."""
    num, den, latex_expanded, latex_factored = parse_tf_expression(expression)
    system = ct.tf(num, den)
    return system, np.asarray(num, dtype=float), np.asarray(den, dtype=float), latex_expanded, latex_factored


def figure_payload(fig, *, dpi: int = 150):
    """Serializa uma figura Matplotlib em PNG base64 e SVG."""
    png_buffer = io.BytesIO()
    fig.savefig(png_buffer, format="png", dpi=dpi, bbox_inches="tight", facecolor="white")
    png_buffer.seek(0)
    image = "data:image/png;base64," + base64.b64encode(png_buffer.read()).decode("utf-8")

    svg_buffer = io.BytesIO()
    fig.savefig(svg_buffer, format="svg", bbox_inches="tight", facecolor="white")
    svg_buffer.seek(0)
    svg = svg_buffer.read().decode("utf-8")
    plt.close(fig)
    return image, svg


def finite_float(value, default=None):
    """Converte valores NumPy/complexos quase reais em JSON seguro."""
    if value is None:
        return default
    try:
        candidate = float(np.real_if_close(value))
    except (TypeError, ValueError):
        return default
    return candidate if math.isfinite(candidate) else default


def json_safe_number(value):
    """Mantém infinitos legíveis sem produzir JSON não padronizado."""
    if isinstance(value, (float, np.floating)) and math.isinf(float(value)):
        return "∞" if float(value) > 0 else "-∞"
    if isinstance(value, (float, np.floating)) and math.isnan(float(value)):
        return None
    return value


def serialize_complex(values: Iterable[complex]):
    return [
        {
            "re": float(np.real(value)),
            "im": float(np.imag(value)),
            "text": f"{np.real(value):.5g}{np.imag(value):+.5g}j",
        }
        for value in values
    ]


def recommended_time_vector(system, final_time=None, points=900):
    """Gera uma janela temporal estável mesmo para plantas lentas ou instáveis."""
    points = int(np.clip(int(points or 900), 200, 4000))
    if final_time is not None:
        final_time = float(final_time)
        if not 0.05 <= final_time <= 10000:
            raise ValueError("O tempo final deve estar entre 0,05 s e 10000 s.")
    else:
        poles = np.asarray(ct.poles(system), dtype=complex)
        rates = np.abs(np.real(poles[np.abs(np.real(poles)) > 1e-8]))
        final_time = 10.0 if rates.size == 0 else float(np.clip(8.0 / np.min(rates), 1.0, 120.0))
    return np.linspace(0.0, final_time, points)


def response_arrays(response):
    """Normaliza o retorno das APIs de resposta do python-control."""
    time = np.asarray(response.time, dtype=float).reshape(-1)
    output = np.asarray(response.outputs, dtype=float).squeeze()
    if output.ndim != 1:
        output = output.reshape(-1)
    return time, output
