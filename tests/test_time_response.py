"""Testes do módulo de resposta no domínio do tempo."""

import unittest

from python_bridge import dispatch
from time_response import analyze_time_response


class TimeResponseTests(unittest.TestCase):
    def test_second_order_metrics_and_graph(self):
        result = analyze_time_response("4 / (s^2 + 2*s + 4)", final_time=10, points=400)
        self.assertTrue(result["success"])
        self.assertTrue(result["details"]["stable"])
        # A resposta é calculada para T(s) = G(s)/(1 + G(s)); seus polos são
        # as raízes de s² + 2s + 8, logo zeta = 1/sqrt(8).
        self.assertAlmostEqual(result["details"]["step"]["damping_ratio"], 1 / (8**0.5), places=2)
        self.assertGreater(result["details"]["step"]["overshoot_percent"], 10)
        self.assertTrue(result["image"].startswith("data:image/png;base64,"))
        self.assertIn("<svg", result["svg"])

    def test_static_error_constants_for_type_one(self):
        result = analyze_time_response("5 / (s * (s + 2))", final_time=8, points=300)
        error = result["details"]["steady_state"]
        self.assertEqual(error["type"], 1)
        self.assertEqual(error["ess_step"], 0.0)
        self.assertAlmostEqual(error["kv"], 2.5, places=5)
        self.assertAlmostEqual(error["ess_ramp"], 0.4, places=5)

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
