import json
import threading
import unittest
from http.server import ThreadingHTTPServer
from urllib.request import Request, urlopen

from web_server import LGRRequestHandler


class WebServerIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), LGRRequestHandler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.base_url = f"http://127.0.0.1:{cls.server.server_port}"

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join(timeout=2)

    def test_current_hub_is_served_without_stale_cache_policy(self):
        with urlopen(f"{self.base_url}/", timeout=5) as response:
            html = response.read().decode("utf-8")
            self.assertEqual(response.status, 200)
            self.assertEqual(response.headers.get("Cache-Control"), "no-cache")
            self.assertIn('id="page-home"', html)
            self.assertIn('src="navigation.js"', html)

    def test_preview_reaches_python_backend(self):
        body = json.dumps(
            {"action": "preview", "mode": "expr", "expr": "1/(s(s+1))"}
        ).encode("utf-8")
        request = Request(
            f"{self.base_url}/api",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urlopen(request, timeout=10) as response:
            payload = json.loads(response.read().decode("utf-8"))
            self.assertEqual(response.status, 200)
            self.assertTrue(payload["success"])
            self.assertEqual(payload["den"], [1.0, 1.0, 0.0])
