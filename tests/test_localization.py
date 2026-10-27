"""Language isolation and scientific exports, without changing numerical results."""
import unittest
from localization import localized_request, translate
from python_bridge import dispatch


class LocalizationTests(unittest.TestCase):
    def test_default_and_invalid_language_remain_portuguese(self):
        @localized_request
        def response(data):
            return {"title": "Estável", "value": "Ganho", "name": "Configurações.m"}
        self.assertEqual(response({})["title"], "Estável")
        self.assertEqual(response({"language": "xx"})["title"], "Estável")
        english = response({"language": "en"})
        self.assertEqual(english["title"], "Stable")
        self.assertEqual(english["value"], "Ganho")
        self.assertEqual(english["name"], "Configurações.m")
        self.assertEqual(translate("Estável"), "Estável")

    def test_errors_are_translated_and_context_is_reset(self):
        @localized_request
        def failure(data):
            raise ValueError("O denominador não pode ser zero.")
        with self.assertRaisesRegex(ValueError, 'The denominator cannot be zero'):
            failure({"language": "en"})
        self.assertEqual(translate("Estável"), "Estável")

    def test_time_response_export_language_and_numeric_equivalence(self):
        payload = {"action": "time_response", "expr": "4/(s^2+2*s+4)", "points": 200}
        english = dispatch({**payload, "language": "en"})
        portuguese = dispatch(payload)
        self.assertTrue(english["success"])
        self.assertEqual([m["value"] for m in english["metrics"][1:]], [m["value"] for m in portuguese["metrics"][1:]])
        self.assertIn('Time (s)', english['svg'])
        self.assertNotIn('Tempo (s)', english['svg'])
        self.assertIn('Tempo (s)', portuguese['svg'])
