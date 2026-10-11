"""Testes do módulo de resposta no domínio do tempo."""

import json
import unittest

from python_bridge import dispatch
from time_response import analyze_time_response


class TimeResponseTests(unittest.TestCase):
    def test_second_order_metrics_and_graph(self):
        result = analyze_time_response("4 / (s^2 + 2*s + 4)", final_time=10, points=1000)
        self.assertTrue(result["success"])
        self.assertTrue(result["details"]["stable"])
        # Para G(s) = 4 / (s^2 + 2s + 4): final_value=1.0, zeta=0.5, wn=2.0 rad/s,
        # sobressinal ~16.3%, tempo de pico ~1.81 s, acomodação (2%) ~4.0 s
        step = result["details"]["step"]
        self.assertAlmostEqual(step["final_value"], 1.0, places=2)
        self.assertAlmostEqual(step["damping_ratio"], 0.5, places=2)
        self.assertAlmostEqual(step["natural_frequency"], 2.0, places=2)
        self.assertAlmostEqual(step["overshoot_percent"], 16.3, delta=0.5)
        self.assertAlmostEqual(step["peak_time"], 1.81, delta=0.05)
        self.assertAlmostEqual(step["settling_time"], 4.0, delta=0.2)
        self.assertTrue(result["image"].startswith("data:image/png;base64,"))
        self.assertIn("<svg", result["svg"])

    def test_higher_order_omits_zeta_wn(self):
        result = analyze_time_response("10 / (s*(s+2)*(s+5))", final_time=10, points=400)
        self.assertTrue(result["success"])
        # Ordem 3: zeta e wn devem ser None (omitidos)
        self.assertIsNone(result["details"]["step"]["damping_ratio"])
        self.assertIsNone(result["details"]["step"]["natural_frequency"])

    def test_improper_system_raises_error(self):
        with self.assertRaises(ValueError):
            analyze_time_response("2s")

    def test_static_error_constants_for_type_one(self):
        result = analyze_time_response("5 / (s * (s + 2))", final_time=8, points=300)
        error = result["details"]["steady_state"]
        self.assertEqual(error["type"], 1)
        self.assertEqual(error["ess_step"], 0.0)
        self.assertAlmostEqual(error["kv"], 2.5, places=5)
        self.assertAlmostEqual(error["ess_ramp"], 0.4, places=5)
        json.dumps(result, allow_nan=False)

    def test_bridge_dispatch(self):
        result = dispatch({
            "action": "time_response",
            "expr": "1 / (s + 1)",
            "final_time": 5,
            "points": 250,
        })
        self.assertTrue(result["success"])
        self.assertEqual(result["module"], "time_response")


if __name__ == "__main__":
    unittest.main()
