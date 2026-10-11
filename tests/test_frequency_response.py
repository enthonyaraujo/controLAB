"""Testes do módulo de resposta em frequência."""

import json
import unittest

from frequency_response import analyze_frequency_response
from python_bridge import dispatch


class FrequencyResponseTests(unittest.TestCase):
    def test_bode_nyquist_and_metrics(self):
        result = analyze_frequency_response("10 / (s * (s + 2) * (s + 5))", omega_max=200, points=350)
        self.assertTrue(result["success"])
        self.assertTrue(result["image"].startswith("data:image/png;base64,"))
        self.assertIn("<svg", result["svg"])
        self.assertIn("nyquist", result["details"])

        details = result["details"]
        # MG = 7 (≈16.9 dB), wcf = sqrt(10) ≈ 3.16 rad/s
        self.assertAlmostEqual(details["gain_margin"], 7.0, places=2)
        self.assertAlmostEqual(details["gain_margin_db"], 16.90, places=1)
        self.assertAlmostEqual(details["phase_crossover_frequency"], 3.16, delta=0.05)
        # MF ≈ 55.6°, wcg ≈ 0.90 rad/s
        self.assertAlmostEqual(details["phase_margin_deg"], 55.6, delta=0.5)
        self.assertAlmostEqual(details["gain_crossover_frequency"], 0.90, delta=0.05)
        # Malha fechada é estável com realimentação unitária
        self.assertTrue(details["stable"])
        self.assertTrue(details["has_origin_pole"])

        # Presença de grupos: Margens e Estimativas no tempo
        headers = [m.get("header") for m in result["metrics"] if "header" in m]
        self.assertIn("Margens", headers)
        self.assertIn("Estimativas no tempo", headers)
        json.dumps(result, allow_nan=False)

    def test_nyquist_relation_for_stable_first_order(self):
        result = analyze_frequency_response("1 / (s + 1)", points=250)
        self.assertEqual(result["details"]["nyquist"], {"Z": 0, "N": 0, "P": 0})

    def test_bridge_dispatch(self):
        result = dispatch({"action": "frequency_response", "expr": "1 / (s + 1)", "points": 220})
        self.assertTrue(result["success"])
        self.assertEqual(result["module"], "frequency_response")


if __name__ == "__main__":
    unittest.main()
