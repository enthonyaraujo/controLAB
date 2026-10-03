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
    get_theme_tokens,
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


def _plot_step(time, output, metrics, tokens, theme):
    fig, ax = plt.subplots(figsize=(9.2, 5.2), dpi=140)
    accent = tokens["accent"]
    ref_color = tokens["reference"]
    mono_font = tokens["font_mono"]

    ax.plot(time, output, color=accent, linewidth=2.2, label="Resposta ao degrau y(t)")

    final_val = metrics.get("final_value")
    if final_val is not None and math.isfinite(final_val):
        ax.axhline(final_val, color=ref_color, linestyle="--", linewidth=1.5, alpha=0.85)
        ax.annotate(
            f"{final_val:.2f}".replace(".", ","),
            xy=(time[-1], final_val),
            xytext=(6, 0),
            textcoords="offset points",
            va="center",
            ha="left",
            color=tokens["text_secondary"],
            fontsize=9,
            fontfamily=mono_font,
        )

    peak_val = metrics.get("peak_value")
    peak_time = metrics.get("peak_time")
    overshoot = metrics.get("overshoot_percent")
    if peak_val is not None and peak_time is not None and math.isfinite(peak_val) and math.isfinite(peak_time):
        ax.plot([peak_time, peak_time], [0, peak_val], linestyle=":", color=tokens.get("accent_secondary", accent), linewidth=1.1, alpha=0.7)
        ax.plot(peak_time, peak_val, marker="o", markersize=6.5, color=accent, markeredgecolor=tokens["text"], markeredgewidth=1.2, zorder=5)
        mp_label = f"Mp {overshoot:.1f}%".replace(".", ",") if overshoot is not None and overshoot > 0 else f"{peak_val:.2f}".replace(".", ",")
        ax.annotate(
            mp_label,
            xy=(peak_time, peak_val),
            xytext=(10, 8),
            textcoords="offset points",
            color=tokens["text"],
            fontsize=9,
            fontweight="bold",
            fontfamily=mono_font,
        )

    ax.set_xlabel("Tempo (s)")
    ax.set_ylabel("y(t)")
    fig.tight_layout()
    return figure_payload(fig, theme=theme)


def _plot_ramp(time, output, tokens, theme):
    fig, ax = plt.subplots(figsize=(9.2, 5.2), dpi=140)
    ax.plot(time, time, color=tokens["reference"], linestyle="--", linewidth=1.5, label="Entrada r(t) = t", alpha=0.85)
    ax.plot(time, output, color=tokens["accent"], linewidth=2.2, label="Resposta à rampa y(t)")
    ax.set_xlabel("Tempo (s)")
    ax.set_ylabel("y(t)")
    ax.legend(loc="upper left")
    fig.tight_layout()
    return figure_payload(fig, theme=theme)


def _plot_impulse(time, output, tokens, theme):
    fig, ax = plt.subplots(figsize=(9.2, 5.2), dpi=140)
    ax.plot(time, output, color=tokens["accent"], linewidth=2.2, label="Resposta ao impulso y_δ(t)")
    ax.axhline(0, color=tokens["spine"], linewidth=0.8, alpha=0.7)
    ax.set_xlabel("Tempo (s)")
    ax.set_ylabel("y_δ(t)")
    fig.tight_layout()
    return figure_payload(fig, theme=theme)


def analyze_time_response(expression, *, final_time=None, points=900, settling_threshold=0.02, theme="dark", response_type="step"):
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

    tokens = get_theme_tokens(theme)
    img_step, svg_step = _plot_step(t_step, y_step, metrics, tokens, theme)
    img_ramp, svg_ramp = _plot_ramp(t_ramp, y_ramp, tokens, theme)
    img_impulse, svg_impulse = _plot_impulse(t_impulse, y_impulse, tokens, theme)

    plots = {
        "step": {"image": img_step, "svg": svg_step},
        "ramp": {"image": img_ramp, "svg": svg_ramp},
        "impulse": {"image": img_impulse, "svg": svg_impulse},
    }

    selected_plot = plots.get(response_type, plots["step"])
    image, svg = selected_plot["image"], selected_plot["svg"]

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
        "plots": plots,
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
