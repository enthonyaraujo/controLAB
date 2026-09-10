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
        self.assertGreaterEqual(len(result["metrics"]), 8)
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
