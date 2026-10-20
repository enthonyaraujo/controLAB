import json
import threading
import unittest
from http.server import ThreadingHTTPServer
from urllib.request import Request, urlopen
from urllib.error import HTTPError

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


    def test_cross_origin_script_write_is_rejected(self):
        body = json.dumps({"action": "script_request", "operation": "state"}).encode()
        request = Request(f"{self.base_url}/api", data=body, headers={"Origin": "https://unrelated.example", "Content-Type": "application/json"}, method="POST")
        with self.assertRaises(HTTPError) as error:
            urlopen(request, timeout=5)
        self.assertEqual(error.exception.code, 403)

    def test_dns_rebinding_host_is_rejected(self):
        request = Request(f"{self.base_url}/api", data=b'{"action":"presets"}', headers={"Host": "unrelated.example", "Origin": "http://unrelated.example"}, method="POST")
        with self.assertRaises(HTTPError) as error:
            urlopen(request, timeout=5)
        self.assertEqual(error.exception.code, 403)

    def test_script_session_persists_through_http(self):
        import uuid
        token = str(uuid.uuid4())
        for source, expected in [("A=[1 2;3 4];", ""), ("det(A)", "-2")]:
            body = json.dumps({"action": "script_request", "operation": "execute", "session": token, "source": source}).encode()
            request = Request(f"{self.base_url}/api", data=body, headers={"Content-Type": "application/json", "Origin": self.base_url}, method="POST")
            with urlopen(request, timeout=5) as response:
                result = json.load(response)
                self.assertTrue(result["success"], result)
                if expected:
                    self.assertIn(expected, result["output"])
