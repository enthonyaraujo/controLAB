"""Análise de resposta em frequência por Bode e Nyquist."""

from __future__ import annotations

import math

import control as ct
import matplotlib.pyplot as plt
import numpy as np

from control_utils import (
    figure_payload,
    finite_float,
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


def analyze_frequency_response(expression, *, omega_min=0.01, omega_max=100.0, points=900):
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

    fig = plt.figure(figsize=(11.2, 7.8))
    grid = fig.add_gridspec(2, 2, width_ratios=(1.2, 1.0), hspace=0.28, wspace=0.28)
    ax_mag = fig.add_subplot(grid[0, 0])
    ax_phase = fig.add_subplot(grid[1, 0], sharex=ax_mag)
    ax_nyquist = fig.add_subplot(grid[:, 1])

    ax_mag.semilogx(omega, magnitude_db, color="#2563eb", linewidth=2, label="Curva real")
    ax_mag.semilogx(omega, asymptote_db, color="#f59e0b", linewidth=1.6, linestyle="--", label="Assíntotas")
    ax_mag.set_ylabel("Magnitude (dB)")
    ax_mag.set_title("Diagrama de Bode", loc="left", fontweight="bold")
    ax_mag.grid(True, which="both", alpha=0.25)
    ax_mag.legend(fontsize=8)

    ax_phase.semilogx(omega, phase_deg, color="#7c3aed", linewidth=2)
    ax_phase.set_xlabel("Frequência (rad/s)")
    ax_phase.set_ylabel("Fase (graus)")
    ax_phase.grid(True, which="both", alpha=0.25)

    ax_nyquist.plot(np.real(complex_response), np.imag(complex_response), color="#059669", linewidth=2, label="ω ≥ 0")
    ax_nyquist.plot(np.real(complex_response), -np.imag(complex_response), color="#059669", linewidth=1.4, linestyle="--", label="ω < 0")
    ax_nyquist.scatter([-1], [0], marker="x", s=90, linewidths=2.2, color="#dc2626", label="Ponto crítico (-1, 0)")
    ax_nyquist.axhline(0, color="#64748b", linewidth=0.8)
    ax_nyquist.axvline(0, color="#64748b", linewidth=0.8)
    ax_nyquist.set_xlabel("Parte real")
    ax_nyquist.set_ylabel("Parte imaginária")
    ax_nyquist.set_title("Diagrama de Nyquist", loc="left", fontweight="bold")
    ax_nyquist.grid(True, alpha=0.25)
    ax_nyquist.legend(fontsize=8)
    ax_nyquist.set_aspect("equal", adjustable="datalim")
    fig.suptitle("Resposta em frequência da malha aberta", fontsize=13, fontweight="bold")
    image, svg = figure_payload(fig)

    metrics = [
        {"label": "Margem de ganho (MG)", "value": _display(gain_margin_db), "unit": "dB"},
        {"label": "Cruzamento de fase (ωcf)", "value": _display(phase_cross), "unit": "rad/s"},
        {"label": "Margem de fase (MF)", "value": _display(phase_margin), "unit": "°"},
        {"label": "Cruzamento de ganho (ωcg)", "value": _display(gain_cross), "unit": "rad/s"},
        {"label": "Largura de banda (ωBW)", "value": _display(bandwidth), "unit": "rad/s"},
        {"label": "Pico de ressonância (Mr)", "value": _display(resonance_peak)},
        {"label": "Frequência de ressonância", "value": _display(resonance_frequency), "unit": "rad/s"},
        {"label": "Nyquist (Z = N + P)", "value": f"{z_count} = {n_count} + {p_count}"},
    ]

    details = {
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
