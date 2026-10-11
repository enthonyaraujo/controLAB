"""Análise de resposta em frequência por Bode e Nyquist."""

from __future__ import annotations

import math

import control as ct
import matplotlib.pyplot as plt
import numpy as np

from control_utils import (
    figure_payload,
    finite_float,
    get_theme_tokens,
    json_safe_number,
    parse_siso_transfer_function,
)


FREQUENCY_PRESETS = {
    "Primeira ordem": "1 / (s + 1)",
    "Segunda ordem": "25 / (s^2 + 4*s + 25)",
    "Tipo 1 com dois polos": "10 / (s * (s + 2) * (s + 5))",
}


def _display(value, digits=5):
    if value is None or not np.isfinite(value):
        return "∞" if value is not None and np.isinf(value) else "Não definido"
    return f"{float(value):.{digits}g}"


def _asymptotic_magnitude(num, den, omega, actual_db):
    """Constrói as assíntotas de Bode por variação de inclinação nos polos/zeros."""
    num = np.trim_zeros(np.asarray(num, dtype=float), "f")
    den = np.trim_zeros(np.asarray(den, dtype=float), "f")
    zeros = np.roots(num) if len(num) > 1 else np.array([], dtype=complex)
    poles = np.roots(den) if len(den) > 1 else np.array([], dtype=complex)
    tolerance = 1e-10
    slope = 20.0 * (
        np.count_nonzero(np.abs(zeros) <= tolerance)
        - np.count_nonzero(np.abs(poles) <= tolerance)
    )
    events = []
    events.extend((abs(root), 20.0) for root in zeros if abs(root) > tolerance)
    events.extend((abs(root), -20.0) for root in poles if abs(root) > tolerance)
    events.sort(key=lambda item: item[0])

    result = np.empty_like(omega, dtype=float)
    result[0] = actual_db[0]
    event_index = 0
    for index in range(1, len(omega)):
        midpoint = math.sqrt(omega[index - 1] * omega[index])
        while event_index < len(events) and events[event_index][0] <= midpoint:
            slope += events[event_index][1]
            event_index += 1
        result[index] = result[index - 1] + slope * math.log10(omega[index] / omega[index - 1])
    return result


def _safe_bandwidth(system):
    try:
        value = float(ct.bandwidth(system))
        return value if np.isfinite(value) else math.inf
    except (ValueError, TypeError, RuntimeError):
        return None


def analyze_frequency_response(expression, *, omega_min=0.01, omega_max=100.0, points=900, theme="dark"):
    omega_min = float(omega_min)
    omega_max = float(omega_max)
    points = int(np.clip(int(points or 900), 200, 4000))
    if omega_min <= 0 or omega_max <= omega_min:
        raise ValueError("Use frequências positivas com ω máximo maior que ω mínimo.")
    if omega_max / omega_min > 1e12:
        raise ValueError("A faixa de frequências não pode exceder 12 décadas.")

    plant, num, den, latex_expanded, latex_factored = parse_siso_transfer_function(expression)
    omega = np.logspace(math.log10(omega_min), math.log10(omega_max), points)
    response = ct.frequency_response(plant, omega)
    magnitude = np.asarray(response.magnitude, dtype=float).squeeze()
    phase_deg = np.unwrap(np.asarray(response.phase, dtype=float).squeeze()) * 180.0 / np.pi
    magnitude_db = 20.0 * np.log10(np.maximum(magnitude, np.finfo(float).tiny))
    asymptote_db = _asymptotic_magnitude(num, den, omega, magnitude_db)
    complex_response = magnitude * np.exp(1j * np.deg2rad(phase_deg))

    gain_margin, phase_margin, stability_margin, phase_cross, gain_cross, stability_cross = ct.stability_margins(plant)
    gain_margin = float(gain_margin)
    phase_margin = float(phase_margin)
    phase_cross = float(phase_cross)
    gain_cross = float(gain_cross)
    gain_margin_db = math.inf if np.isinf(gain_margin) else 20.0 * math.log10(max(gain_margin, np.finfo(float).tiny))

    closed_loop = ct.feedback(plant, 1)
    closed_response = ct.frequency_response(closed_loop, omega)
    closed_magnitude = np.asarray(closed_response.magnitude, dtype=float).squeeze()
    resonance_index = int(np.argmax(closed_magnitude))
    resonance_peak = float(closed_magnitude[resonance_index])
    resonance_frequency = float(omega[resonance_index])
    bandwidth = _safe_bandwidth(closed_loop)

    open_poles = ct.poles(plant)
    closed_poles = ct.poles(closed_loop)
    p_count = int(np.count_nonzero(np.real(open_poles) > 1e-8))
    z_count = int(np.count_nonzero(np.real(closed_poles) > 1e-8))
    n_count = z_count - p_count
    closed_stable = bool(np.all(np.real(closed_poles) < -1e-9))
    has_origin_pole = bool(np.any(np.abs(open_poles) < 1e-8))

    tokens = get_theme_tokens(theme)
    accent = tokens["accent"]
    ref_color = tokens["reference"]
    danger_color = tokens["danger"]

    fig = plt.figure(figsize=(11.2, 7.2), dpi=140)
    grid = fig.add_gridspec(2, 2, width_ratios=(1.2, 1.0), hspace=0.25, wspace=0.25)
    ax_mag = fig.add_subplot(grid[0, 0])
    ax_phase = fig.add_subplot(grid[1, 0], sharex=ax_mag)
    ax_nyquist = fig.add_subplot(grid[:, 1])

    ax_mag.semilogx(omega, magnitude_db, color=accent, linewidth=2.0, label="Curva real")
    ax_mag.semilogx(omega, asymptote_db, color=ref_color, linewidth=1.5, linestyle="--", label="Assíntotas")
    ax_mag.set_ylabel("Magnitude (dB)")
    ax_mag.legend(loc="upper right")

    ax_phase.semilogx(omega, phase_deg, color=accent, linewidth=2.0)
    ax_phase.set_xlabel("Frequência (rad/s)")
    ax_phase.set_ylabel("Fase (graus)")

    ax_nyquist.plot(np.real(complex_response), np.imag(complex_response), color=accent, linewidth=2.0, label="ω ≥ 0")
    ax_nyquist.plot(np.real(complex_response), -np.imag(complex_response), color=accent, linewidth=1.4, linestyle="--", label="ω < 0", alpha=0.7)
    ax_nyquist.scatter([-1], [0], marker="x", s=80, linewidths=2.2, color=danger_color, label="Ponto crítico (-1, 0)", zorder=5)
    ax_nyquist.axhline(0, color=tokens["spine"], linewidth=0.8, alpha=0.7)
    ax_nyquist.axvline(0, color=tokens["spine"], linewidth=0.8, alpha=0.7)

    # Identifica assíntota vertical quando há polo na origem e limita eixos inteligentemente
    asymptote_re = None
    if has_origin_pole:
        asymptote_re = float(np.real(ct.evalfr(plant, 1e-6j)))
        if math.isfinite(asymptote_re):
            ax_nyquist.axvline(asymptote_re, color=ref_color, linestyle=":", linewidth=1.2, alpha=0.7, label=f"Assíntota Re ≈ {asymptote_re:.2f}")

    re_vals = np.real(complex_response)
    im_vals = np.imag(complex_response)
    ref_x = [-1.0, 0.0]
    if math.isfinite(gain_margin) and gain_margin > 0:
        ref_x.append(-1.0 / gain_margin)
    if asymptote_re is not None and math.isfinite(asymptote_re):
        ref_x.append(asymptote_re)

    finite_mask = (np.abs(complex_response) <= 8.0) & np.isfinite(re_vals) & np.isfinite(im_vals)
    if np.any(finite_mask):
        re_rel = re_vals[finite_mask]
        im_rel = im_vals[finite_mask]
        x_min = min(float(np.min(re_rel)), min(ref_x)) - 0.4
        x_max = max(float(np.max(re_rel)), max(ref_x)) + 0.4
        y_max = max(float(np.max(np.abs(im_rel))), 1.2) + 0.3
    else:
        x_min, x_max = -2.0, 1.0
        y_max = 1.5

    x_min = max(-50.0, min(-1.2, x_min))
    x_max = min(50.0, max(0.5, x_max))
    y_max = min(50.0, max(1.2, y_max))

    # Limita proporção máxima para manter visualização sem esmagar
    span_x = x_max - x_min
    span_y = 2.0 * y_max
    if span_x / span_y > 3.0:
        y_max = span_x / 3.0
    elif span_y / span_x > 3.0:
        span = span_y / 3.0
        mid = (x_max + x_min) / 2.0
        x_min = mid - span / 2.0
        x_max = mid + span / 2.0

    ax_nyquist.set_xlim(x_min, x_max)
    ax_nyquist.set_ylim(-y_max, y_max)
    ax_nyquist.set_xlabel("Parte real")
    ax_nyquist.set_ylabel("Parte imaginária")
    ax_nyquist.legend(loc="upper right")
    fig.subplots_adjust(left=0.08, right=0.96, top=0.96, bottom=0.09, hspace=0.28, wspace=0.25)
    image, svg = figure_payload(fig, theme=theme)

    # Separação nos grupos: Margens e Estimativas no tempo
    metrics = [
        {"header": "Margens"},
        {"label": "Estabilidade (malha fechada)", "value": "Estável" if closed_stable else "Instável"},
        {"label": "Margem de ganho (MG)", "value": _display(gain_margin_db), "unit": "dB"},
        {"label": "Cruzamento de fase (ωcf)", "value": _display(phase_cross), "unit": "rad/s"},
        {"label": "Margem de fase (MF)", "value": _display(phase_margin), "unit": "°"},
        {"label": "Cruzamento de ganho (ωcg)", "value": _display(gain_cross), "unit": "rad/s"},
        {"label": "Critério de Nyquist (Z = N + P)", "value": f"{z_count} = {n_count} + {p_count}"},
    ]
    if has_origin_pole:
        metrics.append({"label": "Observação", "value": "Malha aberta com polo na origem"})

    # Estimativas no domínio do tempo (ocultas se não se aplicarem)
    if closed_stable and 0 < phase_margin < 90:
        zeta_est = phase_margin / 100.0
        overshoot_est = math.exp(-math.pi * zeta_est / math.sqrt(1.0 - zeta_est**2)) * 100.0
        metrics.append({"header": "Estimativas no tempo"})
        metrics.append({"label": "Amortecimento estimado (ζ)", "value": _display(zeta_est)})
        metrics.append({"label": "Sobressinal estimado (%OS)", "value": _display(overshoot_est), "unit": "%"})
        if bandwidth is not None and math.isfinite(bandwidth) and bandwidth > 0:
            metrics.append({"label": "Largura de banda (ωBW)", "value": _display(bandwidth), "unit": "rad/s"})
            metrics.append({"label": "Tempo de subida estimado (tr)", "value": _display(1.8 / bandwidth), "unit": "s"})
            metrics.append({"label": "Tempo de acomodação (ts, 2%)", "value": _display(4.0 / (zeta_est * bandwidth)), "unit": "s"})
        if resonance_peak is not None and math.isfinite(resonance_peak):
            metrics.append({"label": "Pico de ressonância (Mr)", "value": _display(resonance_peak)})
            metrics.append({"label": "Frequência de ressonância", "value": _display(resonance_frequency), "unit": "rad/s"})

    details = {
        "stable": closed_stable,
        "has_origin_pole": has_origin_pole,
        "asymptote_re": json_safe_number(asymptote_re),
        "gain_margin": json_safe_number(gain_margin),
        "gain_margin_db": json_safe_number(gain_margin_db),
        "phase_margin_deg": json_safe_number(phase_margin),
        "phase_crossover_frequency": json_safe_number(phase_cross),
        "gain_crossover_frequency": json_safe_number(gain_cross),
        "stability_margin": json_safe_number(float(stability_margin)),
        "stability_margin_frequency": json_safe_number(float(stability_cross)),
        "bandwidth": json_safe_number(bandwidth),
        "resonance_peak": resonance_peak,
        "resonance_peak_db": 20.0 * math.log10(max(resonance_peak, np.finfo(float).tiny)),
        "resonance_frequency": resonance_frequency,
        "nyquist": {"Z": z_count, "N": n_count, "P": p_count},
        "latex_expanded": latex_expanded,
    }

    return {
        "success": True,
        "module": "frequency_response",
        "title": "Resposta em Frequência",
        "description": "Bode real e assintótico, Nyquist e métricas de estabilidade relativa.",
        "image": image,
        "svg": svg,
        "latex": [
            {"label": "Planta", "value": latex_factored.replace("G(s) = ", "")},
            {"label": "Resposta senoidal", "value": r"G(j\omega)=G(s)\vert_{s=j\omega}"},
            {"label": "Critério de Nyquist", "value": rf"Z=N+P\;\Rightarrow\;{z_count}={n_count}+{p_count}"},
        ],
        "metrics": metrics,
        "details": details,
    }


def get_frequency_presets():
    return [
        {"id": name, "title": name, "expr": expression}
        for name, expression in FREQUENCY_PRESETS.items()
    ]
