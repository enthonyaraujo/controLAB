"""Sintonia de controladores PID e projeto de compensadores lead-lag."""

from __future__ import annotations

import math

import control as ct
import matplotlib.pyplot as plt
import numpy as np

from control_utils import (
    figure_payload,
    json_safe_number,
    parse_siso_transfer_function,
    recommended_time_vector,
    response_arrays,
    serialize_complex,
)
from lgr_engine import format_transfer_function


CONTROLLER_PRESETS = {
    "Planta de terceira ordem": "1 / (s * (s + 1) * (s + 5))",
    "Processo de primeira ordem": "1 / (4*s + 1)",
    "Processo de segunda ordem": "1 / (s^2 + 3*s + 2)",
}


def _positive(value, label):
    number = float(value)
    if not np.isfinite(number) or number <= 0:
        raise ValueError(f"{label} deve ser maior que zero.")
    return number


def _pid_parameters(method, controller_type, *, process_gain, delay, time_constant, critical_gain, critical_period, chr_response):
    method = str(method).lower()
    controller_type = str(controller_type).upper()
    if controller_type not in {"P", "PI", "PID"}:
        raise ValueError("O controlador deve ser P, PI ou PID.")

    ti = None
    td = 0.0
    if method == "zn_reaction":
        gain = _positive(process_gain, "O ganho do processo")
        lag = _positive(delay, "O atraso L")
        tau = _positive(time_constant, "A constante de tempo T")
        if controller_type == "P":
            kp = tau / (gain * lag)
        elif controller_type == "PI":
            kp, ti = 0.9 * tau / (gain * lag), 3.33 * lag
        else:
            kp, ti, td = 1.2 * tau / (gain * lag), 2.0 * lag, 0.5 * lag
        method_label = "Ziegler-Nichols — curva de reação"
    elif method == "zn_critical":
        ku = _positive(critical_gain, "O ganho crítico Kcr")
        pu = _positive(critical_period, "O período crítico Pcr")
        if controller_type == "P":
            kp = 0.5 * ku
        elif controller_type == "PI":
            kp, ti = 0.45 * ku, pu / 1.2
        else:
            kp, ti, td = 0.6 * ku, pu / 2.0, pu / 8.0
        method_label = "Ziegler-Nichols — oscilação crítica"
    elif method == "cohen_coon":
        gain = _positive(process_gain, "O ganho do processo")
        lag = _positive(delay, "O atraso L")
        tau = _positive(time_constant, "A constante de tempo T")
        ratio = lag / tau
        if controller_type == "P":
            kp = (tau / (gain * lag)) * (1.0 + ratio / 3.0)
        elif controller_type == "PI":
            kp = (tau / (gain * lag)) * (0.9 + ratio / 12.0)
            ti = lag * (30.0 + 3.0 * ratio) / (9.0 + 20.0 * ratio)
        else:
            kp = (tau / (gain * lag)) * (4.0 / 3.0 + ratio / 4.0)
            ti = lag * (32.0 + 6.0 * ratio) / (13.0 + 8.0 * ratio)
            td = lag * 4.0 / (11.0 + 2.0 * ratio)
        method_label = "Cohen-Coon"
    elif method == "chr":
        gain = _positive(process_gain, "O ganho do processo")
        lag = _positive(delay, "O atraso L")
        tau = _positive(time_constant, "A constante de tempo T")
        response = str(chr_response)
        if response == "20":
            table = {
                "P": (0.7, None, 0.0),
                "PI": (0.6, tau, 0.0),
                "PID": (0.95, 1.4 * tau, 0.47 * lag),
            }
        else:
            table = {
                "P": (0.3, None, 0.0),
                "PI": (0.35, 1.2 * tau, 0.0),
                "PID": (0.6, tau, 0.5 * lag),
            }
        factor, ti, td = table[controller_type]
        kp = factor * tau / (gain * lag)
        method_label = f"CHR — {response}% de sobressinal"
    else:
        raise ValueError("Método de sintonia desconhecido.")

    ki = 0.0 if ti is None else kp / ti
    kd = kp * td
    return {"kp": kp, "ki": ki, "kd": kd, "ti": ti, "td": td, "method_label": method_label}


def _pid_transfer(parameters, controller_type):
    kp, ki, kd = parameters["kp"], parameters["ki"], parameters["kd"]
    if controller_type == "P":
        return ct.tf([kp], [1])
    if controller_type == "PI":
        return ct.tf([kp, ki], [1, 0])
    return ct.tf([kd, kp, ki], [1, 0])


def _step_metrics(system, time):
    try:
        info = ct.step_info(system, timepts=time, SettlingTimeThreshold=0.02)
    except (ValueError, TypeError, RuntimeError, IndexError):
        return {}
    mapping = {
        "rise_time": "RiseTime",
        "settling_time": "SettlingTime",
        "overshoot_percent": "Overshoot",
        "peak": "Peak",
        "peak_time": "PeakTime",
        "steady_state_value": "SteadyStateValue",
    }
    return {name: json_safe_number(float(info[key])) for name, key in mapping.items() if key in info}


def _root_locus_roots(system, gains):
    num = np.asarray(ct.tfdata(system)[0][0][0], dtype=float)
    den = np.asarray(ct.tfdata(system)[1][0][0], dtype=float)
    order = max(len(num), len(den))
    num = np.pad(num, (order - len(num), 0))
    den = np.pad(den, (order - len(den), 0))
    roots = []
    for gain in gains:
        polynomial = np.trim_zeros(den + gain * num, "f")
        roots.append(np.sort_complex(np.roots(polynomial)))
    return np.asarray(roots, dtype=complex)


def _controller_latex(controller):
    num = np.asarray(ct.tfdata(controller)[0][0][0], dtype=float).tolist()
    den = np.asarray(ct.tfdata(controller)[1][0][0], dtype=float).tolist()
    _, _, latex_expanded, _ = format_transfer_function(num, den)
    return latex_expanded.replace("G(s) = ", "")


def _stability_margins(system):
    gain_margin, phase_margin, _, phase_cross, gain_cross, _ = ct.stability_margins(system)
    gain_margin = float(gain_margin)
    gain_margin_db = np.inf if np.isinf(gain_margin) else 20.0 * np.log10(max(gain_margin, np.finfo(float).tiny))
    return {
        "gain_margin_db": json_safe_number(float(gain_margin_db)),
        "phase_margin_deg": json_safe_number(float(phase_margin)),
        "phase_crossover_frequency": json_safe_number(float(phase_cross)),
        "gain_crossover_frequency": json_safe_number(float(gain_cross)),
    }


def design_controller(
    plant_expression,
    *,
    design_type="pid",
    method="zn_critical",
    controller_type="PID",
    process_gain=1.0,
    delay=1.0,
    time_constant=4.0,
    critical_gain=6.0,
    critical_period=2.0,
    chr_response="0",
    compensator_type="lead",
    compensator_zero=1.0,
    compensator_pole=5.0,
    compensator_gain=1.0,
    design_domain="frequency",
    final_time=None,
    points=900,
):
    plant, _, _, plant_latex_expanded, plant_latex_factored = parse_siso_transfer_function(plant_expression)
    design_type = str(design_type).lower()
    controller_type = str(controller_type).upper()
    parameters = None

    if design_type == "pid":
        parameters = _pid_parameters(
            method,
            controller_type,
            process_gain=process_gain,
            delay=delay,
            time_constant=time_constant,
            critical_gain=critical_gain,
            critical_period=critical_period,
            chr_response=chr_response,
        )
        controller = _pid_transfer(parameters, controller_type)
        design_label = f"{controller_type} por {parameters['method_label']}"
    elif design_type == "lead_lag":
        zero = _positive(compensator_zero, "A frequência do zero")
        pole = _positive(compensator_pole, "A frequência do polo")
        gain = _positive(compensator_gain, "O ganho do compensador")
        compensator_type = str(compensator_type).lower()
        if compensator_type == "lead" and pole <= zero:
            raise ValueError("Para avanço de fase, informe polo maior que o zero.")
        if compensator_type == "lag" and pole >= zero:
            raise ValueError("Para atraso de fase, informe polo menor que o zero.")
        controller = ct.tf([gain, gain * zero], [1.0, pole])
        kind = "avanço" if compensator_type == "lead" else "atraso"
        domain = "Bode" if str(design_domain).lower() == "frequency" else "LGR"
        design_label = f"Compensador por {kind} de fase — referência {domain}"
    else:
        raise ValueError("Escolha sintonia PID ou compensador avanço/atraso.")

    uncompensated = ct.feedback(plant, 1)
    compensated_loop = controller * plant
    compensated = ct.feedback(compensated_loop, 1)
    time = recommended_time_vector(compensated, final_time, points)
    t_before, y_before = response_arrays(ct.step_response(uncompensated, timepts=time))
    t_after, y_after = response_arrays(ct.step_response(compensated, timepts=time))
    metrics_before = _step_metrics(uncompensated, time)
    metrics_after = _step_metrics(compensated, time)

    gains = np.concatenate(([0.0], np.logspace(-3, 3, 260)))
    locus_before = _root_locus_roots(plant, gains)
    locus_after = _root_locus_roots(compensated_loop, gains)
    omega = np.logspace(-3, 3, 700)
    frequency_before = ct.frequency_response(plant, omega)
    frequency_after = ct.frequency_response(compensated_loop, omega)
    magnitude_before = 20.0 * np.log10(np.maximum(np.asarray(frequency_before.magnitude).squeeze(), np.finfo(float).tiny))
    magnitude_after = 20.0 * np.log10(np.maximum(np.asarray(frequency_after.magnitude).squeeze(), np.finfo(float).tiny))
    phase_before = np.unwrap(np.asarray(frequency_before.phase).squeeze()) * 180.0 / np.pi
    phase_after = np.unwrap(np.asarray(frequency_after.phase).squeeze()) * 180.0 / np.pi
    margins_before = _stability_margins(plant)
    margins_after = _stability_margins(compensated_loop)

    fig, axes = plt.subplots(2, 2, figsize=(11.2, 8.2))
    ax_step, ax_locus, ax_mag, ax_phase = axes.flat
    ax_step.plot(t_before, y_before, color="#64748b", linewidth=1.8, label="Antes")
    ax_step.plot(t_after, y_after, color="#2563eb", linewidth=2.2, label="Depois")
    ax_step.set_title("Resposta ao degrau", loc="left", fontweight="bold")
    ax_step.set_xlabel("Tempo (s)")
    ax_step.set_ylabel("Saída")
    ax_step.grid(True, alpha=0.25)
    ax_step.legend()

    for branch in range(locus_before.shape[1]):
        ax_locus.plot(np.real(locus_before[:, branch]), np.imag(locus_before[:, branch]), color="#94a3b8", linewidth=1.0, alpha=0.8)
    for branch in range(locus_after.shape[1]):
        ax_locus.plot(np.real(locus_after[:, branch]), np.imag(locus_after[:, branch]), color="#7c3aed", linewidth=1.5)
    ax_locus.axhline(0, color="#64748b", linewidth=0.7)
    ax_locus.axvline(0, color="#64748b", linewidth=0.7)
    ax_locus.set_title("LGR antes (cinza) e depois (roxo)", loc="left", fontweight="bold")
    ax_locus.set_xlabel("Eixo real")
    ax_locus.set_ylabel("Eixo imaginário")
    ax_locus.grid(True, alpha=0.25)

    ax_mag.semilogx(omega, magnitude_before, color="#64748b", linewidth=1.7, label="Antes")
    ax_mag.semilogx(omega, magnitude_after, color="#059669", linewidth=2.0, label="Depois")
    ax_mag.set_title("Bode — magnitude", loc="left", fontweight="bold")
    ax_mag.set_xlabel("Frequência (rad/s)")
    ax_mag.set_ylabel("Magnitude (dB)")
    ax_mag.grid(True, which="both", alpha=0.25)
    ax_mag.legend()

    ax_phase.semilogx(omega, phase_before, color="#64748b", linewidth=1.7, label="Antes")
    ax_phase.semilogx(omega, phase_after, color="#f59e0b", linewidth=2.0, label="Depois")
    ax_phase.set_title("Bode — fase", loc="left", fontweight="bold")
    ax_phase.set_xlabel("Frequência (rad/s)")
    ax_phase.set_ylabel("Fase (graus)")
    ax_phase.grid(True, which="both", alpha=0.25)
    ax_phase.legend()
    fig.suptitle(design_label, fontsize=13, fontweight="bold")
    fig.tight_layout()
    image, svg = figure_payload(fig)

    def metric_value(metrics, key):
        value = metrics.get(key)
        return "Não definido" if value is None else str(value)

    metric_rows = [
        {"label": "Projeto", "value": design_label},
        {"label": "Sobressinal antes", "value": metric_value(metrics_before, "overshoot_percent"), "unit": "%"},
        {"label": "Sobressinal depois", "value": metric_value(metrics_after, "overshoot_percent"), "unit": "%"},
        {"label": "Acomodação antes", "value": metric_value(metrics_before, "settling_time"), "unit": "s"},
        {"label": "Acomodação depois", "value": metric_value(metrics_after, "settling_time"), "unit": "s"},
        {"label": "Margem de ganho antes", "value": str(margins_before["gain_margin_db"]), "unit": "dB"},
        {"label": "Margem de ganho depois", "value": str(margins_after["gain_margin_db"]), "unit": "dB"},
        {"label": "Margem de fase antes", "value": str(margins_before["phase_margin_deg"]), "unit": "°"},
        {"label": "Margem de fase depois", "value": str(margins_after["phase_margin_deg"]), "unit": "°"},
    ]
    if parameters:
        metric_rows.extend([
            {"label": "Kp", "value": f"{parameters['kp']:.6g}"},
            {"label": "Ki", "value": f"{parameters['ki']:.6g}"},
            {"label": "Kd", "value": f"{parameters['kd']:.6g}"},
        ])

    details = {
        "design_type": design_type,
        "design_label": design_label,
        "controller_parameters": parameters,
        "before": metrics_before,
        "after": metrics_after,
        "frequency_margins_before": margins_before,
        "frequency_margins_after": margins_after,
        "closed_loop_poles_before": serialize_complex(ct.poles(uncompensated)),
        "closed_loop_poles_after": serialize_complex(ct.poles(compensated)),
        "plant_latex_expanded": plant_latex_expanded,
    }
    return {
        "success": True,
        "module": "controller_design",
        "title": "Projeto de Controladores",
        "description": "Comparação da resposta ao degrau e do LGR antes e depois da compensação.",
        "image": image,
        "svg": svg,
        "latex": [
            {"label": "Planta", "value": plant_latex_factored.replace("G(s) = ", "")},
            {"label": "Controlador", "value": rf"C(s)={_controller_latex(controller)}"},
            {"label": "Malha fechada compensada", "value": r"T_c(s)=\frac{C(s)G(s)}{1+C(s)G(s)}"},
        ],
        "metrics": metric_rows,
        "details": details,
    }


def get_controller_presets():
    return [
        {"id": name, "title": name, "expr": expression}
        for name, expression in CONTROLLER_PRESETS.items()
    ]
