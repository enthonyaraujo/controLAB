"""Análise temporal contínua: transitório e regime permanente."""

from __future__ import annotations

import math

import control as ct
import matplotlib.pyplot as plt
import numpy as np
import sympy as sp

from control_utils import (
    figure_payload,
    finite_float,
    json_safe_number,
    parse_siso_transfer_function,
    recommended_time_vector,
    response_arrays,
    serialize_complex,
)


TIME_RESPONSE_PRESETS = {
    "Segunda ordem subamortecida": "4 / (s^2 + 2*s + 4)",
    "Primeira ordem": "1 / (2*s + 1)",
    "Sistema tipo 1": "5 / (s * (s + 2))",
}


def _safe_constant(expr):
    value = sp.limit(expr, sp.Symbol("s"), 0)
    if value in (sp.oo, -sp.oo):
        return math.inf
    if value is sp.nan or value.has(sp.zoo):
        return None
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def _static_error_analysis(num, den):
    s = sp.Symbol("s")
    numerator = sp.Poly.from_list(list(num), gens=s).as_expr()
    denominator = sp.Poly.from_list(list(den), gens=s).as_expr()
    loop = sp.cancel(numerator / denominator)
    constants = [_safe_constant((s**order) * loop) for order in range(3)]

    numerator_zeros = max(0, len(num) - len(np.trim_zeros(num, "b")))
    denominator_zeros = max(0, len(den) - len(np.trim_zeros(den, "b")))
    system_type = max(0, denominator_zeros - numerator_zeros)

    kp, kv, ka = constants

    def reciprocal(value, add_one=False):
        if value is None:
            return None
        if math.isinf(value):
            return 0.0
        denominator_value = value + 1.0 if add_one else value
        if abs(denominator_value) < 1e-14:
            return math.inf
        return 1.0 / denominator_value

    return {
        "type": int(system_type),
        "kp": kp,
        "kv": kv,
        "ka": ka,
        "ess_step": reciprocal(kp, add_one=True),
        "ess_ramp": reciprocal(kv),
        "ess_parabolic": reciprocal(ka),
    }


def _crossing_time(time, signal, level):
    indices = np.flatnonzero(signal >= level)
    return float(time[indices[0]]) if indices.size else None


def _step_metrics(time, output, poles, threshold):
    final_value = finite_float(output[-1], 0.0)
    peak_index = int(np.argmax(output))
    peak_value = finite_float(output[peak_index], 0.0)
    peak_time = finite_float(time[peak_index])
    overshoot = None
    if final_value is not None and abs(final_value) > 1e-12:
        overshoot = max(0.0, (peak_value - final_value) / abs(final_value) * 100.0)

    low, high = sorted((0.1 * final_value, 0.9 * final_value))
    rise_start = _crossing_time(time, output, low)
    rise_end = _crossing_time(time, output, high)
    rise_time = None if rise_start is None or rise_end is None else max(0.0, rise_end - rise_start)

    band = threshold * max(abs(final_value), 1e-9)
    outside = np.flatnonzero(np.abs(output - final_value) > band)
    settling_time = None
    if outside.size == 0:
        settling_time = 0.0
    elif outside[-1] < len(time) - 1:
        settling_time = float(time[outside[-1] + 1])

    pole_array = np.asarray(poles, dtype=complex)
    stable_poles = pole_array[np.real(pole_array) < -1e-9]
    dominant = stable_poles[np.argmax(np.real(stable_poles))] if stable_poles.size else None
    natural_frequency = float(abs(dominant)) if dominant is not None else None
    damping_ratio = (
        float(-np.real(dominant) / abs(dominant))
        if dominant is not None and abs(dominant) > 1e-12
        else None
    )

    return {
        "final_value": final_value,
        "peak_value": peak_value,
        "overshoot_percent": overshoot,
        "rise_time": rise_time,
        "peak_time": peak_time,
        "settling_time": settling_time,
        "damping_ratio": damping_ratio,
        "natural_frequency": natural_frequency,
    }


def _display_value(value, digits=5):
    if value is None:
        return "Não definido"
    if isinstance(value, (float, np.floating)) and math.isinf(float(value)):
        return "∞"
    if isinstance(value, (float, np.floating)):
        return f"{float(value):.{digits}g}"
    return str(value)


def analyze_time_response(expression, *, final_time=None, points=900, settling_threshold=0.02):
    """Analisa a resposta de malha fechada e os erros da malha aberta."""
    threshold = float(settling_threshold)
    if threshold not in (0.02, 0.05):
        raise ValueError("O critério de acomodação deve ser 0,02 ou 0,05.")

    plant, num, den, latex_expanded, latex_factored = parse_siso_transfer_function(expression)
    closed_loop = ct.feedback(plant, 1)
    poles = ct.poles(closed_loop)
    stable = bool(np.all(np.real(poles) < -1e-9))
    time = recommended_time_vector(closed_loop, final_time, points)

    t_step, y_step = response_arrays(ct.step_response(closed_loop, timepts=time))
    t_impulse, y_impulse = response_arrays(ct.impulse_response(closed_loop, timepts=time))
    t_ramp, y_ramp = response_arrays(ct.forced_response(closed_loop, timepts=time, inputs=time))

    metrics = _step_metrics(t_step, y_step, poles, threshold)
    errors = _static_error_analysis(num, den)

    fig, axes = plt.subplots(3, 1, figsize=(9.2, 8.0), sharex=True)
    series = [
        (t_step, y_step, "Degrau unitário", "$y(t)$", "#2563eb"),
        (t_impulse, y_impulse, "Impulso", "$y_δ(t)$", "#7c3aed"),
        (t_ramp, y_ramp, "Rampa unitária", "$y_r(t)$", "#059669"),
    ]
    for axis, (t_values, y_values, title, ylabel, color) in zip(axes, series):
        axis.plot(t_values, y_values, color=color, linewidth=2.0)
        axis.set_title(title, loc="left", fontsize=10, fontweight="bold")
        axis.set_ylabel(ylabel)
        axis.grid(True, alpha=0.25)
    axes[-1].set_xlabel("Tempo (s)")
    fig.suptitle("Respostas temporais da malha fechada com realimentação unitária", fontsize=13, fontweight="bold")
    fig.tight_layout()
    image, svg = figure_payload(fig)

    metric_rows = [
        {"label": "Estabilidade", "value": "Estável" if stable else "Instável"},
        {"label": "Sobressinal (%OS)", "value": _display_value(metrics["overshoot_percent"]), "unit": "%"},
        {"label": f"Acomodação ({int(threshold * 100)}%)", "value": _display_value(metrics["settling_time"]), "unit": "s"},
        {"label": "Tempo de subida (tr)", "value": _display_value(metrics["rise_time"]), "unit": "s"},
        {"label": "Tempo de pico (tp)", "value": _display_value(metrics["peak_time"]), "unit": "s"},
        {"label": "Amortecimento (ζ)", "value": _display_value(metrics["damping_ratio"])},
        {"label": "Frequência natural (ωn)", "value": _display_value(metrics["natural_frequency"]), "unit": "rad/s"},
        {"label": "Tipo do sistema", "value": f"Tipo {errors['type']}"},
        {"label": "Kp", "value": _display_value(errors["kp"])},
        {"label": "Kv", "value": _display_value(errors["kv"])},
        {"label": "Ka", "value": _display_value(errors["ka"])},
        {"label": "ess degrau", "value": _display_value(errors["ess_step"])},
        {"label": "ess rampa", "value": _display_value(errors["ess_ramp"])},
        {"label": "ess parabólica", "value": _display_value(errors["ess_parabolic"])},
    ]

    return {
        "success": True,
        "module": "time_response",
        "title": "Resposta no Domínio do Tempo",
        "description": "Degrau, impulso e rampa da malha fechada com análise de regime permanente da malha aberta.",
        "image": image,
        "svg": svg,
        "latex": [
            {"label": "Planta", "value": latex_factored.replace("G(s) = ", "")},
            {"label": "Malha fechada", "value": r"T(s)=\frac{G(s)}{1+G(s)}"},
            {"label": "Constantes de erro", "value": r"K_p=\lim_{s\to0}G(s),\quad K_v=\lim_{s\to0}sG(s),\quad K_a=\lim_{s\to0}s^2G(s)"},
        ],
        "metrics": metric_rows,
        "details": {
            "stable": stable,
            "poles": serialize_complex(poles),
            "step": metrics,
            "steady_state": {key: json_safe_number(value) for key, value in errors.items()},
            "latex_expanded": latex_expanded,
        },
    }


def get_time_response_presets():
    return [
        {"id": name, "title": name, "expr": expression}
        for name, expression in TIME_RESPONSE_PRESETS.items()
    ]
