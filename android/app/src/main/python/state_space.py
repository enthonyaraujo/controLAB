"""Modelagem, análise estrutural e síntese em espaço de estados."""

from __future__ import annotations

import ast

import control as ct
import matplotlib.pyplot as plt
import numpy as np
import sympy as sp

from control_utils import figure_payload, parse_siso_transfer_function, serialize_complex
from lgr_engine import format_transfer_function


STATE_SPACE_PRESETS = {
    "Segunda ordem controlável": {
        "A": "[[0, 1], [-2, -3]]",
        "B": "[[0], [1]]",
        "C": "[[1, 0]]",
        "D": "[[0]]",
    },
    "Oscilador harmônico": {
        "A": "[[0, 1], [-4, 0]]",
        "B": "[[0], [1]]",
        "C": "[[1, 0]]",
        "D": "[[0]]",
    },
}


def parse_matrix(value, name):
    """Aceita JSON/Python matricial ou linhas separadas por ponto e vírgula."""
    if isinstance(value, (list, tuple, np.ndarray)):
        parsed = value
    else:
        text = str(value).strip()
        if not text:
            raise ValueError(f"A matriz {name} não pode ficar vazia.")
        try:
            parsed = ast.literal_eval(text)
        except (SyntaxError, ValueError):
            rows = [row.strip() for row in text.split(";") if row.strip()]
            parsed = [
                [float(item) for item in row.replace(",", " ").split()]
                for row in rows
            ]
    matrix = np.asarray(parsed, dtype=float)
    if matrix.ndim == 0:
        matrix = matrix.reshape(1, 1)
    elif matrix.ndim == 1:
        matrix = matrix.reshape(1, -1)
    if matrix.ndim != 2 or not np.all(np.isfinite(matrix)):
        raise ValueError(f"A matriz {name} deve ser bidimensional e conter apenas números finitos.")
    return matrix


def parse_poles(value, expected, label):
    if value is None or (isinstance(value, str) and not value.strip()):
        return []
    if isinstance(value, (list, tuple, np.ndarray)):
        poles = [complex(item) for item in value]
    else:
        poles = [complex(item.strip().replace("i", "j")) for item in str(value).split(",") if item.strip()]
    if len(poles) != expected:
        raise ValueError(f"{label} exige exatamente {expected} polos.")
    return poles


def _validate_dimensions(a, b, c, d):
    states = a.shape[0]
    if a.shape[1] != states:
        raise ValueError("A matriz A deve ser quadrada.")
    if b.shape[0] != states:
        raise ValueError("B deve possuir o mesmo número de linhas de A.")
    if c.shape[1] != states:
        raise ValueError("C deve possuir o mesmo número de colunas de A.")
    if d.shape != (c.shape[0], b.shape[1]):
        raise ValueError("D deve possuir dimensões saídas × entradas.")


def _serialize_matrix(matrix):
    values = np.asarray(matrix)
    result = []
    for row in values:
        serialized_row = []
        for value in row:
            candidate = complex(value)
            if abs(candidate.imag) < 1e-10:
                serialized_row.append(float(candidate.real))
            else:
                serialized_row.append(f"{candidate.real:.6g}{candidate.imag:+.6g}j")
        result.append(serialized_row)
    return result


def _matrix_latex(name, matrix):
    sympy_matrix = sp.Matrix(np.real_if_close(np.asarray(matrix)).tolist())
    return rf"{name}={sp.latex(sympy_matrix)}"


def _canonical_form(system, form):
    form = str(form).lower()
    if form in {"none", "original", ""}:
        return system, np.eye(system.nstates), "Original"
    if system.ninputs != 1 or system.noutputs != 1:
        raise ValueError("As formas canônicas selecionadas exigem um sistema SISO.")
    if form in {"controllable", "reachable"}:
        canonical, transform = ct.canonical_form(system, "reachable")
        return canonical, transform, "Canônica controlável"
    if form == "observable":
        canonical, transform = ct.canonical_form(system, "observable")
        return canonical, transform, "Canônica observável"
    if form == "diagonal":
        eigenvalues, eigenvectors = np.linalg.eig(system.A)
        if np.linalg.matrix_rank(eigenvectors) < system.nstates:
            raise ValueError("A matriz A não é diagonalizável; use a forma de Jordan.")
        inverse = np.linalg.inv(eigenvectors)
        canonical = ct.ss(
            np.real_if_close(np.diag(eigenvalues)),
            np.real_if_close(inverse @ system.B),
            np.real_if_close(system.C @ eigenvectors),
            system.D,
        )
        return canonical, inverse, "Canônica diagonal"
    if form == "jordan":
        try:
            transform_sympy, jordan_sympy = sp.Matrix(system.A).jordan_form()
            transform = np.asarray(transform_sympy, dtype=complex)
            jordan = np.asarray(jordan_sympy, dtype=complex)
            inverse = np.linalg.inv(transform)
            canonical = ct.ss(
                np.real_if_close(jordan),
                np.real_if_close(inverse @ system.B),
                np.real_if_close(system.C @ transform),
                system.D,
            )
            return canonical, inverse, "Forma de Jordan"
        except (ValueError, TypeError, np.linalg.LinAlgError) as exc:
            raise ValueError("Não foi possível obter uma forma de Jordan numericamente estável.") from exc
    raise ValueError("Forma canônica desconhecida.")


def _transfer_details(system):
    transfer = ct.ss2tf(system)
    if system.ninputs == 1 and system.noutputs == 1:
        num = np.asarray(ct.tfdata(transfer)[0][0][0], dtype=float)
        den = np.asarray(ct.tfdata(transfer)[1][0][0], dtype=float)
        num_out, den_out, latex_expanded, latex_factored = format_transfer_function(num.tolist(), den.tolist())
        return {
            "siso": True,
            "numerator": num_out,
            "denominator": den_out,
            "latex": latex_factored.replace("G(s) = ", ""),
            "latex_expanded": latex_expanded,
        }
    return {
        "siso": False,
        "representation": "G(s) = C(sI-A)^{-1}B + D",
        "inputs": system.ninputs,
        "outputs": system.noutputs,
    }


def analyze_state_space(
    *,
    mode="matrices",
    expression="1 / (s^2 + 3*s + 2)",
    a="[[0, 1], [-2, -3]]",
    b="[[0], [1]]",
    c="[[1, 0]]",
    d="[[0]]",
    canonical_form="controllable",
    desired_poles="",
    observer_poles="",
):
    mode = str(mode).lower()
    source_latex = None
    if mode == "transfer":
        transfer, _, _, source_latex, _ = parse_siso_transfer_function(expression)
        system = ct.tf2ss(transfer)
    elif mode == "matrices":
        matrix_a = parse_matrix(a, "A")
        matrix_b = parse_matrix(b, "B")
        matrix_c = parse_matrix(c, "C")
        matrix_d = parse_matrix(d, "D")
        _validate_dimensions(matrix_a, matrix_b, matrix_c, matrix_d)
        system = ct.ss(matrix_a, matrix_b, matrix_c, matrix_d)
    else:
        raise ValueError("Escolha entrada por matrizes ou função de transferência.")

    states = system.nstates
    controllability = ct.ctrb(system.A, system.B)
    observability = ct.obsv(system.A, system.C)
    controllability_rank = int(np.linalg.matrix_rank(controllability))
    observability_rank = int(np.linalg.matrix_rank(observability))

    canonical, transform, canonical_label = _canonical_form(system, canonical_form)
    requested_poles = parse_poles(desired_poles, states, "A realimentação de estados")
    requested_observer_poles = parse_poles(observer_poles, states, "O observador")

    gain_k = None
    feedback_poles = []
    placement_method = None
    if requested_poles:
        if controllability_rank < states:
            raise ValueError("O sistema não é completamente controlável; não é possível alocar todos os polos.")
        if system.ninputs == 1:
            gain_k = np.atleast_2d(ct.acker(system.A, system.B, requested_poles))
            placement_method = "Fórmula de Ackermann"
        else:
            gain_k = np.asarray(ct.place(system.A, system.B, requested_poles))
            placement_method = "Alocação multivariável"
        feedback_poles = np.linalg.eigvals(system.A - system.B @ gain_k)

    observer_gain = None
    observer_error_poles = []
    observer_method = None
    if requested_observer_poles:
        if observability_rank < states:
            raise ValueError("O sistema não é completamente observável; não é possível projetar o observador completo.")
        if system.noutputs == 1:
            observer_gain = np.atleast_2d(ct.acker(system.A.T, system.C.T, requested_observer_poles)).T
            observer_method = "Ackermann dual — Luenberger"
        else:
            observer_gain = np.asarray(ct.place(system.A.T, system.C.T, requested_observer_poles)).T
            observer_method = "Alocação dual multivariável — Luenberger"
        observer_error_poles = np.linalg.eigvals(system.A - observer_gain @ system.C)

    original_poles = np.linalg.eigvals(system.A)
    fig, axis = plt.subplots(figsize=(8.4, 5.4))
    axis.scatter(np.real(original_poles), np.imag(original_poles), marker="x", s=90, linewidths=2.2, color="#64748b", label="Sistema original")
    if len(feedback_poles):
        axis.scatter(np.real(feedback_poles), np.imag(feedback_poles), marker="o", s=70, facecolors="none", linewidths=2, color="#2563eb", label="A - BK")
    if len(observer_error_poles):
        axis.scatter(np.real(observer_error_poles), np.imag(observer_error_poles), marker="s", s=55, facecolors="none", linewidths=2, color="#7c3aed", label="A - LC")
    axis.axhline(0, color="#64748b", linewidth=0.8)
    axis.axvline(0, color="#64748b", linewidth=0.8)
    axis.set_xlabel("Parte real")
    axis.set_ylabel("Parte imaginária")
    axis.set_title("Polos no plano complexo", loc="left", fontweight="bold")
    axis.grid(True, alpha=0.25)
    axis.legend()
    fig.tight_layout()
    image, svg = figure_payload(fig)

    transfer_details = _transfer_details(system)
    metrics = [
        {"label": "Estados", "value": str(states)},
        {"label": "Entradas", "value": str(system.ninputs)},
        {"label": "Saídas", "value": str(system.noutputs)},
        {"label": "Posto de controlabilidade", "value": f"{controllability_rank} / {states}"},
        {"label": "Controlável", "value": "Sim" if controllability_rank == states else "Não"},
        {"label": "Posto de observabilidade", "value": f"{observability_rank} / {states}"},
        {"label": "Observável", "value": "Sim" if observability_rank == states else "Não"},
        {"label": "Representação", "value": canonical_label},
    ]
    if placement_method:
        metrics.append({"label": "Realimentação", "value": placement_method})
    if observer_method:
        metrics.append({"label": "Observador", "value": observer_method})

    latex = [
        {"label": "Modelo", "value": r"\dot{x}=Ax+Bu,\qquad y=Cx+Du"},
        {"label": "Controlabilidade", "value": r"\mathcal{C}=[B\;AB\;\cdots\;A^{n-1}B]"},
        {"label": "Observabilidade", "value": r"\mathcal{O}=[C^T\;A^TC^T\;\cdots\;(A^T)^{n-1}C^T]^T"},
    ]
    if transfer_details["siso"]:
        latex.append({"label": "Função de transferência", "value": transfer_details["latex"]})
    if gain_k is not None:
        latex.append({"label": "Realimentação de estados", "value": _matrix_latex("K", gain_k)})
    if observer_gain is not None:
        latex.append({"label": "Ganho do observador", "value": _matrix_latex("L", observer_gain)})

    details = {
        "source_mode": mode,
        "source_latex": source_latex,
        "A": _serialize_matrix(system.A),
        "B": _serialize_matrix(system.B),
        "C": _serialize_matrix(system.C),
        "D": _serialize_matrix(system.D),
        "transfer_function": transfer_details,
        "controllability_matrix": _serialize_matrix(controllability),
        "controllability_rank": controllability_rank,
        "observability_matrix": _serialize_matrix(observability),
        "observability_rank": observability_rank,
        "canonical": {
            "label": canonical_label,
            "A": _serialize_matrix(canonical.A),
            "B": _serialize_matrix(canonical.B),
            "C": _serialize_matrix(canonical.C),
            "D": _serialize_matrix(canonical.D),
            "transformation": _serialize_matrix(transform),
        },
        "state_feedback_gain": _serialize_matrix(gain_k) if gain_k is not None else None,
        "state_feedback_method": placement_method,
        "observer_gain": _serialize_matrix(observer_gain) if observer_gain is not None else None,
        "observer_method": observer_method,
        "original_poles": serialize_complex(original_poles),
        "feedback_poles": serialize_complex(feedback_poles),
        "observer_error_poles": serialize_complex(observer_error_poles),
    }
    return {
        "success": True,
        "module": "state_space",
        "title": "Espaço de Estados",
        "description": "Conversão de modelos, testes estruturais, realimentação de estados e observador de ordem plena.",
        "image": image,
        "svg": svg,
        "latex": latex,
        "metrics": metrics,
        "details": details,
    }


def get_state_space_presets():
    return [
        {"id": name, "title": name, **matrices}
        for name, matrices in STATE_SPACE_PRESETS.items()
    ]
