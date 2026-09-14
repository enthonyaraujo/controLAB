"""Regressões de geometria, precisão e custo das prévias do ControLAB."""
import json
import os
import subprocess
import sys
import unittest
from unittest.mock import patch

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.patches import Arc

from lgr_engine import lgr_completo
from python_bridge import handle_calculate


class LgrGeometryTests(unittest.TestCase):
    def tearDown(self):
        plt.close('all')

    def test_equal_units_and_symmetric_limits(self):
        fig, ax, _ = lgr_completo([1], [1, 2, 2, 0])
        fig.canvas.draw()
        origin, horizontal, vertical = ax.transData.transform([(0, 0), (1, 0), (0, 1)])
        self.assertAlmostEqual(np.linalg.norm(horizontal-origin), np.linalg.norm(vertical-origin))
        self.assertAlmostEqual(ax.get_ylim()[0], -ax.get_ylim()[1])

    def test_departure_and_arrival_arcs_are_mirrored(self):
        for num, den in [([1], [1, 2, 2, 0]), ([1, 6, 12], [1, 4, 4, 0])]:
            with self.subTest(num=num):
                _, ax, _ = lgr_completo(num, den)
                arcs = [p for p in ax.patches if isinstance(p, Arc)]
                self.assertEqual(len(arcs), 2)
                upper, lower = arcs
                self.assertAlmostEqual(upper.center[0], lower.center[0])
                self.assertAlmostEqual(upper.center[1], -lower.center[1])
                self.assertAlmostEqual(upper.theta1, -lower.theta2)
                self.assertAlmostEqual(upper.theta2, -lower.theta1)

    def test_angles_match_roots_near_singularities(self):
        for num, den, kind, gain in [([1], [1, 2, 2, 0], 'partida', 1e-5),
                                     ([1, 6, 12], [1, 4, 4, 0], 'chegada', 1e7)]:
            _, _, details = lgr_completo(num, den)
            item = details['angulos_' + kind][0]
            point = item['polo' if kind == 'partida' else 'zero']
            roots = np.roots(np.polyadd(den, np.multiply(num, gain)))
            nearest = min(roots, key=lambda root: abs(root-point))
            actual = np.angle(nearest-point, deg=True)
            error = (actual-item['angulo']+180) % 360-180
            self.assertAlmostEqual(error, 0, delta=0.01)

    def test_imaginary_crossing_keeps_reference_gain(self):
        _, _, details = lgr_completo([1], [1, 3, 2, 0])
        crossings = details['jw_cruzamentos']
        self.assertEqual(len(crossings), 2)
        for item in crossings:
            self.assertAlmostEqual(item['K'], 6, delta=1e-4)
            self.assertAlmostEqual(abs(item['w']), np.sqrt(2), delta=1e-4)

    def test_preview_does_not_import_graphics_or_other_modules(self):
        code = """import sys,json,os
from python_bridge import handle_preview
handle_preview({'expr':'1/(s*(s+1))'})
assert os.environ['MPLBACKEND'] == 'Agg'
print(json.dumps([name for name in ('matplotlib.pyplot','control','routh_hurwitz','time_response','state_space') if name in sys.modules]))
"""
        result = subprocess.run([sys.executable, '-c', code], capture_output=True, text=True, check=True,
                                env={**os.environ, 'MPLBACKEND': 'TkAgg'})
        self.assertEqual(json.loads(result.stdout), [])

    def test_export_failure_releases_figure(self):
        before = plt.get_fignums()
        with patch('matplotlib.figure.Figure.savefig', side_effect=OSError('export failed')):
            with self.assertRaises(OSError):
                handle_calculate({'expr': '1/(s+1)'})
        self.assertEqual(plt.get_fignums(), before)

    def test_direct_expression_rejects_executable_code(self):
        with self.assertRaises(ValueError):
            lgr_completo("__import__('os')")
