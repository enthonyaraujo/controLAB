"""Native formula rendering preserves scientific results without a browser."""
import base64
import io
import unittest

from native_presentation import dispatch_native, render_formulas


class NativePresentationTests(unittest.TestCase):
    def test_formula_is_an_embedded_png_in_both_themes(self):
        from PIL import Image
        for theme in ('light', 'dark'):
            result = render_formulas([r'\frac{1}{s+1}'], theme)
            image = Image.open(io.BytesIO(base64.b64decode(result['images'][0])))
            self.assertEqual(image.format, 'PNG')
            self.assertGreater(image.width, 10)

    def test_ams_arrows_used_by_the_engine_are_typeset(self):
        result = render_formulas([r'\xi = 0{,}5 \implies M_p = 16{,}3\% \iff \text{ok}'])
        self.assertIsNotNone(result['images'][0])

    def test_unsupported_tex_keeps_its_slot_as_none(self):
        result = render_formulas([r'\unknowncommand{G}', r's^{2}', '', 7, 'x' * 5000])
        self.assertTrue(result['success'])
        self.assertIsNone(result['images'][0])
        self.assertIsNotNone(result['images'][1])
        self.assertEqual(result['images'][2:], [None, None, None])

    def test_rejects_non_list_sources(self):
        with self.assertRaises(ValueError):
            render_formulas('s^2')

    def test_scientific_requests_pass_through_unchanged(self):
        reply = {'success': False, 'error': 'entrada inválida'}
        self.assertIs(dispatch_native({'action': 'time_response'}, lambda data: reply), reply)


if __name__ == '__main__':
    unittest.main()
