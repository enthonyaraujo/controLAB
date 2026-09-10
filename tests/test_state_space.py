"""Testes do módulo de espaço de estados."""

import json
import unittest

from python_bridge import dispatch
from state_space import analyze_state_space


class StateSpaceTests(unittest.TestCase):
    def test_structural_analysis_and_ackermann(self):
        result = analyze_state_space(
            a="[[0, 1], [-2, -3]]",
            b="[[0], [1]]",
            c="[[1, 0]]",
            d="[[0]]",
            desired_poles="-4, -5",
            observer_poles="-6, -7",
        )
        self.assertEqual(result["details"]["controllability_rank"], 2)
        self.assertEqual(result["details"]["observability_rank"], 2)
        self.assertEqual(result["details"]["state_feedback_method"], "Fórmula de Ackermann")
        self.assertIn("Luenberger", result["details"]["observer_method"])
        json.dumps(result, allow_nan=False)

    def test_transfer_function_conversion_and_diagonal_form(self):
        result = analyze_state_space(
            mode="transfer",
            expression="1 / (s^2 + 3*s + 2)",
            canonical_form="diagonal",
        )
        self.assertTrue(result["details"]["transfer_function"]["siso"])
        self.assertEqual(result["details"]["canonical"]["label"], "Canônica diagonal")

    def test_bridge_dispatch(self):
        result = dispatch({
            "action": "state_space",
            "mode": "matrices",
            "A": "0, 1; -2, -3",
            "B": "0; 1",
            "C": "1, 0",
            "D": "0",
        })
        self.assertTrue(result["success"])
        self.assertEqual(result["module"], "state_space")


if __name__ == "__main__":
    unittest.main()
