import os
import json
import unittest

os.environ.setdefault("MPLCONFIGDIR", "/tmp/lgr-matplotlib")

import matplotlib.pyplot as plt

from lgr_engine import lgr_completo, parse_tf_expression, parse_tf_parts
from python_bridge import handle_preview
from android_bridge import dispatch_json


class TransferFunctionInputTests(unittest.TestCase):
    def assert_coefficients(self, actual, expected):
        self.assertEqual(len(actual), len(expected))
        for value, wanted in zip(actual, expected):
            self.assertAlmostEqual(value, wanted)

    def test_accepts_label_unicode_powers_and_implicit_multiplication(self):
        num, den, latex_exp, latex_fac = parse_tf_expression(
            "G(s) = (s + 2) ÷ (s³ + 5s² + 4s)"
        )
        self.assert_coefficients(num, [1, 2])
        self.assert_coefficients(den, [1, 5, 4, 0])
        self.assertIn(r"\frac", latex_exp)
        self.assertIn("s \\left(s + 1\\right)", latex_fac)

    def test_accepts_scientific_notation(self):
        num, den, _, _ = parse_tf_expression("1e-3(s + 2)/(s(s + 1))")
        self.assert_coefficients(num, [0.001, 0.002])
        self.assert_coefficients(den, [1, 1, 0])

    def test_accepts_separate_numerator_and_denominator(self):
        num, den, _, latex_fac = parse_tf_parts("s + 2", "s(s + 1)(s + 4)")
        self.assert_coefficients(num, [1, 2])
        self.assert_coefficients(den, [1, 5, 4, 0])
        self.assertIn(r"\frac", latex_fac)

    def test_rejects_names_outside_restricted_grammar(self):
        with self.assertRaisesRegex(ValueError, "símbolos não suportados"):
            parse_tf_expression("__import__('os').system('id')")

    def test_preview_does_not_run_plot(self):
        response = handle_preview(
            {"mode": "parts", "numerator": "s + 2", "denominator": "s(s+1)"}
        )
        self.assertTrue(response["success"])
        self.assert_coefficients(response["den"], [1, 1, 0])

    def test_android_bridge_uses_same_dispatcher(self):
        response = json.loads(
            dispatch_json(
                json.dumps(
                    {
                        "action": "preview",
                        "mode": "expr",
                        "expr": "G(s) = 1/(s(s+1))",
                    }
                )
            )
        )
        self.assertTrue(response["success"])
        self.assert_coefficients(response["den"], [1, 1, 0])


class ClassicalLGRRegressionTests(unittest.TestCase):
    def test_reference_case_keeps_classical_results(self):
        fig, _ax, details = lgr_completo([1, 2], [1, 5, 4, 0], show_plot=False)
        try:
            self.assertEqual(details["P"], 3)
            self.assertEqual(details["Z"], 1)
            self.assertEqual(details["ramos"], 3)
            self.assertAlmostEqual(details["centroide"], -1.5)
            self.assertEqual(
                [round(item["graus"]) for item in details["angulos_assintotas"]],
                [90, 270],
            )
        finally:
            plt.close(fig)


if __name__ == "__main__":
    unittest.main()
