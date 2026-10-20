"""Motor matemático universal e sintonia de controladores em série via LGR e métodos clássicos."""

from __future__ import annotations

import math
from typing import Any, Iterable

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
from lgr_engine import format_transfer_function


CONTROLLER_PRESETS = {
    "Avanço de Fase — G(s) = 5 / (s(s+1))": {
        "plant_expr": "5 / (s * (s + 1))",
        "controller_type": "lead",
        "input_type": "step",
        "settling_criterion": "2",
        "ts": 2.0,
        "mp": 16.0,
        "xi": 0.5,
    },
    "Atraso de Fase — G(s) = 1 / ((s+0.5)(s+2))": {
        "plant_expr": "1 / ((s + 0.5) * (s + 2))",
        "controller_type": "lag",
        "input_type": "step",
        "settling_criterion": "2",
        "error_reduction_factor": 10.0,
        "lag_pole": 0.01,
        "lag_gain": 1.0,
    },
    "Proporcional (P) Ex. 1 — G(s) = 1 / (s(s+1)(s+2))": {
        "plant_expr": "1 / (s * (s + 1) * (s + 2))",
        "controller_type": "P",
        "input_type": "step",
        "settling_criterion": "2",
        "ts": 10.0,
        "mp": 10.0,
        "xi": 0.591,
    },
    "Proporcional (P) Ex. 2 — G(s) = 1 / ((s+1)(s+5))": {
        "plant_expr": "1 / ((s + 1) * (s + 5))",
        "controller_type": "P",
        "input_type": "step",
        "settling_criterion": "2",
        "ts": 2.0,
        "mp": 10.0,
        "xi": 0.591,
    },
    "Proporcional-Derivativo (PD) Ex. 1 — G(s) = 40 / (s(s+2))": {
        "plant_expr": "40 / (s * (s + 2))",
        "controller_type": "PD",
        "input_type": "step",
        "settling_criterion": "2",
        "ts": 1.0,
        "mp": 25.0,
        "xi": 0.4,
    },
    "Proporcional-Derivativo (PD) Ex. 2 — G(s) = 1 / (s(s+4)(s+6))": {
        "plant_expr": "1 / (s * (s + 4) * (s + 6))",
        "controller_type": "PD",
        "input_type": "step",
        "settling_criterion": "2",
        "xi": 0.456,
        "wn": 5.0,
    },
    "Proporcional-Integral (PI) Ex. 1 — G(s) = 10 / ((s+2)(s+5))": {
        "plant_expr": "10 / ((s + 2) * (s + 5))",
        "controller_type": "PI",
        "input_type": "step",
        "settling_criterion": "5",
        "ts": 1.5,
        "mp": 0.63,
        "xi": 0.857,
    },
    "Proporcional-Integral (PI) Ex. 2 — G(s) = 1 / (s(s+4))": {
        "plant_expr": "1 / (s * (s + 4))",
        "controller_type": "PI",
        "input_type": "ramp",
        "settling_criterion": "5",
        "ts": 2.0,
        "mp": 10.0,
        "xi": 0.6,
    },
}


def _fmt_num(val: float | None, digits: int = 4) -> str:
    if val is None:
        return "—"
    if isinstance(val, str):
        return val
    if not math.isfinite(float(val)):
        return "∞" if float(val) > 0 else "-∞"
    v = float(val)
    if abs(v) < 1e-12:
        v = 0.0
    if abs(v - round(v)) < 1e-9:
        return str(int(round(v)))
    text = f"{v:.{digits}g}"
    return text.replace(".", ",")


def _fmt_tex_num(val: float | None, digits: int = 4) -> str:
    if val is None:
        return "0"
    if isinstance(val, str):
        return val
    if not math.isfinite(float(val)):
        return r"\infty"
    v = float(val)
    if abs(v) < 1e-12:
        v = 0.0
    if abs(v - round(v)) < 1e-9:
        return str(int(round(v)))
    return f"{v:.{digits}g}".replace(".", "{,}")


def _to_optional_float(value: Any) -> float | None:
    if value is None or value == "":
        return None
    if isinstance(value, (int, float, np.number)):
        fval = float(value)
        return fval if math.isfinite(fval) else None
    text = str(value).strip().replace(",", ".")
    if not text:
        return None
    fval = float(text)
    return fval if math.isfinite(fval) else None


def _positive(value: Any, label: str) -> float:
    number = _to_optional_float(value)
    if number is None or number <= 0:
        raise ValueError(f"{label} deve ser maior que zero.")
    return number


# ==============================================================================
# FUNÇÕES UNIVERSAIS DO MOTOR MATEMÁTICO DE PROJETO DE CONTROLADORES (LGR)
# ==============================================================================

def mp_from_xi(xi: float) -> float:
    """Converte coeficiente de amortecimento xi para sobressinal Mp(%)."""
    xi = float(xi)
    if xi <= 0:
        return 100.0
    if xi >= 1.0:
        return 0.0
    return float(math.exp(-(xi * math.pi) / math.sqrt(1.0 - xi * xi)) * 100.0)


def xi_from_mp(mp_percent: float) -> float:
    """Isola o coeficiente de amortecimento xi a partir de Mp(%)."""
    mp = float(mp_percent)
    if mp <= 0:
        return 1.0
    if mp >= 100.0:
        raise ValueError("O sobressinal máximo Mp(%) deve estar entre 0% e 100%.")
    ln_mp = math.log(mp / 100.0)
    return float(-ln_mp / math.sqrt(math.pi * math.pi + ln_mp * ln_mp))


def calculate_wn_from_ts(ts: float, xi: float, criterion: str | int = "2") -> float:
    """Calcula wn a partir de ts(2%) = 4/(xi*wn) ou ts(5%) = 3/(xi*wn)."""
    ts_val = _positive(ts, "O tempo de acomodação ts")
    xi_val = _positive(xi, "O coeficiente de amortecimento ξ")
    crit_str = str(criterion).strip().replace("%", "")
    factor = 3.0 if crit_str == "5" else 4.0
    return float(factor / (xi_val * ts_val))


def calculate_ts_from_wn(wn: float, xi: float, criterion: str | int = "2") -> float:
    """Calcula ts(2%) ou ts(5%) a partir de wn e xi."""
    wn_val = _positive(wn, "A frequência natural ωn")
    xi_val = _positive(xi, "O coeficiente de amortecimento ξ")
    crit_str = str(criterion).strip().replace("%", "")
    factor = 3.0 if crit_str == "5" else 4.0
    return float(factor / (xi_val * wn_val))


def resolve_transient_specifications(
    *,
    ts: Any = None,
    mp: Any = None,
    xi: Any = None,
    wn: Any = None,
    settling_criterion: str | int = "2",
    primary_damping_spec: str | None = None,
    primary_freq_spec: str | None = None,
) -> dict[str, Any]:
    """
    Resolve automaticamente os parâmetros transitórios (ts, Mp, xi, wn)
    e os polos dominantes desejados s = -Re +- Im*j.
    """
    crit_str = str(settling_criterion or "2").strip().replace("%", "")
    crit_pct = 5 if crit_str == "5" else 2
    settling_factor = 3.0 if crit_pct == 5 else 4.0

    ts_in = _to_optional_float(ts)
    mp_in = _to_optional_float(mp)
    xi_in = _to_optional_float(xi)
    wn_in = _to_optional_float(wn)

    # 1. Determina xi e Mp(%)
    damping_source = "xi"
    if primary_damping_spec == "mp" and mp_in is not None and 0 < mp_in < 100:
        xi_val = xi_from_mp(mp_in)
        mp_val = mp_in
        damping_source = "mp"
    elif xi_in is not None and xi_in > 0:
        xi_val = xi_in
        mp_val = mp_in if (mp_in is not None and mp_in >= 0) else mp_from_xi(xi_val)
        damping_source = "xi"
    elif mp_in is not None and 0 < mp_in < 100:
        xi_val = xi_from_mp(mp_in)
        mp_val = mp_in
        damping_source = "mp"
    elif ts_in is not None and ts_in > 0 and wn_in is not None and wn_in > 0:
        re_tmp = settling_factor / ts_in
        xi_val = re_tmp / wn_in
        if not (0 < xi_val < 1):
            raise ValueError("A combinação de ts e ωn resulta em ξ fora da faixa subamortecida (0 < ξ < 1).")
        mp_val = mp_from_xi(xi_val)
        damping_source = "ts_wn"
    else:
        xi_val = 0.5
        mp_val = mp_from_xi(xi_val)
        damping_source = "default"

    if not (0 < xi_val < 1):
        raise ValueError("O coeficiente de amortecimento ξ deve estar entre 0 e 1 para polos dominantes complexos.")

    # 2. Determina wn, ts, Re e Im
    freq_source = "ts"
    if primary_freq_spec == "wn" and wn_in is not None and wn_in > 0:
        wn_val = wn_in
        re_val = xi_val * wn_val
        ts_val = settling_factor / re_val
        freq_source = "wn"
    elif ts_in is not None and ts_in > 0 and (wn_in is None or primary_freq_spec != "wn"):
        ts_val = ts_in
        re_val = settling_factor / ts_val
        wn_val = re_val / xi_val
        freq_source = "ts"
    elif wn_in is not None and wn_in > 0:
        wn_val = wn_in
        re_val = xi_val * wn_val
        ts_val = settling_factor / re_val
        freq_source = "wn"
    else:
        ts_val = 2.0
        re_val = settling_factor / ts_val
        wn_val = re_val / xi_val
        freq_source = "default"

    im_val = wn_val * math.sqrt(1.0 - xi_val * xi_val)
    s_dom = complex(-re_val, im_val)

    return {
        "criterion_pct": crit_pct,
        "settling_factor": settling_factor,
        "ts": float(ts_val),
        "mp": float(mp_val),
        "mp_calculated": float(mp_from_xi(xi_val)),
        "xi": float(xi_val),
        "wn": float(wn_val),
        "re": float(re_val),
        "im": float(im_val),
        "dominant_pole": s_dom,
        "damping_source": damping_source,
        "freq_source": freq_source,
    }


def _group_roots(roots: Iterable[complex], tol: float = 1e-4) -> list[dict[str, Any]]:
    """Agrupa raízes reais ou complexas próximas registrando sua multiplicidade."""
    grouped: list[dict[str, Any]] = []
    sorted_roots = sorted(roots, key=lambda r: (-round(float(np.real(r)), 4), round(float(np.imag(r)), 4)))
    for r in sorted_roots:
        r_complex = complex(float(np.real(r)), float(np.imag(r)))
        if abs(r_complex.imag) < tol:
            r_complex = complex(r_complex.real, 0.0)
        if abs(r_complex.real) < tol and abs(r_complex.imag) < tol:
            r_complex = complex(0.0, 0.0)
        matched = False
        for item in grouped:
            if abs(r_complex - item["root"]) <= tol:
                item["multiplicity"] += 1
                matched = True
                break
        if not matched:
            grouped.append({
                "root": r_complex,
                "is_real": abs(r_complex.imag) < tol,
                "pos": float(-r_complex.real),  # p_i tal que fator é (s + p_i)
                "multiplicity": 1,
            })
    return grouped


def calculate_lgr_angle(pos: float, re_dom: float, im_dom: float, tol: float = 1e-7) -> dict[str, Any]:
    """
    Calcula o ângulo (em graus) de um polo ou zero real localizado em -pos (fator s + pos)
    em relação ao polo dominante s = -Re + Im*j:
      - Se pos < Re (à direita de -Re): theta = 180° - atan(Im / (Re - pos))
      - Se pos == Re (alinhado verticalmente com -Re): theta = 90°
      - Se pos > Re (à esquerda de -Re): theta = atan(Im / (pos - Re))
    """
    p_i = float(pos)
    re_v = float(re_dom)
    im_v = float(im_dom)

    if abs(p_i - re_v) <= tol:
        angle_deg = 90.0
        relation = "equal"
        formula_latex = (
            rf"p_i = \text{{Re}} = {_fmt_tex_num(re_v)} \implies \theta = 90^\circ"
        )
    elif p_i < re_v:
        delta = re_v - p_i
        atan_deg = math.degrees(math.atan(im_v / delta))
        angle_deg = 180.0 - atan_deg
        relation = "right"
        formula_latex = (
            rf"p_i = {_fmt_tex_num(p_i)} < \text{{Re}} ({_fmt_tex_num(re_v)}) \implies "
            rf"\theta = 180^\circ - \tan^{{-1}}\!\left(\frac{{{_fmt_tex_num(im_v)}}}{{{_fmt_tex_num(re_v)} - {_fmt_tex_num(p_i)}}}\right) "
            rf"= 180^\circ - {_fmt_tex_num(atan_deg, 4)}^\circ = {_fmt_tex_num(angle_deg, 4)}^\circ"
        )
    else:
        delta = p_i - re_v
        angle_deg = math.degrees(math.atan(im_v / delta))
        relation = "left"
        formula_latex = (
            rf"p_i = {_fmt_tex_num(p_i)} > \text{{Re}} ({_fmt_tex_num(re_v)}) \implies "
            rf"\theta = \tan^{{-1}}\!\left(\frac{{{_fmt_tex_num(im_v)}}}{{{_fmt_tex_num(p_i)} - {_fmt_tex_num(re_v)}}}\right) "
            rf"= {_fmt_tex_num(angle_deg, 4)}^\circ"
        )

    return {
        "pos": p_i,
        "root": -p_i,
        "angle_deg": float(angle_deg),
        "relation": relation,
        "formula_latex": formula_latex,
    }


def calculate_complex_root_angle(root: complex, re_dom: float, im_dom: float) -> dict[str, Any]:
    """Calcula o ângulo para polos/zeros gerais (reais ou complexos)."""
    if abs(root.imag) < 1e-6:
        return calculate_lgr_angle(-root.real, re_dom, im_dom)
    dx = -re_dom - root.real
    dy = im_dom - root.imag
    angle_deg = math.degrees(math.atan2(dy, dx))
    if angle_deg < 0:
        angle_deg += 360.0
    formula_latex = (
        rf"\theta = \angle\!\left((-{_fmt_tex_num(re_dom)} + {_fmt_tex_num(im_dom)}j) - "
        rf"({_fmt_tex_num(root.real)} {_fmt_tex_num(root.imag):+}j)\right) = {_fmt_tex_num(angle_deg, 4)}^\circ"
    )
    return {
        "pos": float(-root.real),
        "root": root,
        "angle_deg": float(angle_deg),
        "relation": "complex",
        "formula_latex": formula_latex,
    }


def normalize_phase_angle(raw_angle_deg: float) -> float:
    """Reduz múltiplos de 360° / 180° para obter o ângulo exigido no intervalo (0°, 180°)."""
    angle = float(raw_angle_deg) % 360.0
    if angle < 0:
        angle += 360.0
    if angle > 180.0 + 1e-7:
        angle = angle - 180.0
    if abs(angle) < 1e-7 or abs(angle - 180.0) < 1e-7:
        # Mantém dentro do intervalo aberto quando próximo da fronteira
        angle = max(1e-4, min(179.9999, angle))
    return float(angle)


def calculate_position_from_angle(theta_deg: float, re_dom: float, im_dom: float, symbol: str = "z", tol: float = 1e-6) -> dict[str, Any]:
    """
    Calcula a posição (pos = z ou p) no eixo real (-pos) a partir do ângulo theta exigido:
      - Se theta < 90° (à esquerda de -Re): tan^-1(Im / (pos - Re)) = theta => pos = (Im + Re*tan(theta)) / tan(theta)
      - Se theta == 90° (alinhado com -Re): pos = Re
      - Se theta > 90° (à direita de -Re): 180° - tan^-1(Im / (Re - pos)) = theta => -Im / (Re - pos) = tan(theta)
    """
    theta = float(theta_deg)
    re_v = float(re_dom)
    im_v = float(im_dom)

    if abs(theta - 90.0) <= tol:
        pos = re_v
        relation = "aligned"
        formula_latex = rf"\theta_{{{symbol}}} = 90^\circ \implies {symbol} = \text{{Re}} = {_fmt_tex_num(pos)}"
    elif theta < 90.0:
        tan_th = math.tan(math.radians(theta))
        pos = (im_v + re_v * tan_th) / tan_th
        relation = "left"
        formula_latex = (
            rf"\theta_{{{symbol}}} = {_fmt_tex_num(theta, 4)}^\circ < 90^\circ \implies "
            rf"\tan^{{-1}}\!\left(\frac{{{_fmt_tex_num(im_v)}}}{{{symbol} - {_fmt_tex_num(re_v)}}}\right) = {_fmt_tex_num(theta, 4)}^\circ "
            rf"\implies {symbol} = \frac{{{_fmt_tex_num(im_v)} + {_fmt_tex_num(re_v)} \times \tan({_fmt_tex_num(theta, 4)}^\circ)}}{{\tan({_fmt_tex_num(theta, 4)}^\circ)}} "
            rf"= {_fmt_tex_num(pos, 4)}"
        )
    else:
        tan_th = math.tan(math.radians(theta))
        pos = re_v + im_v / tan_th
        relation = "right"
        formula_latex = (
            rf"\theta_{{{symbol}}} = {_fmt_tex_num(theta, 4)}^\circ > 90^\circ \implies "
            rf"180^\circ - \tan^{{-1}}\!\left(\frac{{{_fmt_tex_num(im_v)}}}{{{_fmt_tex_num(re_v)} - {symbol}}}\right) = {_fmt_tex_num(theta, 4)}^\circ "
            rf"\implies -\frac{{{_fmt_tex_num(im_v)}}}{{{_fmt_tex_num(re_v)} - {symbol}}} = \tan({_fmt_tex_num(theta, 4)}^\circ) "
            rf"\implies {symbol} = {_fmt_tex_num(pos, 4)}"
        )

    return {
        "pos": float(pos),
        "theta_deg": float(theta),
        "relation": relation,
        "formula_latex": formula_latex,
    }


def _normalize_input_type(input_type: str | None) -> str:
    raw = str(input_type or "step").strip().lower()
    if raw in {"parabola", "parábola", "parabolic", "parabolica", "parabólica", "1/s^3", "1/s3"}:
        return "parabola"
    if raw in {"ramp", "rampa", "1/s^2", "1/s2"}:
        return "ramp"
    return "step"


def evaluate_steady_state_error(num: Iterable[float], den: Iterable[float], input_type: str = "step") -> dict[str, Any]:
    """
    Avalia o erro em regime permanente:
      e(inf) = lim_{s -> 0} s * (1 / (1 + G_ma(s))) * R(s)
    onde R(s) = 1/s (degrau unitário), R(s) = 1/s^2 (rampa unitária)
    ou R(s) = 1/s^3 (parábola unitária).
    """
    s = sp.Symbol("s", positive=True)
    num_list = [float(c) for c in num]
    den_list = [float(c) for c in den]
    n_expr = sum(sp.Float(c) * s ** (len(num_list) - 1 - i) for i, c in enumerate(num_list))
    d_expr = sum(sp.Float(c) * s ** (len(den_list) - 1 - i) for i, c in enumerate(den_list))
    g_expr = n_expr / d_expr

    mode = _normalize_input_type(input_type)
    if mode == "parabola":
        r_expr = 1 / (s ** 3)
        r_latex = r"\frac{1}{s^3}"
    elif mode == "ramp":
        r_expr = 1 / (s ** 2)
        r_latex = r"\frac{1}{s^2}"
    else:
        r_expr = 1 / s
        r_latex = r"\frac{1}{s}"

    expr = sp.simplify(s * (1 / (1 + g_expr)) * r_expr)
    try:
        lim_val = sp.limit(expr, s, 0, dir="+")
        if lim_val.is_infinite:
            value = math.inf
            latex_val = r"\infty"
        else:
            value = float(lim_val)
            if abs(value) < 1e-12:
                value = 0.0
            latex_val = _fmt_tex_num(value, 4)
    except Exception:
        value = math.nan
        latex_val = r"\text{indefinido}"

    formula_latex = (
        rf"e(\infty) = \lim_{{s \to 0}} s \left(\frac{{1}}{{1 + G_{{ma}}(s)}}\right) {r_latex} = {latex_val}"
    )
    return {
        "value": value,
        "json_value": json_safe_number(value),
        "latex_value": latex_val,
        "formula_latex": formula_latex,
        "input_type": mode,
    }


def analyze_uncompensated_system(num: np.ndarray, den: np.ndarray, input_type: str = "step") -> dict[str, Any]:
    """
    Analisa a malha fechada sem controlador T(s) = G(s)/(1 + G(s))
    e compara com a forma padrão de 2ª ordem T(s) = wn^2 / (s^2 + 2*xi*wn*s + wn^2).
    """
    order = max(len(num), len(den))
    num_pad = np.pad(num, (order - len(num), 0))
    den_pad = np.pad(den, (order - len(den), 0))
    cl_den = np.trim_zeros(den_pad + num_pad, "f")
    cl_num = np.trim_zeros(num, "f")

    cl_poles = np.roots(cl_den) if len(cl_den) > 1 else np.array([])
    stable = bool(len(cl_poles) > 0 and np.all(np.real(cl_poles) < -1e-9))

    wn_uncomp = None
    xi_uncomp = None
    mp_uncomp = None
    ts2_uncomp = None
    ts5_uncomp = None

    if len(cl_den) == 3 and cl_den[0] != 0:
        a1 = float(cl_den[1] / cl_den[0])
        a0 = float(cl_den[2] / cl_den[0])
        if a0 > 0:
            wn_uncomp = math.sqrt(a0)
            xi_uncomp = a1 / (2.0 * wn_uncomp)
            if 0 < xi_uncomp < 1:
                mp_uncomp = mp_from_xi(xi_uncomp)
                ts2_uncomp = 4.0 / (xi_uncomp * wn_uncomp)
                ts5_uncomp = 3.0 / (xi_uncomp * wn_uncomp)
    else:
        complex_poles = [p for p in cl_poles if abs(p.imag) > 1e-6 and p.real < 0]
        if complex_poles:
            dom = max(complex_poles, key=lambda p: p.real)
            wn_uncomp = abs(dom)
            xi_uncomp = -dom.real / wn_uncomp
            if 0 < xi_uncomp < 1:
                mp_uncomp = mp_from_xi(xi_uncomp)
                ts2_uncomp = 4.0 / (-dom.real)
                ts5_uncomp = 3.0 / (-dom.real)

    err_info = evaluate_steady_state_error(num, den, input_type=input_type)
    _, _, cl_latex_exp, _ = format_transfer_function(cl_num.tolist(), cl_den.tolist())
    cl_latex = cl_latex_exp.replace("G(s) = ", "T(s) = ")

    return {
        "closed_loop_num": cl_num.tolist(),
        "closed_loop_den": cl_den.tolist(),
        "closed_loop_latex": cl_latex,
        "closed_loop_poles": serialize_complex(cl_poles),
        "stable": stable,
        "wn": finite_float(wn_uncomp),
        "xi": finite_float(xi_uncomp),
        "mp": finite_float(mp_uncomp),
        "ts_2pct": finite_float(ts2_uncomp),
        "ts_5pct": finite_float(ts5_uncomp),
        "steady_state_error": err_info["json_value"],
        "steady_state_error_raw": err_info["value"],
        "steady_state_error_latex": err_info["latex_value"],
    }


def _factor_magnitude_details(s_dom: complex, root: complex, multiplicity: int = 1, label: str = "") -> dict[str, Any]:
    """Calcula o módulo |(s - root)|^multiplicity = (sqrt(Re_i^2 + Im_i^2))^m."""
    diff = s_dom - root
    re_i = float(diff.real)
    im_i = float(diff.imag)
    single_mag = float(math.hypot(re_i, im_i))
    total_mag = float(single_mag ** multiplicity)
    root_pos = float(-root.real)
    if abs(root.imag) < 1e-7:
        if abs(root_pos) < 1e-9:
            factor_tex = "s" if multiplicity == 1 else f"s^{multiplicity}"
        else:
            sign_str = "+" if root_pos >= 0 else "-"
            factor_tex = f"(s {sign_str} {_fmt_tex_num(abs(root_pos))})"
            if multiplicity > 1:
                factor_tex += f"^{multiplicity}"
    else:
        factor_tex = f"(s - ({_fmt_tex_num(root.real)}{_fmt_tex_num(root.imag):+}j))"

    formula = (
        rf"\left|{factor_tex}\right| = \sqrt{{({_fmt_tex_num(re_i, 4)})^2 + ({_fmt_tex_num(im_i, 4)})^2}}"
        + (rf"^{{{multiplicity}}}" if multiplicity > 1 else "")
        + rf" = {_fmt_tex_num(total_mag, 4)}"
    )
    return {
        "label": label or factor_tex,
        "factor_tex": factor_tex,
        "re_i": re_i,
        "im_i": im_i,
        "single_magnitude": single_mag,
        "multiplicity": multiplicity,
        "magnitude": total_mag,
        "formula_latex": formula,
    }


def _build_controller_latex(controller_kind: str, params: dict[str, Any]) -> str:
    kind = controller_kind.upper()
    if kind == "LEAD":
        k_s = _fmt_tex_num(params["K"], 4)
        z_s = _fmt_tex_num(params["z"], 4)
        p_s = _fmt_tex_num(params["p"], 4)
        return rf"G_c(s) = \frac{{{k_s}(s + {z_s})}}{{s + {p_s}}}"
    if kind == "LAG":
        k_val = params["K"]
        k_s = "" if abs(k_val - 1.0) < 1e-7 else _fmt_tex_num(k_val, 4)
        z_s = _fmt_tex_num(params["z"], 4)
        p_s = _fmt_tex_num(params["p"], 4)
        return rf"G_c(s) = \frac{{{k_s}(s + {z_s})}}{{s + {p_s}}}"
    if kind == "P":
        kp_s = _fmt_tex_num(params["kp"], 4)
        return rf"G_c(s) = K_p = {kp_s}"
    if kind == "PD":
        kp_s = _fmt_tex_num(params["kp"], 4)
        td_s = _fmt_tex_num(params["td"], 4)
        z_s = _fmt_tex_num(params["z"], 4)
        return rf"G_c(s) = {kp_s}(1 + {td_s}s) = {kp_s}\cdot {td_s}(s + {z_s})"
    if kind == "PI":
        kp_s = _fmt_tex_num(params["kp"], 4)
        z_s = _fmt_tex_num(params["z"], 4)
        return rf"G_c(s) = \frac{{{kp_s}(s + {z_s})}}{{s}}"
    return "G_c(s) = 1"


def _design_lgr_series_controller(
    plant_expression: str,
    *,
    controller_kind: str = "lead",
    input_type: str = "step",
    settling_criterion: str | int = "2",
    ts: Any = None,
    mp: Any = None,
    xi: Any = None,
    wn: Any = None,
    primary_damping_spec: str | None = None,
    primary_freq_spec: str | None = None,
    error_reduction_factor: Any = None,
    target_steady_state_error: Any = None,
    lead_zero: Any = None,
    lag_pole: Any = None,
    lag_gain: Any = None,
    lgr_branch_re: Any = None,
    final_time: Any = None,
    points: int = 900,
    theme: str = "dark",
) -> dict[str, Any]:
    """
    Executa o projeto analítico completo de controladores em série G_ma(s) = G_c(s)G(s)
    para os 5 tipos: Avanço de Fase (lead), Atraso de Fase (lag), P, PD e PI.
    """
    plant, num, den, plant_latex_expanded, plant_latex_factored = parse_siso_transfer_function(plant_expression)
    kind = controller_kind.upper()
    if kind in {"AVANCO", "AVANÇO", "LEAD"}:
        kind = "LEAD"
    elif kind in {"ATRASO", "LAG"}:
        kind = "LAG"

    input_mode = _normalize_input_type(input_type)
    input_labels = {
        "step": "Degrau unitário R(s) = 1/s",
        "ramp": "Rampa unitária R(s) = 1/s²",
        "parabola": "Parábola unitária R(s) = 1/s³",
    }
    input_label = input_labels[input_mode]

    # Extração de ganho líder, polos e zeros da planta G(s)
    lead_gain = float(num[0] / den[0])
    plant_poles_raw = np.roots(den) if len(den) > 1 else np.array([])
    plant_zeros_raw = np.roots(num) if len(num) > 1 else np.array([])
    grouped_poles = _group_roots(plant_poles_raw)
    grouped_zeros = _group_roots(plant_zeros_raw)

    # Passo 1: Análise sem controlador
    uncomp = analyze_uncompensated_system(num, den, input_type=input_mode)
    steps: list[dict[str, Any]] = []

    step1_eqs = [
        plant_latex_factored,
        uncomp["closed_loop_latex"],
    ]
    if uncomp["wn"] is not None and uncomp["xi"] is not None:
        step1_eqs.append(
            rf"s^2 + 2\xi\omega_n s + \omega_n^2 \implies \omega_{{n,\text{{atual}}}} = {_fmt_tex_num(uncomp['wn'], 4)}\text{{ rad/s}}, \quad \xi_{{\text{{atual}}}} = {_fmt_tex_num(uncomp['xi'], 4)}"
        )
    step1_eqs.append(
        rf"e_{{\text{{atual}}}}(\infty) = \lim_{{s \to 0}} s\left(\frac{{1}}{{1 + G(s)}}\right)R(s) = {uncomp['steady_state_error_latex']}"
    )
    poles_desc = ", ".join(
        f"s = {_fmt_num(g['root'].real)}" + (f" (×{g['multiplicity']})" if g["multiplicity"] > 1 else "")
        if g["is_real"]
        else f"s = {_fmt_num(g['root'].real)}{g['root'].imag:+.3g}j"
        for g in grouped_poles
    ) or "nenhum"
    zeros_desc = ", ".join(
        f"s = {_fmt_num(g['root'].real)}" + (f" (×{g['multiplicity']})" if g["multiplicity"] > 1 else "")
        if g["is_real"]
        else f"s = {_fmt_num(g['root'].real)}{g['root'].imag:+.3g}j"
        for g in grouped_zeros
    ) or "nenhum"

    steps.append({
        "number": 1,
        "title": "Análise da Planta e Sistema Sem Controlador",
        "description": f"Análise em malha fechada com realimentação unitária e entrada {input_label.lower()}.",
        "equations": step1_eqs,
        "notes": [
            f"Polos de G(s): {poles_desc}",
            f"Zeros de G(s): {zeros_desc}",
        ],
    })

    angle_contributions: list[dict[str, Any]] = []
    controller_params: dict[str, Any] = {"controller_type": kind}
    design_label = ""

    # ==========================================================================
    # CASO 2: ATRASO DE FASE (Gc(s) = K(s+z)/(s+p), p < z)
    # ==========================================================================
    if kind == "LAG":
        design_label = "Controlador por Atraso de Fase — Gc(s) = K(s+z)/(s+p)"
        k_val = _positive(lag_gain if lag_gain is not None and lag_gain != "" else 1.0, "O ganho K")
        p_val = _positive(lag_pole if lag_pole is not None and lag_pole != "" else 0.01, "O polo p do controlador")

        e_atual = uncomp["steady_state_error_raw"]
        if not math.isfinite(e_atual) or e_atual <= 0:
            # Caso a planta tenha erro nulo para a entrada atual, testa rampa e parábola
            for fallback_mode in ("ramp", "parabola"):
                if fallback_mode == input_mode:
                    continue
                err_fb = evaluate_steady_state_error(num, den, input_type=fallback_mode)
                if math.isfinite(err_fb["value"]) and err_fb["value"] > 0:
                    e_atual = err_fb["value"]
                    input_mode = fallback_mode
                    break
            else:
                raise ValueError("O erro em regime permanente atual é zero ou infinito para a entrada selecionada; verifique a planta ou o tipo de entrada R(s).")

        factor_in = _to_optional_float(error_reduction_factor)
        target_in = _to_optional_float(target_steady_state_error)

        if target_in is not None and target_in > 0 and (factor_in is None or factor_in <= 0):
            e_novo = target_in
            reduction_factor = e_atual / e_novo
        else:
            reduction_factor = factor_in if (factor_in is not None and factor_in > 0) else 10.0
            e_novo = e_atual / reduction_factor

        if e_novo <= 0 or e_novo >= e_atual:
            raise ValueError("O novo erro em regime permanente e_novo(∞) deve ser positivo e menor que o erro atual.")

        # Resolve simbolicamente lim_{s->0} s * (1 / (1 + K * (z/p) * G(s))) * R(s) = e_novo para achar r = z/p
        s_sym = sp.Symbol("s", positive=True)
        r_sym = sp.Symbol("r", positive=True)
        n_expr = sum(sp.Float(c) * s_sym ** (len(num) - 1 - i) for i, c in enumerate(num))
        d_expr = sum(sp.Float(c) * s_sym ** (len(den) - 1 - i) for i, c in enumerate(den))
        g_expr = n_expr / d_expr
        if input_mode == "parabola":
            r_input = 1 / (s_sym ** 3)
        elif input_mode == "ramp":
            r_input = 1 / (s_sym ** 2)
        else:
            r_input = 1 / s_sym
        comp_err_limit = sp.limit(s_sym * (1 / (1 + sp.Float(k_val) * r_sym * g_expr)) * r_input, s_sym, 0, dir="+")
        solutions = sp.solve(sp.Eq(comp_err_limit, sp.Float(e_novo)), r_sym)
        real_sols = [float(sol.evalf()) for sol in solutions if sol.is_real and float(sol.evalf()) > 0]
        if not real_sols:
            raise ValueError("Não foi possível encontrar uma razão z/p positiva para a meta de erro informada.")
        z_over_p = float(real_sols[0])
        z_val = z_over_p * p_val

        if p_val >= z_val:
            raise ValueError("No compensador por Atraso de Fase, o polo p deve ser menor que o zero z (p < z).")

        controller_tf = ct.tf([k_val, k_val * z_val], [1.0, p_val])
        comp_loop_tf = controller_tf * plant
        comp_cl_tf = ct.feedback(comp_loop_tf, 1)
        comp_poles = ct.poles(comp_cl_tf)
        complex_cl = [cp for cp in comp_poles if abs(cp.imag) > 1e-5 and cp.real < 0]
        if complex_cl:
            s_dom = max(complex_cl, key=lambda cp: cp.real)
            re_dom, im_dom = float(-s_dom.real), float(abs(s_dom.imag))
            wn_dom = abs(s_dom)
            xi_dom = re_dom / wn_dom
        else:
            re_dom, im_dom = 1.0, 1.0
            wn_dom = math.hypot(re_dom, im_dom)
            xi_dom = re_dom / wn_dom
            s_dom = complex(-re_dom, im_dom)

        controller_params.update({
            "K": float(k_val),
            "kp": float(k_val),
            "z": float(z_val),
            "p": float(p_val),
            "z_over_p": float(z_over_p),
            "e_atual": float(e_atual),
            "e_novo": float(e_novo),
            "reduction_factor": float(reduction_factor),
            "re": float(re_dom),
            "im": float(im_dom),
            "xi": float(xi_dom),
            "wn": float(wn_dom),
            "ts": float(4.0 / re_dom) if re_dom > 0 else None,
            "mp": float(mp_from_xi(xi_dom)),
            "criterion_pct": 2 if str(settling_criterion).strip().replace("%", "") != "5" else 5,
        })

        steps.append({
            "number": 2,
            "title": "Especificação de Regime Permanente e Razão z/p",
            "description": "Redução do erro em regime permanente mantendo K para preservar a resposta transitória.",
            "equations": [
                rf"e_{{\text{{atual}}}}(\infty) = {_fmt_tex_num(e_atual, 4)}, \quad \text{{Fator de redução}} = {_fmt_tex_num(reduction_factor, 4)} \implies e_{{\text{{novo}}}}(\infty) = \frac{{{_fmt_tex_num(e_atual, 4)}}}{{{_fmt_tex_num(reduction_factor, 4)}}} = {_fmt_tex_num(e_novo, 4)}",
                rf"\lim_{{s \to 0}} s\left(\frac{{1}}{{1 + G_c(s)G(s)}}\right)R(s) = e_{{\text{{novo}}}}(\infty) \implies \frac{{1}}{{1 + K\frac{{z}}{{p}}G(0)}} = {_fmt_tex_num(e_novo, 4)}" if input_mode == "step" else rf"\lim_{{s \to 0}} s\left(\frac{{1}}{{1 + G_c(s)G(s)}}\right)R(s) = e_{{\text{{novo}}}}(\infty)",
                rf"\frac{{z}}{{p}} = {_fmt_tex_num(z_over_p, 4)}",
            ],
            "notes": [
                f"Ganho adotado: K = {_fmt_num(k_val)}",
                f"Meta de erro em regime permanente: e(∞) = {_fmt_num(e_novo)}",
            ],
        })

        steps.append({
            "number": 3,
            "title": "Alocação do Polo e Zero do Controlador por Atraso de Fase",
            "description": "Aloca-se o polo próximo à origem (p < z) para não alterar significativamente os ângulos do LGR.",
            "equations": [
                rf"p = {_fmt_tex_num(p_val, 4)}",
                rf"z = \left(\frac{{z}}{{p}}\right) \times p = {_fmt_tex_num(z_over_p, 4)} \times {_fmt_tex_num(p_val, 4)} = {_fmt_tex_num(z_val, 4)}",
                _build_controller_latex("LAG", controller_params),
            ],
            "notes": [
                f"Polo do controlador em s = -{_fmt_num(p_val)} e zero em s = -{_fmt_num(z_val)} (p < z).",
            ],
        })

    # ==========================================================================
    # CASOS COM ESPECIFICAÇÃO TRANSITÓRIA NO LGR: AVANÇO, P, PD, PI
    # ==========================================================================
    else:
        specs = resolve_transient_specifications(
            ts=ts,
            mp=mp,
            xi=xi,
            wn=wn,
            settling_criterion=settling_criterion,
            primary_damping_spec=primary_damping_spec,
            primary_freq_spec=primary_freq_spec,
        )
        re_dom = specs["re"]
        im_dom = specs["im"]
        xi_val = specs["xi"]
        wn_val = specs["wn"]
        ts_val = specs["ts"]
        mp_val = specs["mp"]
        crit_pct = specs["criterion_pct"]
        settling_factor = specs["settling_factor"]
        s_dom = specs["dominant_pole"]

        step2_eqs = [
            rf"M_p(\%) = \left[e^{{-(\xi \cdot \pi / \sqrt{{1 - \xi^2}})}}\right] \times 100 = {_fmt_tex_num(mp_val, 4)}\% \iff \xi = {_fmt_tex_num(xi_val, 4)}",
            rf"t_s({crit_pct}\%) = \frac{{{int(settling_factor)}}}{{\xi\omega_n}} = {_fmt_tex_num(ts_val, 4)}\text{{ s}} \implies \xi\omega_n = {_fmt_tex_num(re_dom, 4)} \implies \omega_n = \frac{{{_fmt_tex_num(re_dom, 4)}}}{{{_fmt_tex_num(xi_val, 4)}}} = {_fmt_tex_num(wn_val, 4)}\text{{ rad/s}}",
            rf"s = -\xi\omega_n \pm \omega_n\sqrt{{1 - \xi^2}}j = -{_fmt_tex_num(re_dom, 4)} \pm {_fmt_tex_num(im_dom, 4)}j",
        ]
        steps.append({
            "number": 2,
            "title": "Especificações Transitórias e Polos Dominantes Desejados",
            "description": f"Cálculo de ξ, ωn e dos polos dominantes s = -Re ± Im·j pelo critério de {crit_pct}%.",
            "equations": step2_eqs,
            "notes": [
                f"Parte real desejada: Re = ξ·ωn = {_fmt_num(re_dom)}",
                f"Parte imaginária desejada: Im = ωn·√(1 - ξ²) = {_fmt_num(im_dom)}",
            ],
        })

        # ----------------------------------------------------------------------
        # CASO 1: AVANÇO DE FASE (Gc(s) = K(s+z)/(s+p), z < p)
        # ----------------------------------------------------------------------
        if kind == "LEAD":
            design_label = "Controlador por Avanço de Fase — Gc(s) = K(s+z)/(s+p)"
            custom_z = _to_optional_float(lead_zero)
            z_val = custom_z if (custom_z is not None and custom_z > 0) else re_dom
            z_angle_info = calculate_lgr_angle(z_val, re_dom, im_dom)
            theta_z = z_angle_info["angle_deg"]

            angle_eqs = [
                rf"\text{{Zero do controlador alocado em }} z = {_fmt_tex_num(z_val, 4)} \implies \theta_z = {_fmt_tex_num(theta_z, 4)}^\circ"
            ]
            angle_contributions.append({
                "type": "controller_zero",
                "label": f"Zero Gc (s = -{_fmt_num(z_val)})",
                "root": complex(-z_val, 0.0),
                "multiplicity": 1,
                "angle_deg": theta_z,
                "total_angle_deg": theta_z,
            })

            sum_zeros_g = 0.0
            for idx, gz in enumerate(grouped_zeros, start=1):
                ang_info = calculate_complex_root_angle(gz["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gz["multiplicity"]
                sum_zeros_g += tot_ang
                mult_str = f"{gz['multiplicity']} \\times " if gz["multiplicity"] > 1 else ""
                angle_eqs.append(rf"\text{{Zero }} z_{idx}: {ang_info['formula_latex']}" + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gz["multiplicity"] > 1 else ""))
                angle_contributions.append({
                    "type": "plant_zero",
                    "label": f"Zero G (s = {_fmt_num(gz['root'].real)})",
                    "root": gz["root"],
                    "multiplicity": gz["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            sum_poles_g = 0.0
            for idx, gp in enumerate(grouped_poles, start=1):
                ang_info = calculate_complex_root_angle(gp["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gp["multiplicity"]
                sum_poles_g += tot_ang
                mult_str = f"{gp['multiplicity']} \\times " if gp["multiplicity"] > 1 else ""
                angle_eqs.append(rf"\text{{Polo }} p_{idx}: {ang_info['formula_latex']}" + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gp["multiplicity"] > 1 else ""))
                angle_contributions.append({
                    "type": "plant_pole",
                    "label": f"Polo G (s = {_fmt_num(gp['root'].real)})",
                    "root": gp["root"],
                    "multiplicity": gp["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            raw_theta_p = theta_z + sum_zeros_g - sum_poles_g + 180.0
            theta_p = normalize_phase_angle(raw_theta_p)
            pos_p_info = calculate_position_from_angle(theta_p, re_dom, im_dom, symbol="p")
            p_val = pos_p_info["pos"]

            if p_val <= 0:
                raise ValueError(f"A condição de fase resultou em um polo p = {p_val:.4g} fora do semiplano esquerdo. Ajuste as especificações ou a posição do zero z.")

            angle_contributions.append({
                "type": "controller_pole",
                "label": f"Polo Gc (s = -{_fmt_num(p_val)})",
                "root": complex(-p_val, 0.0),
                "multiplicity": 1,
                "angle_deg": theta_p,
                "total_angle_deg": theta_p,
            })

            angle_eqs.extend([
                rf"\theta_z + \sum \theta_{{\text{{zeros de }} G}} - \sum \theta_{{\text{{polos de }} G}} - \theta_p = -180^\circ",
                rf"{_fmt_tex_num(theta_z, 4)}^\circ + {_fmt_tex_num(sum_zeros_g, 4)}^\circ - {_fmt_tex_num(sum_poles_g, 4)}^\circ - \theta_p = -180^\circ \implies \theta_p = {_fmt_tex_num(theta_p, 4)}^\circ",
                pos_p_info["formula_latex"],
            ])

            steps.append({
                "number": 3,
                "title": "Condição de Fase no LGR e Posição do Polo p",
                "description": "Cálculo dos ângulos θi no polo dominante e determinação do polo p do compensador por avanço de fase.",
                "equations": angle_eqs,
                "notes": [
                    f"Zero do controlador: z = {_fmt_num(z_val)} (θz = {_fmt_num(theta_z)}°)",
                    f"Polo do controlador: p = {_fmt_num(p_val)} (θp = {_fmt_num(theta_p)}°)",
                ],
            })

            # Passo 4: Condição de Módulo
            mag_num_factors = [_factor_magnitude_details(s_dom, complex(-z_val, 0.0), 1, f"(s + {_fmt_num(z_val)})")]
            for gz in grouped_zeros:
                mag_num_factors.append(_factor_magnitude_details(s_dom, gz["root"], gz["multiplicity"]))
            mag_den_factors = [_factor_magnitude_details(s_dom, complex(-p_val, 0.0), 1, f"(s + {_fmt_num(p_val)})")]
            for gp in grouped_poles:
                mag_den_factors.append(_factor_magnitude_details(s_dom, gp["root"], gp["multiplicity"]))

            num_mag_prod = abs(lead_gain) * math.prod(f["magnitude"] for f in mag_num_factors)
            den_mag_prod = math.prod(f["magnitude"] for f in mag_den_factors)
            k_val = float(den_mag_prod / num_mag_prod)

            mag_eqs = [f["formula_latex"] for f in mag_num_factors + mag_den_factors]
            mag_eqs.append(
                rf"\left|G_{{ma}}(s)\right| = \left|\frac{{K(s + {_fmt_tex_num(z_val, 4)})}}{{s + {_fmt_tex_num(p_val, 4)}}} G(s)\right| = 1 \implies K = \frac{{{_fmt_tex_num(den_mag_prod, 4)}}}{{{_fmt_tex_num(num_mag_prod, 4)}}} = {_fmt_tex_num(k_val, 4)}"
            )
            steps.append({
                "number": 4,
                "title": "Condição de Módulo e Cálculo do Ganho K",
                "description": "Aplicação de |Gma(s)| = |Gc(s)G(s)| = 1 no polo dominante s = -Re + Im·j.",
                "equations": mag_eqs,
                "notes": [
                    f"Ganho calculado: K = {_fmt_num(k_val)}",
                ],
            })

            controller_tf = ct.tf([k_val, k_val * z_val], [1.0, p_val])
            controller_params.update({
                "K": k_val,
                "kp": k_val,
                "z": float(z_val),
                "p": float(p_val),
                "theta_z": float(theta_z),
                "theta_p": float(theta_p),
                "re": float(re_dom),
                "im": float(im_dom),
                "xi": float(xi_val),
                "wn": float(wn_val),
                "ts": float(ts_val),
                "mp": float(mp_val),
                "criterion_pct": crit_pct,
            })

        # ----------------------------------------------------------------------
        # CASO 3: PROPORCIONAL — P (Gc(s) = Kp)
        # ----------------------------------------------------------------------
        elif kind == "P":
            design_label = "Controlador Proporcional (P) — Gc(s) = Kp"
            re_theoretical = re_dom
            im_theoretical = im_dom

            # Verifica se há ramo vertical do LGR numa abscissa -Re_LGR != -Re
            branch_re_in = _to_optional_float(lgr_branch_re)
            detected_branch_re = None
            if branch_re_in is not None and branch_re_in > 0:
                detected_branch_re = abs(branch_re_in)
            elif len(plant_poles_raw) == 2 and len(plant_zeros_raw) == 0 and all(abs(p.imag) < 1e-6 for p in plant_poles_raw):
                detected_branch_re = float(-np.mean(np.real(plant_poles_raw)))

            step3_eqs = []
            step3_notes = []
            if detected_branch_re is not None and abs(detected_branch_re - re_dom) > 1e-5 and detected_branch_re > 0:
                theta_rad = math.atan(im_theoretical / re_theoretical)
                theta_deg = math.degrees(theta_rad)
                tan_theta = math.tan(theta_rad)
                re_dom = detected_branch_re
                im_dom = re_dom * tan_theta
                s_dom = complex(-re_dom, im_dom)
                step3_eqs.extend([
                    rf"\text{{Ramo vertical do LGR em }} -\text{{Re}}_{{\text{{LGR}}}} = -{_fmt_tex_num(re_dom, 4)} \neq -{_fmt_tex_num(re_theoretical, 4)}",
                    rf"\theta = \tan^{{-1}}\!\left(\frac{{\text{{Im}}}}{{\text{{Re}}}}\right) = \tan^{{-1}}\!\left(\frac{{{_fmt_tex_num(im_theoretical, 4)}}}{{{_fmt_tex_num(re_theoretical, 4)}}}\right) = {_fmt_tex_num(theta_deg, 4)}^\circ \quad (\tan\theta = {_fmt_tex_num(tan_theta, 4)})",
                    rf"\omega_d = \text{{Re}}_{{\text{{LGR}}}} \times \tan\theta = {_fmt_tex_num(re_dom, 4)} \times {_fmt_tex_num(tan_theta, 4)} = {_fmt_tex_num(im_dom, 4)}",
                    rf"s = -\text{{Re}}_{{\text{{LGR}}}} \pm \omega_d j = -{_fmt_tex_num(re_dom, 4)} \pm {_fmt_tex_num(im_dom, 4)}j",
                ])
                step3_notes.append(f"Polo dominante ajustado para o ramo vertical do LGR em s = -{_fmt_num(re_dom)} ± {_fmt_num(im_dom)}j.")
            else:
                step3_eqs.append(
                    rf"s = -{_fmt_tex_num(re_dom, 4)} \pm {_fmt_tex_num(im_dom, 4)}j"
                )
                step3_notes.append(f"Polo dominante utilizado diretamente: s = -{_fmt_num(re_dom)} ± {_fmt_num(im_dom)}j.")

            for idx, gp in enumerate(grouped_poles, start=1):
                ang_info = calculate_complex_root_angle(gp["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gp["multiplicity"]
                angle_contributions.append({
                    "type": "plant_pole",
                    "label": f"Polo G (s = {_fmt_num(gp['root'].real)})",
                    "root": gp["root"],
                    "multiplicity": gp["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            steps.append({
                "number": 3,
                "title": "Localização do Polo Dominante sobre o LGR",
                "description": "Verificação da geometria do LGR da planta para alocação por ganho proporcional Kp.",
                "equations": step3_eqs,
                "notes": step3_notes,
            })

            mag_num_factors = [_factor_magnitude_details(s_dom, gz["root"], gz["multiplicity"]) for gz in grouped_zeros]
            mag_den_factors = [_factor_magnitude_details(s_dom, gp["root"], gp["multiplicity"]) for gp in grouped_poles]
            num_mag_prod = abs(lead_gain) * (math.prod(f["magnitude"] for f in mag_num_factors) if mag_num_factors else 1.0)
            den_mag_prod = math.prod(f["magnitude"] for f in mag_den_factors) if mag_den_factors else 1.0
            kp_val = float(den_mag_prod / num_mag_prod)

            mag_eqs = [f["formula_latex"] for f in mag_num_factors + mag_den_factors]
            mag_eqs.append(
                rf"1 + K_p G(s) = 0 \implies K_p = \left|\frac{{1}}{{G(s)}}\right| = \frac{{{_fmt_tex_num(den_mag_prod, 4)}}}{{{_fmt_tex_num(num_mag_prod, 4)}}} = {_fmt_tex_num(kp_val, 4)}"
            )
            steps.append({
                "number": 4,
                "title": "Condição de Módulo e Cálculo de Kp",
                "description": "Isolamento do ganho proporcional Kp a partir da equação característica 1 + Kp·G(s) = 0.",
                "equations": mag_eqs,
                "notes": [
                    f"Ganho proporcional: Kp = {_fmt_num(kp_val)}",
                ],
            })

            controller_tf = ct.tf([kp_val], [1.0])
            controller_params.update({
                "K": kp_val,
                "kp": kp_val,
                "re_theoretical": float(re_theoretical),
                "im_theoretical": float(im_theoretical),
                "re": float(re_dom),
                "im": float(im_dom),
                "lgr_branch_re": detected_branch_re,
                "xi": float(xi_val),
                "wn": float(wn_val),
                "ts": float(ts_val),
                "mp": float(mp_val),
                "criterion_pct": crit_pct,
            })

        # ----------------------------------------------------------------------
        # CASO 4: PROPORCIONAL E DERIVATIVO — PD (Gc(s) = Kp(1 + td*s), z = 1/td)
        # ----------------------------------------------------------------------
        elif kind == "PD":
            design_label = "Controlador Proporcional-Derivativo (PD) — Gc(s) = Kp(1 + τd·s)"
            angle_eqs = []
            sum_poles_g = 0.0
            for idx, gp in enumerate(grouped_poles, start=1):
                ang_info = calculate_complex_root_angle(gp["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gp["multiplicity"]
                sum_poles_g += tot_ang
                mult_str = f"{gp['multiplicity']} \\times " if gp["multiplicity"] > 1 else ""
                angle_eqs.append(rf"\text{{Polo }} p_{idx}: {ang_info['formula_latex']}" + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gp["multiplicity"] > 1 else ""))
                angle_contributions.append({
                    "type": "plant_pole",
                    "label": f"Polo G (s = {_fmt_num(gp['root'].real)})",
                    "root": gp["root"],
                    "multiplicity": gp["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            sum_zeros_g = 0.0
            for idx, gz in enumerate(grouped_zeros, start=1):
                ang_info = calculate_complex_root_angle(gz["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gz["multiplicity"]
                sum_zeros_g += tot_ang
                mult_str = f"{gz['multiplicity']} \\times " if gz["multiplicity"] > 1 else ""
                angle_eqs.append(rf"\text{{Zero }} z_{idx}: {ang_info['formula_latex']}" + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gz["multiplicity"] > 1 else ""))
                angle_contributions.append({
                    "type": "plant_zero",
                    "label": f"Zero G (s = {_fmt_num(gz['root'].real)})",
                    "root": gz["root"],
                    "multiplicity": gz["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            raw_theta_z = sum_poles_g - sum_zeros_g + 180.0
            theta_z = normalize_phase_angle(raw_theta_z)
            pos_z_info = calculate_position_from_angle(theta_z, re_dom, im_dom, symbol="z")
            z_val = pos_z_info["pos"]
            if z_val <= 0:
                raise ValueError(f"A condição de fase resultou em z = {z_val:.4g} <= 0. Verifique as especificações transitórias.")
            td_val = 1.0 / z_val

            angle_contributions.append({
                "type": "controller_zero",
                "label": f"Zero PD (s = -{_fmt_num(z_val)})",
                "root": complex(-z_val, 0.0),
                "multiplicity": 1,
                "angle_deg": theta_z,
                "total_angle_deg": theta_z,
            })

            angle_eqs.extend([
                rf"\theta_z = \sum \theta_{{\text{{polos de }} G}} - \sum \theta_{{\text{{zeros de }} G}} + 180^\circ \pmod{{360^\circ}} = {_fmt_tex_num(sum_poles_g, 4)}^\circ - {_fmt_tex_num(sum_zeros_g, 4)}^\circ - 180^\circ = {_fmt_tex_num(theta_z, 4)}^\circ",
                pos_z_info["formula_latex"],
                rf"\tau_d = \frac{{1}}{{z}} = \frac{{1}}{{{_fmt_tex_num(z_val, 4)}}} = {_fmt_tex_num(td_val, 4)}\text{{ s}}",
            ])

            steps.append({
                "number": 3,
                "title": "Condição de Fase no LGR, Zero z e Tempo Derivativo τd",
                "description": "Cálculo de θz pela condição de fase e determinação de z e τd = 1/z.",
                "equations": angle_eqs,
                "notes": [
                    f"Ângulo exigido do zero: θz = {_fmt_num(theta_z)}°",
                    f"Zero do controlador: z = {_fmt_num(z_val)} → τd = {_fmt_num(td_val)} s",
                ],
            })

            # Condição de Módulo: |Kp * (1 + td * s) * G(s)| = 1
            pd_factor = 1.0 + td_val * s_dom
            pd_mag = float(math.hypot(pd_factor.real, pd_factor.imag))
            pd_factor_eq = (
                rf"\left|1 + \tau_d s\right| = \left|1 + {_fmt_tex_num(td_val, 4)}(-{_fmt_tex_num(re_dom, 4)} + {_fmt_tex_num(im_dom, 4)}j)\right| "
                rf"= \sqrt{{({_fmt_tex_num(pd_factor.real, 4)})^2 + ({_fmt_tex_num(pd_factor.imag, 4)})^2}} = {_fmt_tex_num(pd_mag, 4)}"
            )
            mag_num_factors = [_factor_magnitude_details(s_dom, gz["root"], gz["multiplicity"]) for gz in grouped_zeros]
            mag_den_factors = [_factor_magnitude_details(s_dom, gp["root"], gp["multiplicity"]) for gp in grouped_poles]

            num_mag_prod = abs(lead_gain) * pd_mag * (math.prod(f["magnitude"] for f in mag_num_factors) if mag_num_factors else 1.0)
            den_mag_prod = math.prod(f["magnitude"] for f in mag_den_factors) if mag_den_factors else 1.0
            kp_val = float(den_mag_prod / num_mag_prod)
            kd_val = float(kp_val * td_val)

            mag_eqs = [pd_factor_eq] + [f["formula_latex"] for f in mag_num_factors + mag_den_factors]
            mag_eqs.append(
                rf"\left|G_{{ma}}(s)\right| = \left|K_p(1 + \tau_d s)G(s)\right| = 1 \implies K_p = \frac{{{_fmt_tex_num(den_mag_prod, 4)}}}{{{_fmt_tex_num(num_mag_prod, 4)}}} = {_fmt_tex_num(kp_val, 4)}"
            )
            steps.append({
                "number": 4,
                "title": "Condição de Módulo e Cálculo de Kp",
                "description": "Aplicação de |Gma(s)| = |Kp(1 + τd·s)G(s)| = 1 no polo dominante.",
                "equations": mag_eqs,
                "notes": [
                    f"Ganho proporcional: Kp = {_fmt_num(kp_val)}",
                    f"Constante derivativa: τd = {_fmt_num(td_val)} s (Kd = Kp·τd = {_fmt_num(kd_val)})",
                ],
            })

            controller_tf = ct.tf([kp_val * td_val, kp_val], [1.0])
            controller_params.update({
                "K": kp_val,
                "kp": kp_val,
                "kd": kd_val,
                "td": float(td_val),
                "z": float(z_val),
                "theta_z": float(theta_z),
                "re": float(re_dom),
                "im": float(im_dom),
                "xi": float(xi_val),
                "wn": float(wn_val),
                "ts": float(ts_val),
                "mp": float(mp_val),
                "criterion_pct": crit_pct,
            })

        # ----------------------------------------------------------------------
        # CASO 5: PROPORCIONAL-INTEGRAL — PI (Gc(s) = Kp(s + z)/s, z = 1/ti)
        # ----------------------------------------------------------------------
        elif kind == "PI":
            design_label = "Controlador Proporcional-Integral (PI) — Gc(s) = Kp(s + z)/s"
            # Inclui o polo na origem (s = 0) do controlador PI na lista de polos
            pi_poles_raw = list(plant_poles_raw) + [0.0]
            grouped_total_poles = _group_roots(pi_poles_raw)

            angle_eqs = []
            sum_poles_total = 0.0
            for idx, gp in enumerate(grouped_total_poles, start=1):
                ang_info = calculate_complex_root_angle(gp["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gp["multiplicity"]
                sum_poles_total += tot_ang
                is_origin = abs(gp["root"]) < 1e-7
                origin_note = " (inclui polo do PI em s = 0)" if is_origin else ""
                mult_str = f"{gp['multiplicity']} \\times " if gp["multiplicity"] > 1 else ""
                angle_eqs.append(
                    rf"\text{{Polo }} p_{idx}{origin_note}: {ang_info['formula_latex']}"
                    + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gp["multiplicity"] > 1 else "")
                )
                angle_contributions.append({
                    "type": "controller_pole" if (is_origin and gp["multiplicity"] == 1) else "plant_pole",
                    "label": f"Polo s = {_fmt_num(gp['root'].real)}" + (f" (×{gp['multiplicity']})" if gp["multiplicity"] > 1 else ""),
                    "root": gp["root"],
                    "multiplicity": gp["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            sum_zeros_g = 0.0
            for idx, gz in enumerate(grouped_zeros, start=1):
                ang_info = calculate_complex_root_angle(gz["root"], re_dom, im_dom)
                tot_ang = ang_info["angle_deg"] * gz["multiplicity"]
                sum_zeros_g += tot_ang
                mult_str = f"{gz['multiplicity']} \\times " if gz["multiplicity"] > 1 else ""
                angle_eqs.append(rf"\text{{Zero }} z_{idx}: {ang_info['formula_latex']}" + (rf" \implies {mult_str}{_fmt_tex_num(ang_info['angle_deg'], 4)}^\circ = {_fmt_tex_num(tot_ang, 4)}^\circ" if gz["multiplicity"] > 1 else ""))
                angle_contributions.append({
                    "type": "plant_zero",
                    "label": f"Zero G (s = {_fmt_num(gz['root'].real)})",
                    "root": gz["root"],
                    "multiplicity": gz["multiplicity"],
                    "angle_deg": ang_info["angle_deg"],
                    "total_angle_deg": tot_ang,
                })

            raw_theta_z = sum_poles_total - sum_zeros_g + 180.0
            theta_z = normalize_phase_angle(raw_theta_z)
            pos_z_info = calculate_position_from_angle(theta_z, re_dom, im_dom, symbol="z")
            z_val = pos_z_info["pos"]
            if z_val <= 0:
                raise ValueError(f"A condição de fase resultou em z = {z_val:.4g} <= 0. Ajuste as especificações transitórias.")
            ti_val = 1.0 / z_val

            angle_contributions.append({
                "type": "controller_zero",
                "label": f"Zero PI (s = -{_fmt_num(z_val)})",
                "root": complex(-z_val, 0.0),
                "multiplicity": 1,
                "angle_deg": theta_z,
                "total_angle_deg": theta_z,
            })

            angle_eqs.extend([
                rf"\theta_z = \sum \theta_{{\text{{polos totais}}}} - \sum \theta_{{\text{{zeros de }} G}} + 180^\circ \pmod{{360^\circ}} = {_fmt_tex_num(sum_poles_total, 4)}^\circ - {_fmt_tex_num(sum_zeros_g, 4)}^\circ - 180^\circ = {_fmt_tex_num(theta_z, 4)}^\circ",
                pos_z_info["formula_latex"],
                rf"\tau_i = \frac{{1}}{{z}} = \frac{{1}}{{{_fmt_tex_num(z_val, 4)}}} = {_fmt_tex_num(ti_val, 4)}\text{{ s}}",
            ])

            steps.append({
                "number": 3,
                "title": "Condição de Fase no LGR, Zero z e Tempo Integral τi",
                "description": "Inclusão do polo na origem (s = 0) do PI e cálculo da posição do zero z = 1/τi.",
                "equations": angle_eqs,
                "notes": [
                    f"Ângulo exigido do zero: θz = {_fmt_num(theta_z)}° ({'θz > 90°, zero à direita de -Re' if theta_z > 90 else 'θz < 90°, zero à esquerda de -Re'})",
                    f"Zero do controlador PI: z = {_fmt_num(z_val)} → τi = {_fmt_num(ti_val)} s",
                ],
            })

            # Condição de Módulo: |(Kp*(s+z)/s)*G(s)| = 1
            mag_num_factors = [_factor_magnitude_details(s_dom, complex(-z_val, 0.0), 1, f"(s + {_fmt_num(z_val)})")]
            for gz in grouped_zeros:
                mag_num_factors.append(_factor_magnitude_details(s_dom, gz["root"], gz["multiplicity"]))
            mag_den_factors = [_factor_magnitude_details(s_dom, gp["root"], gp["multiplicity"]) for gp in grouped_total_poles]

            num_mag_prod = abs(lead_gain) * math.prod(f["magnitude"] for f in mag_num_factors)
            den_mag_prod = math.prod(f["magnitude"] for f in mag_den_factors)
            kp_val = float(den_mag_prod / num_mag_prod)
            ki_val = float(kp_val * z_val)

            mag_eqs = [f["formula_latex"] for f in mag_num_factors + mag_den_factors]
            mag_eqs.append(
                rf"\left|G_{{ma}}(s)\right| = \left|\frac{{K_p(s + {_fmt_tex_num(z_val, 4)})}}{{s}}G(s)\right| = 1 \implies K_p = \frac{{{_fmt_tex_num(den_mag_prod, 4)}}}{{{_fmt_tex_num(num_mag_prod, 4)}}} = {_fmt_tex_num(kp_val, 4)}"
            )
            steps.append({
                "number": 4,
                "title": "Condição de Módulo e Cálculo de Kp",
                "description": "Aplicação de |Gma(s)| = |(Kp(s+z)/s)G(s)| = 1 no polo dominante.",
                "equations": mag_eqs,
                "notes": [
                    f"Ganho proporcional: Kp = {_fmt_num(kp_val)}",
                    f"Ganho integral: Ki = Kp·z = {_fmt_num(ki_val)} (τi = {_fmt_num(ti_val)} s)",
                ],
            })

            controller_tf = ct.tf([kp_val, kp_val * z_val], [1.0, 0.0])
            controller_params.update({
                "K": kp_val,
                "kp": kp_val,
                "ki": ki_val,
                "ti": float(ti_val),
                "z": float(z_val),
                "p": 0.0,
                "theta_z": float(theta_z),
                "re": float(re_dom),
                "im": float(im_dom),
                "xi": float(xi_val),
                "wn": float(wn_val),
                "ts": float(ts_val),
                "mp": float(mp_val),
                "criterion_pct": crit_pct,
            })
        else:
            raise ValueError(f"Tipo de controlador não suportado: {controller_kind}")

    # ==========================================================================
    # MONTAGEM FINAL DE G_c(s), G_ma(s), T(s), ERRO EM REGIME E GRÁFICOS
    # ==========================================================================
    compensated_loop = controller_tf * plant
    comp_num = np.asarray(ct.tfdata(compensated_loop)[0][0][0], dtype=float)
    comp_den = np.asarray(ct.tfdata(compensated_loop)[1][0][0], dtype=float)
    comp_num = np.where(np.abs(comp_num) < 1e-10, 0.0, comp_num)
    comp_den = np.where(np.abs(comp_den) < 1e-10, 0.0, comp_den)

    # Verifica se G_ma(s) é própria para simulação temporal; se imprópria, usa polo rápido para simulação
    if len(np.trim_zeros(comp_num, "f")) > len(np.trim_zeros(comp_den, "f")):
        sim_loop = compensated_loop * ct.tf([1.0], [1e-3, 1.0])
    else:
        sim_loop = compensated_loop

    uncompensated_cl = ct.feedback(plant, 1)
    compensated_cl = ct.feedback(sim_loop, 1)
    compensated_poles = ct.poles(compensated_cl)
    compensated_stable = bool(np.all(np.real(compensated_poles) < -1e-7))
    warning = "Malha fechada instável com estes parâmetros" if not compensated_stable else None

    err_after = evaluate_steady_state_error(comp_num, comp_den, input_type=input_mode)
    controller_params["e_atual"] = uncomp["steady_state_error_raw"]
    controller_params["e_novo"] = err_after["value"]

    gc_latex = _build_controller_latex(kind, controller_params)
    plant_clean_latex = plant_latex_factored.replace("G(s) = ", "")
    gma_factored_latex = rf"G_{{ma}}(s) = G_c(s)G(s) = {gc_latex.replace('G_c(s) = ', '')} \cdot \left({plant_clean_latex}\right)"
    _, _, gma_exp_raw, _ = format_transfer_function(
        [float(f"{c:.5g}") for c in np.trim_zeros(comp_num, "f")],
        [float(f"{c:.5g}") for c in np.trim_zeros(comp_den, "f")],
    )
    gma_expanded_latex = gma_exp_raw.replace("G(s) = ", "G_{ma}(s) = ")

    cl_num_arr = np.asarray(ct.tfdata(compensated_cl)[0][0][0], dtype=float)
    cl_den_arr = np.asarray(ct.tfdata(compensated_cl)[1][0][0], dtype=float)
    _, _, tc_exp_raw, _ = format_transfer_function(
        [float(f"{c:.5g}") for c in np.trim_zeros(cl_num_arr, "f")],
        [float(f"{c:.5g}") for c in np.trim_zeros(cl_den_arr, "f")],
    )
    tc_latex = tc_exp_raw.replace("G(s) = ", "T(s) = ")

    steps.append({
        "number": len(steps) + 1,
        "title": "Funções de Transferência Finais e Erro em Regime Permanente",
        "description": "Função do controlador Gc(s), malha aberta compensada Gma(s) = Gc(s)G(s) e erro final e(∞).",
        "equations": [
            gc_latex,
            gma_factored_latex,
            gma_expanded_latex,
            err_after["formula_latex"],
        ],
        "notes": [
            f"Estabilidade em malha fechada: {'Estável' if compensated_stable else 'Instável'}",
            f"Erro em regime permanente antes: {uncomp['steady_state_error_latex']} → depois: {err_after['latex_value']}",
        ],
    })

    # Simulação temporal (degrau, rampa ou parábola)
    time = recommended_time_vector(compensated_cl, final_time, points)
    if input_mode == "parabola":
        u_parabola = 0.5 * (time ** 2)
        _, y_before = response_arrays(ct.forced_response(uncompensated_cl, T=time, U=u_parabola))
        _, y_after = response_arrays(ct.forced_response(compensated_cl, T=time, U=u_parabola))
        ref_signal = u_parabola
        resp_title = "Resposta à Parábola Unitária"
    elif input_mode == "ramp":
        _, y_before = response_arrays(ct.forced_response(uncompensated_cl, T=time, U=time))
        _, y_after = response_arrays(ct.forced_response(compensated_cl, T=time, U=time))
        ref_signal = time
        resp_title = "Resposta à Rampa Unitária"
    else:
        _, y_before = response_arrays(ct.step_response(uncompensated_cl, timepts=time))
        _, y_after = response_arrays(ct.step_response(compensated_cl, timepts=time))
        ref_signal = np.ones_like(time)
        resp_title = "Resposta ao Degrau Unitário"

    metrics_before = _step_metrics(uncompensated_cl, time, threshold=0.05 if controller_params.get("criterion_pct") == 5 else 0.02)
    metrics_after = _step_metrics(compensated_cl, time, threshold=0.05 if controller_params.get("criterion_pct") == 5 else 0.02)

    # Cálculo das curvas do LGR (compensado unitário ao redor de G_ma)
    gains = np.concatenate(([0.0], np.logspace(-3, 2.5, 280)))
    locus_before = _root_locus_roots(plant, gains)
    locus_after = _root_locus_roots(compensated_loop, np.concatenate(([0.0], np.linspace(0.01, 1.0, 80), np.logspace(0.01, 2.2, 180))))

    # Plotagem: Painel 1 = LGR + Geometria de Ângulos; Painel 2 = Resposta Temporal
    tokens = get_theme_tokens(theme)
    accent = tokens["accent"]
    ref_color = tokens["reference"]
    danger_color = tokens.get("danger", "#ef4444")

    fig, (ax_lgr, ax_resp) = plt.subplots(1, 2, figsize=(11.8, 5.4), dpi=140)

    # --- Subplot 1: LGR Compensado, Polos/Zeros, Polo Dominante e Ângulos theta ---
    for branch in range(locus_before.shape[1]):
        ax_lgr.plot(
            np.real(locus_before[:, branch]),
            np.imag(locus_before[:, branch]),
            color=ref_color,
            linestyle=":",
            linewidth=1.1,
            alpha=0.55,
            label="LGR sem controlador" if branch == 0 else "_nolegend_",
        )
    for branch in range(locus_after.shape[1]):
        ax_lgr.plot(
            np.real(locus_after[:, branch]),
            np.imag(locus_after[:, branch]),
            color=accent,
            linewidth=2.0,
            label="LGR compensado Gma(s)" if branch == 0 else "_nolegend_",
        )

    ax_lgr.axhline(0, color=tokens["spine"], linewidth=0.9, alpha=0.75)
    ax_lgr.axvline(0, color=tokens["spine"], linewidth=0.9, alpha=0.75)

    # Polos e zeros da planta
    if len(plant_poles_raw) > 0:
        ax_lgr.plot(
            np.real(plant_poles_raw),
            np.imag(plant_poles_raw),
            marker="x",
            linestyle="none",
            markersize=8.5,
            markeredgewidth=2.0,
            color=ref_color,
            label="Polos de G(s)",
        )
    if len(plant_zeros_raw) > 0:
        ax_lgr.plot(
            np.real(plant_zeros_raw),
            np.imag(plant_zeros_raw),
            marker="o",
            linestyle="none",
            markersize=7.5,
            markerfacecolor="none",
            markeredgewidth=2.0,
            color=ref_color,
            label="Zeros de G(s)",
        )

    # Polo e zero do controlador
    if "z" in controller_params and controller_params["z"] is not None:
        cz = -float(controller_params["z"])
        ax_lgr.plot(
            [cz],
            [0.0],
            marker="o",
            linestyle="none",
            markersize=9.0,
            markerfacecolor=accent,
            markeredgewidth=1.8,
            color=accent,
            label=f"Zero Gc (-z = {_fmt_num(cz)})",
        )
    if "p" in controller_params and controller_params["p"] is not None:
        cp = -float(controller_params["p"])
        ax_lgr.plot(
            [cp],
            [0.0],
            marker="P",
            linestyle="none",
            markersize=9.0,
            color=danger_color,
            label=f"Polo Gc (-p = {_fmt_num(cp)})",
        )

    # Polo dominante desejado e linhas de ângulo theta
    dom_re = controller_params.get("re", 0.0)
    dom_im = controller_params.get("im", 0.0)
    if dom_im > 0:
        ax_lgr.plot(
            [-dom_re, -dom_re],
            [dom_im, -dom_im],
            marker="*",
            linestyle="none",
            markersize=11.5,
            color="#f59e0b",
            label=f"Polo dom. (-{_fmt_num(dom_re, 3)} ± {_fmt_num(dom_im, 3)}j)",
            zorder=6,
        )
        for contrib in angle_contributions:
            r_pt = contrib["root"]
            ax_lgr.plot(
                [r_pt.real, -dom_re],
                [r_pt.imag, dom_im],
                linestyle="--",
                linewidth=1.0,
                color="#f59e0b",
                alpha=0.75,
            )
            mid_x = 0.55 * r_pt.real + 0.45 * (-dom_re)
            mid_y = 0.55 * r_pt.imag + 0.45 * dom_im
            ax_lgr.annotate(
                f"{_fmt_num(contrib['angle_deg'], 3)}°",
                xy=(mid_x, mid_y),
                fontsize=7.5,
                color=tokens["text"],
                bbox={"boxstyle": "round,pad=0.18", "facecolor": tokens.get("bg_fig", "#0f172a"), "edgecolor": "none", "alpha": 0.8},
            )

    # Ajusta janela de visualização do LGR com foco na região de projeto
    interest_x = [-dom_re, 0.0] + [float(np.real(p)) for p in plant_poles_raw] + [float(np.real(z)) for z in plant_zeros_raw]
    if "z" in controller_params and controller_params["z"] is not None:
        interest_x.append(-float(controller_params["z"]))
    if "p" in controller_params and controller_params["p"] is not None:
        interest_x.append(-float(controller_params["p"]))
    min_x, max_x = min(interest_x), max(interest_x)
    span_x = max(abs(max_x - min_x), dom_re * 1.5, 2.0)
    span_y = max(dom_im * 1.6, span_x * 0.65, 2.0)
    ax_lgr.set_xlim(min_x - 0.3 * span_x, max(max_x + 0.25 * span_x, 0.8))
    ax_lgr.set_ylim(-span_y, span_y)
    ax_lgr.set_title("Lugar Geométrico das Raízes e Ângulos θ")
    ax_lgr.set_xlabel("Eixo Real (σ)")
    ax_lgr.set_ylabel("Eixo Imaginário (jω)")
    ax_lgr.legend(loc="best", fontsize=7.5)

    # --- Subplot 2: Resposta Temporal (Degrau ou Rampa) ---
    ax_resp.plot(time, ref_signal, color=tokens["spine"], linestyle=":", linewidth=1.2, label="Referência r(t)")
    ax_resp.plot(time, y_before, color=ref_color, linestyle="--", linewidth=1.6, label="Sem controlador")
    ax_resp.plot(time, y_after, color=accent, linewidth=2.2, label="Com controlador Gc(s)")
    ax_resp.set_title(resp_title)
    ax_resp.set_xlabel("Tempo (s)")
    ax_resp.set_ylabel("Saída y(t)")
    ax_resp.legend(loc="best", fontsize=8.0)

    fig.tight_layout()
    image, svg = figure_payload(fig, theme=theme)

    # Tabela de métricas estruturada para o painel de resultados
    dom_pole_str = f"-{_fmt_num(dom_re, 4)} ± {_fmt_num(dom_im, 4)}j"
    metric_rows: list[dict[str, Any]] = [
        {"header": "Síntese do Controlador"},
        {"label": "Tipo de controlador", "value": kind},
        {"label": "Polo dominante s", "value": dom_pole_str},
    ]

    if kind in {"LEAD", "LAG"}:
        metric_rows.extend([
            {"label": "Ganho (K)", "value": f"{controller_params['K']:.4g}"},
            {"label": "Zero do controlador (z)", "value": f"{controller_params['z']:.4g}"},
            {"label": "Polo do controlador (p)", "value": f"{controller_params['p']:.4g}"},
        ])
        if "z_over_p" in controller_params:
            metric_rows.append({"label": "Razão (z/p)", "value": f"{controller_params['z_over_p']:.4g}"})
        if "theta_p" in controller_params:
            metric_rows.append({"label": "Ângulo do polo (θp)", "value": f"{controller_params['theta_p']:.4g}", "unit": "°"})
    elif kind == "P":
        metric_rows.append({"label": "Ganho proporcional (Kp)", "value": f"{controller_params['kp']:.4g}"})
    elif kind == "PD":
        metric_rows.extend([
            {"label": "Ganho proporcional (Kp)", "value": f"{controller_params['kp']:.4g}"},
            {"label": "Zero do controlador (z)", "value": f"{controller_params['z']:.4g}"},
            {"label": "Tempo derivativo (τd)", "value": f"{controller_params['td']:.4g}", "unit": "s"},
            {"label": "Ângulo do zero (θz)", "value": f"{controller_params['theta_z']:.4g}", "unit": "°"},
        ])
    elif kind == "PI":
        metric_rows.extend([
            {"label": "Ganho proporcional (Kp)", "value": f"{controller_params['kp']:.4g}"},
            {"label": "Zero do controlador (z)", "value": f"{controller_params['z']:.4g}"},
            {"label": "Tempo integral (τi)", "value": f"{controller_params['ti']:.4g}", "unit": "s"},
            {"label": "Ângulo do zero (θz)", "value": f"{controller_params['theta_z']:.4g}", "unit": "°"},
        ])

    metric_rows.extend([
        {"header": "Especificações e Desempenho"},
        {"label": "Amortecimento (ξ)", "value": f"{controller_params['xi']:.4g}"},
        {"label": "Frequência natural (ωn)", "value": f"{controller_params['wn']:.4g}", "unit": "rad/s"},
        {"label": f"Tempo de acomodação ts({controller_params['criterion_pct']}%)", "value": f"{controller_params['ts']:.4g}" if controller_params.get("ts") is not None else "—", "unit": "s"},
        {"label": "Sobressinal máximo (Mp)", "value": f"{controller_params['mp']:.4g}", "unit": "%"},
        {"label": "Erro e(∞) sem controlador", "value": uncomp["steady_state_error_latex"].replace(r"\infty", "∞").replace("{,}", ",")},
        {"label": "Erro e(∞) compensado", "value": err_after["latex_value"].replace(r"\infty", "∞").replace("{,}", ",")},
        {"label": "Estabilidade", "value": "Estável" if compensated_stable else "Instável"},
    ])
    if warning:
        metric_rows.append({"label": "Aviso", "value": warning})

    details = {
        "design_type": kind.lower(),
        "controller_type": kind,
        "design_label": design_label,
        "input_type": input_mode,
        "settling_criterion": controller_params.get("criterion_pct", 2),
        "stable": compensated_stable,
        "warning": warning,
        "controller_parameters": controller_params,
        "uncompensated": uncomp,
        "angle_contributions": [
            {
                "type": c["type"],
                "label": c["label"],
                "root": {"re": float(c["root"].real), "im": float(c["root"].imag)},
                "multiplicity": c["multiplicity"],
                "angle_deg": float(c["angle_deg"]),
                "total_angle_deg": float(c["total_angle_deg"]),
            }
            for c in angle_contributions
        ],
        "steps": steps,
        "gc_latex": gc_latex,
        "gma_factored_latex": gma_factored_latex,
        "gma_expanded_latex": gma_expanded_latex,
        "tc_latex": tc_latex,
        "plant_latex_factored": plant_latex_factored,
        "plant_latex_expanded": plant_latex_expanded,
        "before": metrics_before,
        "after": metrics_after,
        "closed_loop_poles_before": serialize_complex(ct.poles(uncompensated_cl)),
        "closed_loop_poles_after": serialize_complex(compensated_poles),
    }

    return {
        "success": True,
        "module": "controller_design",
        "title": "Projeto de Controladores",
        "description": design_label,
        "image": image,
        "svg": svg,
        "latex": [
            {"label": "Controlador Gc(s)", "value": gc_latex},
            {"label": "Malha Aberta Compensada Gma(s) = Gc(s)G(s)", "value": gma_factored_latex},
            {"label": "Malha Aberta Expandida", "value": gma_expanded_latex},
            {"label": "Malha Fechada Compensada T(s)", "value": tc_latex},
            {"label": "Planta Original G(s)", "value": plant_latex_factored},
        ],
        "metrics": metric_rows,
        "details": details,
    }


# ==============================================================================
# MÉTODOS CLÁSSICOS LEGADOS (MANTIDOS PARA RETROCOMPATIBILIDADE DE TESTES)
# ==============================================================================

def _calculate_critical_params(plant):
    gm, _, _, wpc, _, _ = ct.stability_margins(plant)
    if not (math.isfinite(gm) and gm > 0 and math.isfinite(wpc) and wpc > 0):
        raise ValueError("A planta não possui frequência de oscilação crítica (-180°). Utilize a opção Manual ou sintonia por curva de reação.")
    ku = float(gm)
    pu = float(2.0 * math.pi / wpc)
    return ku, pu, float(wpc)


def _pid_parameters(method, controller_type, *, plant=None, process_gain=1.0, delay=1.0, time_constant=4.0, critical_gain=6.0, critical_period=2.0, chr_response="0"):
    method = str(method).lower()
    controller_type = str(controller_type).upper()
    if controller_type not in {"P", "PI", "PD", "PID"}:
        raise ValueError("O controlador deve ser P, PI, PD ou PID.")

    ti = None
    td = 0.0
    ku = None
    pu = None
    w_osc = None

    if method == "zn_reaction":
        gain = _positive(process_gain, "O ganho do processo")
        lag = _positive(delay, "O atraso L")
        tau = _positive(time_constant, "A constante de tempo T")
        if controller_type == "P":
            kp = tau / (gain * lag)
        elif controller_type == "PI":
            kp, ti = 0.9 * tau / (gain * lag), 3.33 * lag
        elif controller_type == "PD":
            kp, td = 1.2 * tau / (gain * lag), 0.5 * lag
        else:
            kp, ti, td = 1.2 * tau / (gain * lag), 2.0 * lag, 0.5 * lag
        method_label = "Ziegler-Nichols — curva de reação"
    elif method in {"zn_critical", "zn_critical_auto"}:
        if plant is not None:
            ku, pu, w_osc = _calculate_critical_params(plant)
        else:
            ku = _positive(critical_gain, "O ganho crítico Kcr")
            pu = _positive(critical_period, "O período crítico Pcr")
            w_osc = 2.0 * math.pi / pu
        if controller_type == "P":
            kp = 0.5 * ku
        elif controller_type == "PI":
            kp, ti = 0.45 * ku, pu / 1.2
        elif controller_type == "PD":
            kp, td = 0.8 * ku, pu / 8.0
        else:
            kp, ti, td = 0.6 * ku, pu / 2.0, pu / 8.0
        method_label = "Ziegler-Nichols — oscilação crítica (automático)"
    elif method == "zn_critical_manual":
        ku = _positive(critical_gain, "O ganho crítico Kcr")
        pu = _positive(critical_period, "O período crítico Pcr")
        w_osc = 2.0 * math.pi / pu
        if controller_type == "P":
            kp = 0.5 * ku
        elif controller_type == "PI":
            kp, ti = 0.45 * ku, pu / 1.2
        elif controller_type == "PD":
            kp, td = 0.8 * ku, pu / 8.0
        else:
            kp, ti, td = 0.6 * ku, pu / 2.0, pu / 8.0
        method_label = "Ziegler-Nichols — oscilação crítica (manual)"
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
        elif controller_type == "PD":
            kp = (tau / (gain * lag)) * (1.25 + ratio / 6.0)
            td = lag * (6.0 - 2.0 * ratio) / (22.0 + 3.0 * ratio)
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
                "PD": (0.7, None, 0.45 * lag),
                "PID": (0.95, 1.4 * tau, 0.47 * lag),
            }
        else:
            table = {
                "P": (0.3, None, 0.0),
                "PI": (0.35, 1.2 * tau, 0.0),
                "PD": (0.3, None, 0.5 * lag),
                "PID": (0.6, tau, 0.5 * lag),
            }
        factor, ti, td = table[controller_type]
        kp = factor * tau / (gain * lag)
        method_label = f"CHR — {response}% de sobressinal"
    else:
        raise ValueError("Método de sintonia desconhecido.")

    ki = 0.0 if ti is None else kp / ti
    kd = kp * td
    return {
        "kp": kp,
        "ki": ki,
        "kd": kd,
        "ti": ti,
        "td": td,
        "kcr": ku,
        "pcr": pu,
        "wcr": w_osc,
        "method_label": method_label,
    }


def _pid_transfer(parameters, controller_type):
    kp, ki, kd = parameters["kp"], parameters["ki"], parameters["kd"]
    ti, td = parameters.get("ti"), parameters.get("td", 0.0)
    if controller_type == "P":
        return ct.tf([kp], [1])
    if controller_type == "PI":
        return ct.tf([kp, ki], [1, 0])
    if controller_type == "PD":
        N = 10.0
        if td is not None and td > 0:
            tau_d = td / N
            num = [kp * (tau_d + td), kp]
            den = [tau_d, 1.0]
            return ct.tf(num, den)
        return ct.tf([kp], [1])
    N = 10.0
    if td is not None and td > 0:
        tau_d = td / N
        num = [kp * (tau_d + td), kp + ki * tau_d, ki]
        den = [tau_d, 1.0, 0.0]
        return ct.tf(num, den)
    return ct.tf([kp, ki], [1, 0])


def _step_metrics(system, time, threshold: float = 0.02):
    try:
        info = ct.step_info(system, timepts=time, SettlingTimeThreshold=threshold)
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
    num = np.asarray(ct.tfdata(controller)[0][0][0], dtype=float)
    den = np.asarray(ct.tfdata(controller)[1][0][0], dtype=float)
    num = np.where(np.abs(num) < 1e-9, 0.0, num)
    den = np.where(np.abs(den) < 1e-9, 0.0, den)
    num = [float(f"{c:.4g}") for c in num]
    den = [float(f"{c:.4g}") for c in den]
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


def _legacy_design_controller(
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
    theme="dark",
):
    plant, _, _, plant_latex_expanded, plant_latex_factored = parse_siso_transfer_function(plant_expression)
    design_type = str(design_type).lower()
    controller_type = str(controller_type).upper()
    parameters = None

    if design_type == "pid":
        parameters = _pid_parameters(
            method,
            controller_type,
            plant=plant,
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
    compensated_poles = ct.poles(compensated)
    compensated_stable = bool(np.all(np.real(compensated_poles) < -1e-9))
    warning = "Malha fechada instável com estes parâmetros" if not compensated_stable else None

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

    tokens = get_theme_tokens(theme)
    accent = tokens["accent"]
    ref_color = tokens["reference"]

    fig, axes = plt.subplots(2, 2, figsize=(11.2, 7.8), dpi=140)
    ax_step, ax_locus, ax_mag, ax_phase = axes.flat

    ax_step.plot(t_before, y_before, color=ref_color, linestyle="--", linewidth=1.8, label="Antes")
    ax_step.plot(t_after, y_after, color=accent, linewidth=2.2, label="Depois")
    ax_step.set_xlabel("Tempo (s)")
    ax_step.set_ylabel("Saída y(t)")
    ax_step.legend(loc="best")

    for branch in range(locus_before.shape[1]):
        ax_locus.plot(np.real(locus_before[:, branch]), np.imag(locus_before[:, branch]), color=ref_color, linestyle="--", linewidth=1.2, alpha=0.75, label="Antes" if branch == 0 else "_nolegend_")
    for branch in range(locus_after.shape[1]):
        ax_locus.plot(np.real(locus_after[:, branch]), np.imag(locus_after[:, branch]), color=accent, linewidth=2.0, label="Depois" if branch == 0 else "_nolegend_")
    ax_locus.axhline(0, color=tokens["spine"], linewidth=0.8, alpha=0.7)
    ax_locus.axvline(0, color=tokens["spine"], linewidth=0.8, alpha=0.7)
    ax_locus.set_xlabel("Eixo real")
    ax_locus.set_ylabel("Eixo imaginário")
    ax_locus.legend(loc="best")

    ax_mag.semilogx(omega, magnitude_before, color=ref_color, linestyle="--", linewidth=1.6, label="Antes")
    ax_mag.semilogx(omega, magnitude_after, color=accent, linewidth=2.0, label="Depois")
    ax_mag.set_xlabel("Frequência (rad/s)")
    ax_mag.set_ylabel("Magnitude (dB)")
    ax_mag.legend(loc="best")

    ax_phase.semilogx(omega, phase_before, color=ref_color, linestyle="--", linewidth=1.6, label="Antes")
    ax_phase.semilogx(omega, phase_after, color=accent, linewidth=2.0, label="Depois")
    ax_phase.set_xlabel("Frequência (rad/s)")
    ax_phase.set_ylabel("Fase (graus)")
    ax_phase.legend(loc="best")

    fig.tight_layout()
    image, svg = figure_payload(fig, theme=theme)

    def metric_value(metrics, key):
        value = metrics.get(key)
        return "Não definido" if value is None else str(value)

    metric_rows = [
        {"label": "Estabilidade", "value": "Estável" if compensated_stable else "Instável"},
    ]
    if warning:
        metric_rows.append({"label": "Aviso", "value": warning})

    if parameters:
        metric_rows.extend([
            {"label": "Kp", "value": f"{parameters['kp']:.5g}"},
            {"label": "Ki", "value": f"{parameters['ki']:.5g}"},
            {"label": "Kd", "value": f"{parameters['kd']:.5g}"},
            {"label": "Ti", "value": f"{parameters['ti']:.5g}" if parameters["ti"] is not None else "—", "unit": "s" if parameters["ti"] is not None else ""},
            {"label": "Td", "value": f"{parameters['td']:.5g}" if parameters["td"] is not None else "—", "unit": "s" if parameters["td"] is not None else ""},
        ])

    metric_rows.extend([
        {"label": "Sobressinal (%OS)", "value": metric_value(metrics_after, "overshoot_percent"), "unit": "%"},
        {"label": "Tempo de acomodação (ts)", "value": metric_value(metrics_after, "settling_time"), "unit": "s"},
        {"label": "Margem de ganho (MG)", "value": str(margins_after.get("gain_margin_db")), "unit": "dB"},
        {"label": "Margem de fase (MF)", "value": str(margins_after.get("phase_margin_deg")), "unit": "°"},
        {"label": "Projeto", "value": design_label},
    ])

    details = {
        "design_type": design_type,
        "design_label": design_label,
        "stable": compensated_stable,
        "warning": warning,
        "controller_parameters": parameters,
        "kcr": parameters.get("kcr") if parameters else None,
        "pcr": parameters.get("pcr") if parameters else None,
        "wcr": parameters.get("wcr") if parameters else None,
        "before": metrics_before,
        "after": metrics_after,
        "frequency_margins_before": margins_before,
        "frequency_margins_after": margins_after,
        "closed_loop_poles_before": serialize_complex(ct.poles(uncompensated)),
        "closed_loop_poles_after": serialize_complex(compensated_poles),
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


def design_controller(
    plant_expression: str,
    *,
    design_type: str | None = None,
    method: str | None = None,
    controller_type: str = "lead",
    input_type: str = "step",
    settling_criterion: str | int = "2",
    ts: Any = None,
    mp: Any = None,
    xi: Any = None,
    wn: Any = None,
    primary_damping_spec: str | None = None,
    primary_freq_spec: str | None = None,
    error_reduction_factor: Any = None,
    target_steady_state_error: Any = None,
    lead_zero: Any = None,
    lag_pole: Any = None,
    lag_gain: Any = None,
    lgr_branch_re: Any = None,
    process_gain: Any = 1.0,
    delay: Any = 1.0,
    time_constant: Any = 4.0,
    critical_gain: Any = 6.0,
    critical_period: Any = 2.0,
    chr_response: str = "0",
    compensator_type: str = "lead",
    compensator_zero: Any = 1.0,
    compensator_pole: Any = 5.0,
    compensator_gain: Any = 1.0,
    design_domain: str = "frequency",
    final_time: Any = None,
    points: int = 900,
    theme: str = "dark",
) -> dict[str, Any]:
    """
    Ponto de entrada unificado para projeto de controladores:
    - Por padrão (ou quando controller_type in {'lead', 'lag', 'P', 'PD', 'PI'} sem method legado),
      executa o motor analítico universal de controladores em série no LGR.
    - Quando method legado ('zn_critical', 'cohen_coon', 'chr', etc.) ou design_type='lead_lag'
      com parâmetros diretos é informado, mantém compatibilidade com os testes legados.
    """
    legacy_methods = {"zn_critical", "zn_critical_auto", "zn_critical_manual", "zn_reaction", "cohen_coon", "chr"}
    if (
        (method is not None and str(method).lower() in legacy_methods)
        or (str(design_type or "").lower() == "lead_lag")
    ):
        return _legacy_design_controller(
            plant_expression,
            design_type=design_type or "pid",
            method=method or "zn_critical",
            controller_type=controller_type if str(controller_type).upper() in {"P", "PI", "PD", "PID"} else "PID",
            process_gain=process_gain,
            delay=delay,
            time_constant=time_constant,
            critical_gain=critical_gain,
            critical_period=critical_period,
            chr_response=chr_response,
            compensator_type=compensator_type,
            compensator_zero=compensator_zero,
            compensator_pole=compensator_pole,
            compensator_gain=compensator_gain,
            design_domain=design_domain,
            final_time=final_time,
            points=points,
            theme=theme,
        )

    effective_kind = str(controller_type or design_type or "lead").strip()
    return _design_lgr_series_controller(
        plant_expression,
        controller_kind=effective_kind,
        input_type=input_type,
        settling_criterion=settling_criterion,
        ts=ts,
        mp=mp,
        xi=xi,
        wn=wn,
        primary_damping_spec=primary_damping_spec,
        primary_freq_spec=primary_freq_spec,
        error_reduction_factor=error_reduction_factor,
        target_steady_state_error=target_steady_state_error,
        lead_zero=lead_zero,
        lag_pole=lag_pole,
        lag_gain=lag_gain,
        lgr_branch_re=lgr_branch_re,
        final_time=final_time,
        points=points,
        theme=theme,
    )


def get_controller_presets():
    return [
        {"id": name, "title": name, **cfg}
        for name, cfg in CONTROLLER_PRESETS.items()
    ]
