"""Suíte de testes numéricos e de isolamento de estado do ControLAB v3.1.1.

Cobre com tolerância relativa de 1e-2 todos os módulos do sistema:
- TEMPO: G(s) = 4 / (s^2 + 2s + 4)
- ESTAB: s^3 + 3s^2 + 2s + K
- LGR: G(s) = (s + 2) / (s * (s + 1) * (s + 4))
- FREQ: G(s) = 10 / (s * (s + 2) * (s + 5))
- CONTROL: G(s) = 1 / (s * (s + 1) * (s + 5)) com Ziegler-Nichols automático
- ESTADOS: A=[[0,1],[-2,-3]], B=[[0],[1]], C=[[1,0]], D=[[0]]
- Isolamento de estado entre abas
"""

import math
import numpy as np
import pytest
import scipy.signal as signal
import control as ct

from time_response import analyze_time_response
from routh_hurwitz import analyze_routh_hurwitz
from lgr_engine import lgr_completo
from frequency_response import analyze_frequency_response
from controller_design import design_controller
from state_space import analyze_state_space
from python_bridge import dispatch


def test_tempo_numerico():
    """A1. TEMPO: G(s) = 4/(s^2+2s+4)"""
    res = analyze_time_response("4 / (s^2 + 2*s + 4)")
    step = res["details"]["step"]

    # Comparação direta com python-control / scipy
    plant = ct.tf([4.0], [1.0, 2.0, 4.0])
    info = ct.step_info(plant, SettlingTimeThreshold=0.02)

    assert res["details"]["stable"] is True
    # Valor final = 1.0
    assert pytest.approx(1.0, rel=1e-2) == step["final_value"]
    # zeta = 0.5
    assert pytest.approx(0.5, rel=1e-2) == step["damping_ratio"]
    # wn = 2.0 rad/s
    assert pytest.approx(2.0, rel=1e-2) == step["natural_frequency"]
    # Sobressinal ~ 16.3%
    assert pytest.approx(info["Overshoot"], rel=1e-2) == step["overshoot_percent"]
    assert pytest.approx(16.3, rel=0.05) == step["overshoot_percent"]
    # Tempo de pico ~ 1.81 s
    assert pytest.approx(info["PeakTime"], rel=1e-2) == step["peak_time"]
    assert pytest.approx(1.81, rel=0.02) == step["peak_time"]
    # Tempo de acomodação (2%) ~ 4.0 s
    assert pytest.approx(info["SettlingTime"], rel=1e-2) == step["settling_time"]
    assert pytest.approx(4.0, rel=0.05) == step["settling_time"]


def test_estab_numerico():
    """A6. ESTAB: s^3 + 3s^2 + 2s + K"""
    res = analyze_routh_hurwitz("s^3 + 3*s^2 + 2*s + K")
    k_range = res["k_range"]

    assert k_range["has_k"] is True
    # 0 < K < 6
    marginal = k_range["marginal_cases"]
    assert len(marginal) >= 1
    mc = marginal[0]
    assert pytest.approx(6.0, rel=1e-2) == mc["k_val"]
    assert pytest.approx(math.sqrt(2.0), rel=1e-2) == mc["omega_osc"]

    # Conferência com raízes em K = 6 (polos puramente imaginários em +- j*sqrt(2))
    roots_crit = np.roots([1.0, 3.0, 2.0, 6.0])
    imag_roots = [abs(r.imag) for r in roots_crit if abs(r.real) < 1e-4]
    assert len(imag_roots) == 2
    assert pytest.approx(math.sqrt(2.0), rel=1e-2) == min(imag_roots)


def test_lgr_numerico():
    """A5. LGR: G(s) = (s+2)/(s*(s+1)*(s+4))"""
    _fig, _ax, det = lgr_completo([1.0, 2.0], [1.0, 5.0, 4.0, 0.0])

    assert det["P"] == 3
    assert det["Z"] == 1
    assert det["ramos"] == 3
    # Centroide = -1.5
    assert pytest.approx(-1.5, rel=1e-2) == det["centroide"]
    # 2 assíntotas a +-90°
    assert len(det["angulos_assintotas"]) == 2
    asymp_deg = [round(a["graus"]) for a in det["angulos_assintotas"]]
    assert set(asymp_deg) == {90, 270}

    # Ponto de ruptura em ~ -0.55 com K ~ 0.59
    assert len(det["break_points"]) >= 1
    bp = det["break_points"][0]
    assert pytest.approx(-0.55, rel=0.03) == bp["s"]
    assert pytest.approx(0.59, rel=0.03) == bp["K"]

    # Sem cruzamento do eixo jw
    assert len(det["jw_cruzamentos"]) == 0


def test_freq_numerico():
    """A2. FREQ: G(s) = 10/(s*(s+2)*(s+5))"""
    res = analyze_frequency_response("10 / (s * (s + 2) * (s + 5))")
    details = res["details"]

    # python-control stability_margins
    plant = ct.tf([10.0], [1.0, 7.0, 10.0, 0.0])
    gm_ct, pm_ct, wpc_ct, wgc_ct = ct.margin(plant)

    # Margem de Ganho = 7 (16.9 dB) em wcf = sqrt(10) ~ 3.16 rad/s
    assert pytest.approx(7.0, rel=1e-2) == details["gain_margin"]
    assert pytest.approx(20.0 * math.log10(7.0), rel=1e-2) == details["gain_margin_db"]
    assert pytest.approx(math.sqrt(10.0), rel=1e-2) == details["phase_crossover_frequency"]

    # Margem de Fase ~ 55° (55.6°) em wcg ~ 0.90 rad/s
    assert pytest.approx(pm_ct, rel=1e-2) == details["phase_margin_deg"]
    assert pytest.approx(wgc_ct, rel=1e-2) == details["gain_crossover_frequency"]
    assert pytest.approx(55.6, rel=0.03) == details["phase_margin_deg"]
    assert pytest.approx(0.898, rel=0.03) == details["gain_crossover_frequency"]

    # Estabilidade em malha fechada (1 + G = 0)
    assert details["stable"] is True
    assert details["has_origin_pole"] is True


def test_control_numerico():
    """A3. CONTROL: planta G(s) = 1/(s*(s+1)*(s+5)), ZN automático e manual"""
    # ZN Automático
    res_auto = design_controller(
        "1 / (s * (s + 1) * (s + 5))",
        method="zn_critical",
        controller_type="PID",
    )
    p_auto = res_auto["details"]["controller_parameters"]

    # Kcr = 30, wcr = sqrt(5), Pcr = 2*pi/sqrt(5) ~ 2.81 s
    assert pytest.approx(30.0, rel=1e-2) == p_auto["kcr"]
    assert pytest.approx(math.sqrt(5.0), rel=1e-2) == p_auto["wcr"]
    assert pytest.approx(2.0 * math.pi / math.sqrt(5.0), rel=1e-2) == p_auto["pcr"]
    assert pytest.approx(2.81, rel=0.02) == p_auto["pcr"]

    # Tabela ZN PID: Kp = 0.6*Kcr = 18, Ti = Pcr/2 ~ 1.405 s, Td = Pcr/8 ~ 0.351 s
    assert pytest.approx(18.0, rel=1e-2) == p_auto["kp"]
    assert pytest.approx(1.405, rel=0.02) == p_auto["ti"]
    assert pytest.approx(0.351, rel=0.02) == p_auto["td"]

    # Malha fechada compensada é estável com PID filtrado (N=10)
    assert res_auto["details"]["stable"] is True
    assert res_auto["details"]["warning"] is None

    # ZN Manual com parâmetros instáveis (Kcr=6, Pcr=2)
    res_man = design_controller(
        "1 / (s * (s + 1) * (s + 5))",
        method="zn_critical_manual",
        controller_type="PID",
        critical_gain=6.0,
        critical_period=2.0,
    )
    assert res_man["details"]["stable"] is False
    assert res_man["details"]["warning"] == "Malha fechada instável com estes parâmetros"


def test_estados_numerico():
    """A4. ESTADOS: A=[[0,1],[-2,-3]], B=[[0],[1]], C=[[1,0]], D=[[0]]"""
    A = np.array([[0.0, 1.0], [-2.0, -3.0]])
    B = np.array([[0.0], [1.0]])
    C = np.array([[1.0, 0.0]])
    D = np.array([[0.0]])

    res = analyze_state_space(
        a="[[0, 1], [-2, -3]]",
        b="[[0], [1]]",
        c="[[1, 0]]",
        d="[[0]]",
        desired_poles="-4, -5",
        observer_poles="-6, -7",
    )
    details = res["details"]

    # Polos em -1 e -2 -> estável
    assert details["stable"] is True
    orig_poles = [p["re"] for p in details["original_poles"]]
    np.testing.assert_allclose(sorted(orig_poles), [-2.0, -1.0], rtol=1e-2)

    # Controlabilidade e Observabilidade
    assert details["controllability_rank"] == 2
    assert details["observability_rank"] == 2

    # Ganhos K e L validados com scipy.signal.place_poles
    scipy_k = signal.place_poles(A, B, [-4, -5]).gain_matrix
    scipy_l = signal.place_poles(A.T, C.T, [-6, -7]).gain_matrix.T
    k_res = np.array(details["state_feedback_gain"], dtype=float)
    l_res = np.array(details["observer_gain"], dtype=float)

    np.testing.assert_allclose(k_res, scipy_k, rtol=1e-2)
    np.testing.assert_allclose(l_res, scipy_l, rtol=1e-2)
    np.testing.assert_allclose(k_res, [[18.0, 6.0]], rtol=1e-2)
    np.testing.assert_allclose(l_res, [[10.0], [10.0]], rtol=1e-2)


def test_tab_state_isolation():
    """Garante que a execução de uma aba não muta ou polui os dados de outra aba."""
    # 1. Executa aba TEMPO
    res_tempo = dispatch({"action": "time_response", "expr": "4 / (s^2 + 2*s + 4)"})
    assert res_tempo["details"]["stable"] is True
    assert res_tempo["module"] == "time_response"
    assert pytest.approx(1.0, rel=1e-2) == res_tempo["details"]["step"]["final_value"]

    # 2. Executa aba FREQ
    res_freq = dispatch({"action": "frequency_response", "expr": "10 / (s * (s + 2) * (s + 5))"})
    assert res_freq["details"]["stable"] is True
    assert res_freq["module"] == "frequency_response"
    assert pytest.approx(7.0, rel=1e-2) == res_freq["details"]["gain_margin"]

    # 3. Executa aba CONTROL com planta instável
    res_ctrl = dispatch({
        "action": "controller_design",
        "plant_expr": "1 / (s * (s + 1) * (s + 5))",
        "method": "zn_critical_manual",
        "critical_gain": 6.0,
        "critical_period": 2.0,
    })
    assert res_ctrl["details"]["stable"] is False
    assert res_ctrl["module"] == "controller_design"

    # 4. Executa aba ESTADOS
    res_estados = dispatch({
        "action": "state_space",
        "mode": "matrices",
        "A": "[[0, 1], [-2, -3]]",
        "B": "[[0], [1]]",
        "C": "[[1, 0]]",
        "D": "[[0]]",
        "desired_poles": "-4, -5",
        "observer_poles": "-6, -7",
    })
    assert res_estados["details"]["stable"] is True
    assert res_estados["module"] == "state_space"

    # 5. Executa aba ROUTH
    res_routh = dispatch({"action": "routh_hurwitz", "expr": "s^3 + 3*s^2 + 2*s + 6"})
    assert res_routh["verdict"] == "Marginalmente Estável"

    # Re-verifica que resultados anteriores permaneceram íntegros e isolados
    assert pytest.approx(1.0, rel=1e-2) == res_tempo["details"]["step"]["final_value"]
    assert pytest.approx(7.0, rel=1e-2) == res_freq["details"]["gain_margin"]
    assert res_ctrl["details"]["stable"] is False
    assert res_estados["details"]["stable"] is True
