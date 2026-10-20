"""Testes unitários completos do módulo de Projeto de Controladores (LGR e sintonia clássica)."""

import json
import math
import unittest

from controller_design import (
    analyze_uncompensated_system,
    calculate_lgr_angle,
    calculate_position_from_angle,
    calculate_wn_from_ts,
    design_controller,
    evaluate_steady_state_error,
    mp_from_xi,
    resolve_transient_specifications,
    xi_from_mp,
)
from python_bridge import dispatch


class ControllerDesignLGRTests(unittest.TestCase):
    """Valida todos os cenários do motor matemático universal e as regras de projeto dos 5 controladores."""

    def test_mp_and_xi_conversion(self):
        xi = 0.5
        mp = mp_from_xi(xi)
        self.assertAlmostEqual(mp, 16.303, places=2)
        self.assertAlmostEqual(xi_from_mp(mp), xi, places=6)

        xi_10 = xi_from_mp(10.0)
        self.assertAlmostEqual(xi_10, 0.591, places=3)

        xi_25 = xi_from_mp(25.0)
        self.assertAlmostEqual(xi_25, 0.404, places=2)

    def test_wn_for_ts_2pct_and_5pct(self):
        # Critério de 2%: wn = 4 / (xi * ts)
        self.assertAlmostEqual(calculate_wn_from_ts(2.0, 0.5, "2"), 4.0, places=6)
        # Critério de 5%: wn = 3 / (xi * ts)
        self.assertAlmostEqual(calculate_wn_from_ts(2.0, 0.6, "5"), 2.5, places=6)

    def test_generic_lgr_angles_and_positions(self):
        # Polo à direita de -Re (p_i < Re): Re=2, Im=2*sqrt(3), p_i=0 -> theta = 120°
        im = 2.0 * math.sqrt(3.0)
        a_right = calculate_lgr_angle(0.0, 2.0, im)
        self.assertAlmostEqual(a_right["angle_deg"], 120.0, places=4)

        # Polo alinhado com -Re (p_i == Re): theta = 90°
        a_equal = calculate_lgr_angle(2.0, 2.0, im)
        self.assertAlmostEqual(a_equal["angle_deg"], 90.0, places=4)

        # Polo à esquerda de -Re (p_i > Re): p_i=5.6, Re=2, Im=2*sqrt(3) -> theta ~ 43.9°
        a_left = calculate_lgr_angle(5.6, 2.0, im)
        self.assertAlmostEqual(a_left["angle_deg"], 43.8979, places=3)

        # Posição a partir de theta < 90°, theta == 90° e theta > 90°
        pos_left = calculate_position_from_angle(a_left["angle_deg"], 2.0, im, "p")
        self.assertAlmostEqual(pos_left["pos"], 5.6, places=4)

        pos_aligned = calculate_position_from_angle(90.0, 2.0, im, "z")
        self.assertAlmostEqual(pos_aligned["pos"], 2.0, places=4)

        pos_right = calculate_position_from_angle(112.4, 1.5, 2.0, "z")
        self.assertAlmostEqual(pos_right["pos"], 0.6757, delta=0.01)

    def test_case_1_lead_compensator(self):
        """
        Avanço de Fase: G(s) = 5 / (s(s+1)), ts(2%) = 2 s, Mp = 16% (xi = 0.5)
        -> s = -2 +- 2*sqrt(3)j, z = 2, p = 5.6, K = 4.2 (exato: 4.16).
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "5 / (s * (s + 1))",
            "controller_type": "lead",
            "settling_criterion": "2",
            "ts": 2.0,
            "mp": 16.0,
            "xi": 0.5,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 2.0, places=4)
        self.assertAlmostEqual(params["im"], 2.0 * math.sqrt(3.0), places=3)
        self.assertAlmostEqual(params["z"], 2.0, places=4)
        self.assertAlmostEqual(params["p"], 5.6, places=2)
        self.assertAlmostEqual(params["K"], 4.2, delta=0.06)
        self.assertTrue(result["details"]["stable"])
        self.assertGreaterEqual(len(result["details"]["steps"]), 5)
        json.dumps(result, allow_nan=False)

    def test_case_2_lag_compensator(self):
        """
        Atraso de Fase: G(s) = 1 / ((s+0.5)(s+2)), reduzir e(inf) de 50% em 10 vezes (0.05)
        com K = 1 e p = 0.01 -> z/p = 19, z = 0.19.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / ((s + 0.5) * (s + 2))",
            "controller_type": "lag",
            "input_type": "step",
            "error_reduction_factor": 10.0,
            "lag_pole": 0.01,
            "lag_gain": 1.0,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["e_atual"], 0.5, places=4)
        self.assertAlmostEqual(params["e_novo"], 0.05, places=4)
        self.assertAlmostEqual(params["z_over_p"], 19.0, places=4)
        self.assertAlmostEqual(params["p"], 0.01, places=4)
        self.assertAlmostEqual(params["z"], 0.19, places=4)
        self.assertAlmostEqual(params["K"], 1.0, places=4)
        self.assertTrue(result["details"]["stable"])

    def test_case_3_proportional_example_1(self):
        """
        Proporcional (P) Ex. 1: G(s) = 1 / (s(s+1)(s+2)), ts(2%) = 10 s, Mp <= 10% (xi = 0.591)
        -> s = -0.4 +- 0.55j, Kp = 0.93.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 1) * (s + 2))",
            "controller_type": "P",
            "settling_criterion": "2",
            "ts": 10.0,
            "mp": 10.0,
            "xi": 0.591,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 0.4, places=3)
        self.assertAlmostEqual(params["im"], 0.55, delta=0.01)
        self.assertAlmostEqual(params["kp"], 0.93, delta=0.015)
        self.assertTrue(result["details"]["stable"])

    def test_case_4_proportional_example_2_vertical_lgr(self):
        """
        Proporcional (P) Ex. 2: G(s) = 1 / ((s+1)(s+5)), ts(2%) = 2 s, Mp = 10% (xi = 0.591),
        ramo do LGR em -3 -> s = -3 +- 4.05j, Kp = 20.4.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / ((s + 1) * (s + 5))",
            "controller_type": "P",
            "settling_criterion": "2",
            "ts": 2.0,
            "mp": 10.0,
            "xi": 0.591,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 3.0, places=3)
        self.assertAlmostEqual(params["im"], 4.05, delta=0.05)
        self.assertAlmostEqual(params["kp"], 20.4, delta=0.4)
        self.assertTrue(result["details"]["stable"])

    def test_case_5_pd_example_1(self):
        """
        Proporcional-Derivativo (PD) Ex. 1: G(s) = 40 / (s(s+2)), ts(2%) = 1 s, Mp = 25% (xi = 0.4)
        -> s = -4 +- 9.16j, z = 16.72, td = 0.06, Kp = 2.5.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "40 / (s * (s + 2))",
            "controller_type": "PD",
            "settling_criterion": "2",
            "ts": 1.0,
            "mp": 25.0,
            "xi": 0.4,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 4.0, places=3)
        self.assertAlmostEqual(params["im"], 9.16, delta=0.01)
        self.assertAlmostEqual(params["z"], 16.72, delta=0.1)
        self.assertAlmostEqual(params["td"], 0.06, delta=0.002)
        self.assertAlmostEqual(params["kp"], 2.5, delta=0.02)
        self.assertTrue(result["details"]["stable"])

    def test_case_6_pd_example_2(self):
        """
        Proporcional-Derivativo (PD) Ex. 2: G(s) = 1 / (s(s+4)(s+6)), xi = 0.456, wn = 5 rad/s
        -> s = -2.3 +- 4.45j, z = 5.2, td = 0.19, Kp = 135.53.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 4) * (s + 6))",
            "controller_type": "PD",
            "settling_criterion": "2",
            "xi": 0.456,
            "wn": 5.0,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 2.3, delta=0.03)
        self.assertAlmostEqual(params["im"], 4.45, delta=0.01)
        self.assertAlmostEqual(params["z"], 5.2, delta=0.1)
        self.assertAlmostEqual(params["td"], 0.19, delta=0.005)
        self.assertAlmostEqual(params["kp"], 135.53, delta=1.0)
        self.assertTrue(result["details"]["stable"])

    def test_case_7_pi_example_1(self):
        """
        Proporcional-Integral (PI) Ex. 1: G(s) = 10 / ((s+2)(s+5)), ts(5%) = 1.5 s, Mp = 0.63% (xi = 0.857)
        -> s = -2 +- 1.2j, z = 2.18, Kp = 0.75.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "10 / ((s + 2) * (s + 5))",
            "controller_type": "PI",
            "settling_criterion": "5",
            "ts": 1.5,
            "mp": 0.63,
            "xi": 0.857,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 2.0, places=3)
        self.assertAlmostEqual(params["im"], 1.2, delta=0.01)
        self.assertAlmostEqual(params["z"], 2.18, delta=0.02)
        self.assertAlmostEqual(params["kp"], 0.75, delta=0.01)
        self.assertTrue(result["details"]["stable"])

    def test_case_8_pi_example_2_ramp_double_origin_pole(self):
        """
        Proporcional-Integral (PI) Ex. 2: G(s) = 1 / (s(s+4)), entrada rampa unitária,
        ts(5%) = 2 s, Mp = 10% (xi = 0.6) -> s = -1.5 +- 2j, z = 0.67, Kp ~ 9.24.
        """
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 4))",
            "controller_type": "PI",
            "input_type": "ramp",
            "settling_criterion": "5",
            "ts": 2.0,
            "mp": 10.0,
            "xi": 0.6,
            "points": 250,
        })
        self.assertTrue(result["success"])
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["re"], 1.5, places=3)
        self.assertAlmostEqual(params["im"], 2.0, places=3)
        self.assertAlmostEqual(params["z"], 0.67, delta=0.01)
        self.assertAlmostEqual(params["kp"], 9.24, delta=0.05)
        self.assertAlmostEqual(params["e_novo"], 0.0, places=5)
        self.assertTrue(result["details"]["stable"])

    def test_parabola_input_steady_state_error_and_lag_controller(self):
        """
        Valida entrada parábola unitária R(s) = 1/s³ em sistema Tipo 2:
        G(s) = 2*(s+1) / (s^2*(s+2)*(s+4)) -> Ka = 2/(2*4) = 0.25 -> e_atual(inf) = 4.0.
        Reduzindo em 10x: e_novo(inf) = 0.4 -> z/p = 10 -> p = 0.01, z = 0.1.
        """
        err = evaluate_steady_state_error([2.0, 2.0], [1.0, 6.0, 8.0, 0.0, 0.0], input_type="parabola")
        self.assertEqual(err["input_type"], "parabola")
        self.assertAlmostEqual(err["value"], 4.0, places=5)
        self.assertIn(r"\frac{1}{s^3}", err["formula_latex"])

        result = dispatch({
            "action": "controller_design",
            "plant_expr": "2 * (s + 1) / (s^2 * (s + 2) * (s + 4))",
            "controller_type": "lag",
            "input_type": "parabola",
            "error_reduction_factor": 10.0,
            "lag_pole": 0.01,
            "lag_gain": 1.0,
            "points": 200,
        })
        self.assertTrue(result["success"])
        self.assertEqual(result["details"]["input_type"], "parabola")
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["e_atual"], 4.0, places=4)
        self.assertAlmostEqual(params["e_novo"], 0.4, places=4)
        self.assertAlmostEqual(params["z_over_p"], 10.0, places=4)
        self.assertAlmostEqual(params["p"], 0.01, places=5)
        self.assertAlmostEqual(params["z"], 0.1, places=4)


class ControllerDesignLegacyTests(unittest.TestCase):
    """Mantém cobertura dos métodos clássicos legados (Ziegler-Nichols, Cohen-Coon, CHR)."""

    def test_ziegler_nichols_critical_pid_auto(self):
        result = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical",
            controller_type="PID",
            final_time=12,
            points=300,
        )
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["kcr"], 30.0, places=3)
        self.assertAlmostEqual(params["wcr"], 5**0.5, places=3)
        self.assertAlmostEqual(params["kp"], 18.0, places=3)
        self.assertAlmostEqual(params["ti"], 1.405, places=2)
        self.assertAlmostEqual(params["td"], 0.351, places=2)
        self.assertTrue(result["details"]["stable"])
        self.assertIsNone(result["details"]["warning"])

    def test_ziegler_nichols_critical_pid_manual_unstable(self):
        result = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical_manual",
            controller_type="PID",
            critical_gain=6,
            critical_period=2,
            final_time=12,
            points=300,
        )
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["kp"], 3.6)
        self.assertAlmostEqual(params["ki"], 3.6)
        self.assertAlmostEqual(params["kd"], 0.9)
        self.assertFalse(result["details"]["stable"])
        self.assertEqual(result["details"]["warning"], "Malha fechada instável com estes parâmetros")


if __name__ == "__main__":
    unittest.main()
