"""Testes de sintonia PID e compensadores."""

import json
import unittest

from controller_design import design_controller
from python_bridge import dispatch


class ControllerDesignTests(unittest.TestCase):
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
        self.assertIn("frequency_margins_after", result["details"])
        json.dumps(result, allow_nan=False)

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

    def test_ziegler_nichols_critical_pd_auto(self):
        result = design_controller(
            "1 / (s * (s + 1) * (s + 5))",
            method="zn_critical",
            controller_type="PD",
            final_time=12,
            points=300,
        )
        params = result["details"]["controller_parameters"]
        self.assertAlmostEqual(params["kcr"], 30.0, places=3)
        self.assertAlmostEqual(params["wcr"], 5**0.5, places=3)
        self.assertAlmostEqual(params["kp"], 24.0, places=3)
        self.assertIsNone(params["ti"])
        self.assertEqual(params["ki"], 0.0)
        self.assertAlmostEqual(params["td"], 0.351, places=2)
        self.assertAlmostEqual(params["kd"], 24.0 * params["td"], places=3)
        self.assertTrue(result["details"]["stable"])
        self.assertIn("PD por", result["details"]["design_label"])
        json.dumps(result, allow_nan=False)

    def test_pd_all_tuning_methods(self):
        for method in ("zn_reaction", "zn_critical_manual", "cohen_coon", "chr"):
            result = design_controller(
                "1 / (4*s + 1)",
                method=method,
                controller_type="PD",
                process_gain=1,
                delay=0.5,
                time_constant=4,
                critical_gain=6,
                critical_period=2,
                final_time=20,
                points=220,
            )
            self.assertTrue(result["success"])
            params = result["details"]["controller_parameters"]
            self.assertGreater(params["kp"], 0.0)
            self.assertIsNone(params["ti"])
            self.assertEqual(params["ki"], 0.0)
            self.assertGreater(params["td"], 0.0)
            self.assertGreater(params["kd"], 0.0)
            self.assertIn("PD por", result["details"]["design_label"])
            json.dumps(result, allow_nan=False)

    def test_pd_dispatch_via_python_bridge(self):
        result = dispatch({
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 1) * (s + 5))",
            "design_type": "pid",
            "method": "zn_critical",
            "controller_type": "PD",
            "points": 200,
        })
        self.assertTrue(result["success"])
        self.assertEqual(result["module"], "controller_design")
        self.assertIn("PD por", result["details"]["design_label"])


if __name__ == "__main__":
    unittest.main()
