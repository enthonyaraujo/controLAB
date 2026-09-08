import os
import unittest

os.environ.setdefault("MPLCONFIGDIR", "/tmp/lgr-matplotlib")

from lgr_engine import lgr_completo, PRESETS
from python_bridge import handle_calculate


class StepByStepDeductionsTests(unittest.TestCase):
    def test_deductions_structure_present_in_calculate(self):
        payload = {
            "mode": "expr",
            "expr": "(s + 2) / (s * (s + 1) * (s + 4))",
            "title": "Teste Deduções",
        }
        res = handle_calculate(payload)
        self.assertTrue(res["success"])
        det = res["detalhes"]
        self.assertIn("passo_a_passo", det)
        pap = det["passo_a_passo"]

        # Verifica que todos os 5 blocos dos 7 passos estão presentes
        for key in ["passo_1_2_3", "passo_4", "passo_5", "passo_6", "passo_7"]:
            self.assertIn(key, pap)

        # Passo 1, 2 e 3
        p1 = pap["passo_1_2_3"]
        self.assertEqual(p1["P"], 3)
        self.assertEqual(p1["Z"], 1)
        self.assertEqual(p1["ramos"], 3)
        self.assertGreater(len(p1["segmentos_eixo_real"]), 0)

        # Passo 4
        p4 = pap["passo_4"]
        self.assertTrue(p4["tem_assintotas"])
        self.assertEqual(p4["n_assintotas"], 2)
        self.assertIn(r"\sigma_a", p4["substituicao_centroide"])
        self.assertEqual(len(p4["angulos_deduzidos"]), 2)

        # Passo 5
        p5 = pap["passo_5"]
        self.assertIn("condicao", p5)
        self.assertIn("derivada_N", p5)
        self.assertIn("derivada_D", p5)
        self.assertIn("polinomio_break", p5)
        self.assertGreater(len(p5["raizes_analisadas"]), 0)

        # Passo 6
        p6 = pap["passo_6"]
        self.assertIn("eq_caracteristica", p6)
        self.assertIn("parte_real_eq", p6)
        self.assertIn("parte_imaginaria_eq", p6)

    def test_complex_poles_departure_angles(self):
        # Exemplo 2: Polos complexos conjugados
        p = PRESETS["Exemplo 2: Polos Complexos Conjugados e Ângulo de Partida"]
        _, _, det = lgr_completo(p["num"], p["den"], show_plot=False)
        pap = det["passo_a_passo"]
        p7 = pap["passo_7"]
        self.assertTrue(p7["tem_partida"])
        self.assertGreater(len(p7["deducoes_partida"]), 0)
        partida = p7["deducoes_partida"][0]
        self.assertIn("calculo_substituicao", partida)
        self.assertIn(r"\theta_d", partida["formula_aplicada"])


if __name__ == "__main__":
    unittest.main()
