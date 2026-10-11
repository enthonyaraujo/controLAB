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
import unittest
import numpy as np
import scipy.signal as signal
import control as ct

from time_response import analyze_time_response
from routh_hurwitz import analyze_routh_hurwitz
from lgr_engine import lgr_completo
from frequency_response import analyze_frequency_response
from controller_design import design_controller
from state_space import analyze_state_space
from python_bridge import dispatch


class TestNumerico(unittest.TestCase):
    def assert_approx(self, actual, expected, rel=1e-2, abs_tol=1e-6):
        self.assertTrue(
            math.isclose(actual, expected, rel_tol=rel, abs_tol=abs_tol),
            f"Esperado {expected}, obtido {actual} (rel_tol={rel}, abs_tol={abs_tol})"
        )

    def test_tempo_numerico(self):
        """A1. TEMPO: G(s) = 4/(s^2+2s+4)"""
        res = analyze_time_response("4 / (s^2 + 2*s + 4)")
        step = res["details"]["step"]

        # Comparação direta com python-control / scipy
        plant = ct.tf([4.0], [1.0, 2.0, 4.0])
        info = ct.step_info(plant, SettlingTimeThreshold=0.02)

        self.assertTrue(res["details"]["stable"])
        # Valor final = 1.0
        self.assert_approx(step["final_value"], 1.0, rel=1e-2)
        # zeta = 0.5
        self.assert_approx(step["damping_ratio"], 0.5, rel=1e-2)
        # wn = 2.0 rad/s
        self.assert_approx(step["natural_frequency"], 2.0, rel=1e-2)
        # Sobressinal ~ 16.3%
        self.assert_approx(step["overshoot_percent"], info["Overshoot"], rel=1e-2)
        self.assert_approx(step["overshoot_percent"], 16.3, rel=0.05)
        # Tempo de pico ~ 1.81 s
        self.assert_approx(step["peak_time"], info["PeakTime"], rel=1e-2)
        self.assert_approx(step["peak_time"], 1.81, rel=0.02)
        # Tempo de acomodação (2%) ~ 4.0 s
        self.assert_approx(step["settling_time"], info["SettlingTime"], rel=1e-2)
        self.assert_approx(step["settling_time"], 4.0, rel=0.05)

    def test_estab_numerico(self):
        """A6. ESTAB: s^3 + 3s^2 + 2s + K"""
        res = analyze_routh_hurwitz("s^3 + 3*s^2 + 2*s + K")
        k_range = res["k_range"]

        self.assertTrue(k_range["has_k"])
        # 0 < K < 6
        marginal = k_range["marginal_cases"]
        self.assertGreaterEqual(len(marginal), 1)
        mc = marginal[0]
        self.assert_approx(mc["k_val"], 6.0, rel=1e-2)
        self.assert_approx(mc["omega_osc"], math.sqrt(2.0), rel=1e-2)

        # Conferência com raízes em K = 6 (polos puramente imaginários em +- j*sqrt(2))
        roots_crit = np.roots([1.0, 3.0, 2.0, 6.0])
        imag_roots = [abs(r.imag) for r in roots_crit if abs(r.real) < 1e-4]
        self.assertEqual(len(imag_roots), 2)
        self.assert_approx(min(imag_roots), math.sqrt(2.0), rel=1e-2)

    def test_lgr_numerico(self):
        """A5. LGR: G(s) = (s+2)/(s*(s+1)*(s+4))"""
        _fig, _ax, det = lgr_completo([1.0, 2.0], [1.0, 5.0, 4.0, 0.0])

        self.assertEqual(det["P"], 3)
        self.assertEqual(det["Z"], 1)
        self.assertEqual(det["ramos"], 3)
        # Centroide = -1.5
        self.assert_approx(det["centroide"], -1.5, rel=1e-2)
        # 2 assíntotas a +-90°
        self.assertEqual(len(det["angulos_assintotas"]), 2)
        asymp_deg = [round(a["graus"]) for a in det["angulos_assintotas"]]
        self.assertEqual(set(asymp_deg), {90, 270})

        # Ponto de ruptura em ~ -0.55 com K ~ 0.59
        self.assertGreaterEqual(len(det["break_points"]), 1)
        bp = det["break_points"][0]
        self.assert_approx(bp["s"], -0.55, rel=0.03)
        self.assert_approx(bp["K"], 0.59, rel=0.03)

        # Sem cruzamento do eixo jw
        self.assertEqual(len(det["jw_cruzamentos"]), 0)

    def test_freq_numerico(self):
        """A2. FREQ: G(s) = 10/(s*(s+2)*(s+5))"""
        res = analyze_frequency_response("10 / (s * (s + 2) * (s + 5))")
        details = res["details"]

        # python-control stability_margins
        plant = ct.tf([10.0], [1.0, 7.0, 10.0, 0.0])
        gm_ct, pm_ct, wpc_ct, wgc_ct = ct.margin(plant)

        # Margem de Ganho = 7 (16.9 dB) em wcf = sqrt(10) ~ 3.16 rad/s
        self.assert_approx(details["gain_margin"], 7.0, rel=1e-2)
        self.assert_approx(details["gain_margin_db"], 20.0 * math.log10(7.0), rel=1e-2)
        self.assert_approx(details["phase_crossover_frequency"], math.sqrt(10.0), rel=1e-2)

        # Margem de Fase ~ 55° (55.6°) em wcg ~ 0.90 rad/s
        self.assert_approx(details["phase_margin_deg"], pm_ct, rel=1e-2)
        self.assert_approx(details["gain_crossover_frequency"], wgc_ct, rel=1e-2)
        self.assert_approx(details["phase_margin_deg"], 55.6, rel=0.03)
        self.assert_approx(details["gain_crossover_frequency"], 0.898, rel=0.03)

        # Estabilidade em malha fechada (1 + G = 0)
        self.assertTrue(details["stable"])
        self.assertTrue(details["has_origin_pole"])

    def test_control_numerico(self):
        """A3. CONTROL: planta G(s) = 1/(s*(s+1)*(s+5)), ZN automático e manual"""
        # ZN Automático
        res_auto = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical",
            controller_type="PID",
        )
        p_auto = res_auto["details"]["controller_parameters"]

        # Kcr = 30, wcr = sqrt(5), Pcr = 2*pi/sqrt(5) ~ 2.81 s
        self.assert_approx(p_auto["kcr"], 30.0, rel=1e-2)
        self.assert_approx(p_auto["wcr"], math.sqrt(5.0), rel=1e-2)
        self.assert_approx(p_auto["pcr"], 2.0 * math.pi / math.sqrt(5.0), rel=1e-2)
        self.assert_approx(p_auto["pcr"], 2.81, rel=0.02)

        # Tabela ZN PID: Kp = 0.6*Kcr = 18, Ti = Pcr/2 ~ 1.405 s, Td = Pcr/8 ~ 0.351 s
        self.assert_approx(p_auto["kp"], 18.0, rel=1e-2)
        self.assert_approx(p_auto["ti"], 1.405, rel=0.02)
        self.assert_approx(p_auto["td"], 0.351, rel=0.02)

        # Malha fechada compensada é estável com PID filtrado (N=10)
        self.assertTrue(res_auto["details"]["stable"])
        self.assertIsNone(res_auto["details"]["warning"])

        # ZN Manual com parâmetros instáveis (Kcr=6, Pcr=2)
        res_man = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical_manual",
            controller_type="PID",
            critical_gain=6.0,
            critical_period=2.0,
        )
        self.assertFalse(res_man["details"]["stable"])
        self.assertEqual(res_man["details"]["warning"], "Malha fechada instável com estes parâmetros")

    def test_estados_numerico(self):
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
        self.assertTrue(details["stable"])
        orig_poles = [p["re"] for p in details["original_poles"]]
        np.testing.assert_allclose(sorted(orig_poles), [-2.0, -1.0], rtol=1e-2)

        # Controlabilidade e Observabilidade
        self.assertEqual(details["controllability_rank"], 2)
        self.assertEqual(details["observability_rank"], 2)

        # Ganhos K e L validados com scipy.signal.place_poles
        scipy_k = signal.place_poles(A, B, [-4, -5]).gain_matrix
        scipy_l = signal.place_poles(A.T, C.T, [-6, -7]).gain_matrix.T
        k_res = np.array(details["state_feedback_gain"], dtype=float)
        l_res = np.array(details["observer_gain"], dtype=float)

        np.testing.assert_allclose(k_res, scipy_k, rtol=1e-2)
        np.testing.assert_allclose(l_res, scipy_l, rtol=1e-2)
        np.testing.assert_allclose(k_res, [[18.0, 6.0]], rtol=1e-2)
        np.testing.assert_allclose(l_res, [[10.0], [10.0]], rtol=1e-2)

    def test_tab_state_isolation(self):
        """Garante que a execução de uma aba não muta ou polui os dados de outra aba."""
        # 1. Executa aba TEMPO
        res_tempo = dispatch({"action": "time_response", "expr": "4 / (s^2 + 2*s + 4)"})
        self.assertTrue(res_tempo["details"]["stable"])
        self.assertEqual(res_tempo["module"], "time_response")
        self.assert_approx(res_tempo["details"]["step"]["final_value"], 1.0, rel=1e-2)

        # 2. Executa aba FREQ
        res_freq = dispatch({"action": "frequency_response", "expr": "10 / (s * (s + 2) * (s + 5))"})
        self.assertTrue(res_freq["details"]["stable"])
        self.assertEqual(res_freq["module"], "frequency_response")
        self.assert_approx(res_freq["details"]["gain_margin"], 7.0, rel=1e-2)

        # 3. Executa aba CONTROL com planta instável
        res_ctrl = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 1) * (s + 5))",
            "method": "zn_critical_manual",
            "critical_gain": 6.0,
            "critical_period": 2.0,
        })
        self.assertFalse(res_ctrl["details"]["stable"])
        self.assertEqual(res_ctrl["module"], "controller_design")

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
        self.assertTrue(res_estados["details"]["stable"])
        self.assertEqual(res_estados["module"], "state_space")

        # 5. Executa aba ROUTH
        res_routh = dispatch({"action": "routh_hurwitz", "expr": "s^3 + 3*s^2 + 2*s + 6"})
        self.assertEqual(res_routh["verdict"], "Marginalmente Estável")

        # Re-verifica que resultados anteriores permaneceram íntegros e isolados
        self.assert_approx(res_tempo["details"]["step"]["final_value"], 1.0, rel=1e-2)
        self.assert_approx(res_freq["details"]["gain_margin"], 7.0, rel=1e-2)
        self.assertFalse(res_ctrl["details"]["stable"])
        self.assertTrue(res_estados["details"]["stable"])


if __name__ == "__main__":
    unittest.main()
