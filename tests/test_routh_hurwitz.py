"""
Testes unitários para o módulo de Análise de Estabilidade por Routh-Hurwitz.
Valida graus 1 a n, casos especiais (epsilon e linha nula), faixa de K e integração com bridge.
"""

import unittest
from routh_hurwitz import analyze_routh_hurwitz, ROUTH_PRESETS
from python_bridge import dispatch


class TestRouthHurwitz(unittest.TestCase):
    def test_degree_1_and_2(self):
        # Grau 1 estavel: s + 5 = 0
        r1_s = analyze_routh_hurwitz("s + 5")
        self.assertEqual(r1_s["degree"], 1)
        self.assertEqual(r1_s["verdict"], "Estável")
        self.assertEqual(r1_s["sign_changes"], 0)

        # Grau 1 instavel: s - 5 = 0
        r1_u = analyze_routh_hurwitz("s - 5")
        self.assertEqual(r1_u["verdict"], "Instável")
        self.assertEqual(r1_u["sign_changes"], 1)

        # Grau 2 estavel: s^2 + 3s + 2 = 0
        r2_s = analyze_routh_hurwitz("s^2 + 3*s + 2")
        self.assertEqual(r2_s["degree"], 2)
        self.assertEqual(r2_s["verdict"], "Estável")
        self.assertEqual(r2_s["sign_changes"], 0)

        # Grau 2 instavel: s^2 - 3s + 2 = 0
        r2_u = analyze_routh_hurwitz("s^2 - 3*s + 2")
        self.assertEqual(r2_u["verdict"], "Instável")
        self.assertEqual(r2_u["sign_changes"], 2)

    def test_degree_3_stable_and_unstable(self):
        # Grau 3 estavel: s^3 + 6s^2 + 11s + 6 = 0
        r3_s = analyze_routh_hurwitz("s^3 + 6*s^2 + 11*s + 6")
        self.assertEqual(r3_s["degree"], 3)
        self.assertEqual(r3_s["verdict"], "Estável")
        self.assertEqual(r3_s["sign_changes"], 0)
        self.assertEqual(r3_s["rhp_poles"], 0)

        # Grau 3 instavel: s^3 + s^2 + 2s + 24 = 0
        r3_u = analyze_routh_hurwitz("s^3 + s^2 + 2*s + 24")
        self.assertEqual(r3_u["degree"], 3)
        self.assertEqual(r3_u["verdict"], "Instável")
        self.assertEqual(r3_u["sign_changes"], 2)
        self.assertEqual(r3_u["rhp_poles"], 2)

    def test_degree_3_with_k_range(self):
        # 1 / (s(s+1)(s+2)) => s^3 + 3s^2 + 2s + K = 0
        res = analyze_routh_hurwitz("s^3 + 3*s^2 + 2*s + K")
        self.assertEqual(res["degree"], 3)
        self.assertEqual(res["verdict"], "Dependente de K")
        self.assertTrue(res["k_range"]["has_k"])
        self.assertIn("0 < K < 6", res["k_range"]["range_latex"])

        # Ponto critico K = 6 e frequencia de oscilacao omega = sqrt(2) approx 1.414 rad/s
        crit = res["k_range"]["critical_k"]
        self.assertTrue(len(crit) > 0)
        self.assertAlmostEqual(crit[0]["k_val"], 6.0, places=1)
        self.assertAlmostEqual(crit[0]["omega_osc"], 1.4142, places=2)

        # Regimes completos de K
        self.assertIn("0 < K < 6", res["k_range"]["stable_latex"])
        self.assertIn("K = 6", res["k_range"]["marginal_latex"])
        self.assertIn("K > 6", res["k_range"]["unstable_latex"])

        # Presença do gráfico do plano s
        self.assertIn("data:image/png;base64", res["plot_image"])
        self.assertIsNotNone(res["plot_svg"])

    def test_degree_4_epsilon_case(self):
        # s^4 + s^3 + 2s^2 + 2s + 3 = 0
        # Linha s^2 tem elemento zero: piv = (1*2 - 1*2)/1 = 0
        res = analyze_routh_hurwitz("s^4 + s^3 + 2*s^2 + 2*s + 3")
        self.assertEqual(res["degree"], 4)
        self.assertEqual(res["verdict"], "Instável")
        self.assertEqual(res["sign_changes"], 2)
        self.assertTrue(any(sc["type"] == "zero_first_element" for sc in res["special_cases"]))

    def test_degree_4_row_of_zeros_case(self):
        # s^4 + 2s^3 + 6s^2 + 8s + 8 = 0
        # Linha s^1 e totalmente nula, aux poly A(s) = 2s^2 + 8 => polos em +-2j
        res = analyze_routh_hurwitz("s^4 + 2*s^3 + 6*s^2 + 8*s + 8")
        self.assertEqual(res["degree"], 4)
        self.assertEqual(res["verdict"], "Marginalmente Estável")
        self.assertEqual(res["sign_changes"], 0)
        self.assertTrue(any(sc["type"] == "row_of_zeros" for sc in res["special_cases"]))

    def test_degree_5_and_high_order(self):
        # Polinomio de grau 5
        res5 = analyze_routh_hurwitz("s^5 + 2*s^4 + 2*s^3 + 4*s^2 + 11*s + 10")
        self.assertEqual(res5["degree"], 5)
        self.assertEqual(res5["verdict"], "Instável")
        self.assertEqual(res5["sign_changes"], 2)

        # Polinomio de grau 6
        res6 = analyze_routh_hurwitz("s^6 + s^5 + 5*s^4 + 3*s^3 + 2*s^2 + 8*s + 4")
        self.assertEqual(res6["degree"], 6)

    def test_transfer_function_rational_input(self):
        # Entrada como fracao G(s) = (s + 2) / (s(s + 1)(s + 4))
        res = analyze_routh_hurwitz("(s + 2) / (s * (s + 1) * (s + 4))")
        self.assertTrue(res["is_fraction"])
        self.assertEqual(res["degree"], 3)
        self.assertTrue(res["k_range"]["has_k"])

    def test_python_bridge_dispatch_routh(self):
        # Testa presets
        presets_resp = dispatch({"action": "routh_presets"})
        self.assertTrue(presets_resp["success"])
        self.assertTrue(len(presets_resp["presets"]) >= 5)

        # Testa calculo via bridge
        calc_resp = dispatch({
            "action": "routh_hurwitz",
            "expr": "s^3 + 6*s^2 + 11*s + 6",
            "theme": "dark",
        })
        self.assertTrue(calc_resp["success"])
        self.assertEqual(calc_resp["verdict"], "Estável")

        # Testa simulador pontual de ganho K via bridge
        k_eval_stable = dispatch({
            "action": "routh_evaluate_k",
            "expr": "s^3 + 3*s^2 + 2*s + K",
            "k_val": 3.0,
        })
        self.assertTrue(k_eval_stable["success"])
        self.assertEqual(k_eval_stable["verdict"], "Estável")
        self.assertEqual(k_eval_stable["spd_count"], 0)

        k_eval_crit = dispatch({
            "action": "routh_evaluate_k",
            "expr": "s^3 + 3*s^2 + 2*s + K",
            "k_val": 6.0,
        })
        self.assertTrue(k_eval_crit["success"])
        self.assertEqual(k_eval_crit["verdict"], "Marginalmente Estável")
        self.assertEqual(k_eval_crit["jw_count"], 2)

        k_eval_unstable = dispatch({
            "action": "routh_evaluate_k",
            "expr": "s^3 + 3*s^2 + 2*s + K",
            "k_val": 10.0,
        })
        self.assertTrue(k_eval_unstable["success"])
        self.assertEqual(k_eval_unstable["verdict"], "Instável")
        self.assertTrue(k_eval_unstable["spd_count"] > 0)


if __name__ == "__main__":
    unittest.main()
