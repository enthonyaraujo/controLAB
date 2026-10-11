"""Testes de compatibilidade tipográfica e prevenção do KeyError 500 no Android."""

import json
import unittest
import matplotlib.pyplot as plt
from matplotlib import font_manager as fm

import plot_theme
from android_bridge import dispatch_json
from control_utils import figure_payload


class TestFontCompatAndKeyError(unittest.TestCase):
    def test_weight_dict_numeric_and_string_safety(self):
        """Garante que fm.weight_dict nunca lance KeyError para pesos numéricos."""
        test_weights = [100, 200, 300, 400, 500, 600, 700, 800, 900, "100", "500", "medium", "bold", "normal"]
        for w in test_weights:
            try:
                resolved = fm.weight_dict[w]
                self.assertIsInstance(resolved, (int, float))
            except KeyError:
                self.fail(f"KeyError disparado para o peso: {w!r}")

    def test_apply_plot_theme_axis_labels_weight(self):
        """Garante que os rótulos de eixos tenham peso 'medium' compatível."""
        fig, ax = plt.subplots()
        plot_theme.apply_plot_theme(fig, ax, theme="dark")
        self.assertEqual(ax.xaxis.label.get_fontweight(), "medium")
        self.assertEqual(ax.yaxis.label.get_fontweight(), "medium")
        plt.close(fig)

    def test_svg_export_plain_text_labels(self):
        """Simula a exportação de SVG com textos puros sem caracteres matemáticos ($)."""
        fig, ax = plt.subplots()
        ax.set_xlabel("Tempo (s)")
        ax.set_ylabel("y(t)")
        plot_theme.apply_plot_theme(fig, ax, theme="dark")

        # Não deve lançar KeyError: 500 ao gerar SVG
        img, svg = figure_payload(fig, dpi=140, theme="dark")
        self.assertTrue(img.startswith("data:image/png;base64,"))
        self.assertIn("<svg", svg)
        self.assertIn("Tempo (s)", svg)

    def test_android_dispatch_time_response(self):
        """Valida que a bridge Android execute resposta no tempo com sucesso."""
        payload = {
            "action": "time_response",
            "expr": "4 / (s^2 + 2*s + 4)",
            "final_time": 10,
            "settling_threshold": 0.02,
            "theme": "dark",
            "response_type": "step",
            "width": 500,
            "dpi": 200,
        }
        res_raw = dispatch_json(json.dumps(payload))
        res = json.loads(res_raw)
        self.assertTrue(res.get("success"), f"Falha no módulo de tempo: {res.get('error')}")
        self.assertIn("image", res)
        self.assertIn("svg", res)

    def test_android_dispatch_frequency_response(self):
        """Valida que a bridge Android execute resposta em frequência com sucesso."""
        payload = {
            "action": "frequency_response",
            "expr": "10 / (s * (s + 2) * (s + 5))",
            "omega_min": 0.01,
            "omega_max": 100,
            "theme": "dark",
            "width": 500,
            "dpi": 200,
        }
        res_raw = dispatch_json(json.dumps(payload))
        res = json.loads(res_raw)
        self.assertTrue(res.get("success"), f"Falha no módulo de frequência: {res.get('error')}")
        self.assertIn("image", res)
        self.assertIn("svg", res)

    def test_android_dispatch_controller_design(self):
        """Valida que a bridge Android execute projeto de controladores com sucesso."""
        payload = {
            "action": "controller_design",
            "plant_expr": "1 / (s * (s + 1) * (s + 5))",
            "design_type": "pid",
            "method": "zn_critical",
            "controller_type": "PID",
            "theme": "dark",
            "final_time": 20,
            "width": 500,
            "dpi": 200,
        }
        res_raw = dispatch_json(json.dumps(payload))
        res = json.loads(res_raw)
        self.assertTrue(res.get("success"), f"Falha no módulo de controladores: {res.get('error')}")
        self.assertIn("image", res)
        self.assertIn("svg", res)

    def test_android_dispatch_state_space(self):
        """Valida que a bridge Android execute espaço de estados com sucesso."""
        payload = {
            "action": "state_space",
            "mode": "matrices",
            "A": "[[0, 1], [-2, -3]]",
            "B": "[[0], [1]]",
            "C": "[[1, 0]]",
            "D": "[[0]]",
            "canonical_form": "controllable",
            "desired_poles": "-4, -5",
            "theme": "dark",
            "width": 500,
            "dpi": 200,
        }
        res_raw = dispatch_json(json.dumps(payload))
        res = json.loads(res_raw)
        self.assertTrue(res.get("success"), f"Falha no módulo de espaço de estados: {res.get('error')}")
        self.assertIn("image", res)
        self.assertIn("svg", res)


if __name__ == "__main__":
    unittest.main()
