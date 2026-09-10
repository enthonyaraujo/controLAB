"""Testes de sintonia PID e compensadores."""

import json
import unittest

from controller_design import design_controller
from python_bridge import dispatch


class ControllerDesignTests(unittest.TestCase):
    def test_ziegler_nichols_critical_pid(self):
        result = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical",
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
        json.dumps(result, allow_nan=False)

    def test_cohen_coon_and_chr_are_available(self):
        for method in ("cohen_coon", "chr", "zn_reaction"):
            result = design_controller(
                "1 / (4*s + 1)",
                method=method,
                controller_type="PI",
                process_gain=1,
                delay=0.5,
                time_constant=4,
                final_time=20,
                points=220,
            )
            self.assertTrue(result["success"])

    def test_lead_compensator_and_bridge(self):
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 2))",
            "design_type": "lead_lag",
            "compensator_type": "lead",
            "compensator_zero": 1,
            "compensator_pole": 5,
            "compensator_gain": 2,
            "points": 220,
        })
        self.assertTrue(result["success"])
        self.assertEqual(result["module"], "controller_design")
        self.assertIn("avanço", result["details"]["design_label"])


if __name__ == "__main__":
    unittest.main()
