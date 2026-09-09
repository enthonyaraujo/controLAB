"""Servidor local responsivo do ControLAB para desktop, tablet e celular."""

import argparse
import json
import mimetypes
from http import HTTPStatus
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from python_bridge import dispatch


RENDERER_DIR = Path(__file__).resolve().parent / "src" / "renderer"
MAX_REQUEST_BYTES = 1_000_000

mimetypes.add_type("application/manifest+json", ".webmanifest")
mimetypes.add_type("image/svg+xml", ".svg")
mimetypes.add_type("font/woff2", ".woff2")


class LGRRequestHandler(SimpleHTTPRequestHandler):
    """Entrega o frontend compartilhado e uma API JSON de mesma origem."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(RENDERER_DIR), **kwargs)

    def end_headers(self):
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "no-referrer")
        self.send_header("X-Frame-Options", "DENY")
        if self.path != "/api" and Path(self.path.split("?", 1)[0]).suffix in {
            "",
            ".html",
            ".js",
            ".css",
        }:
            self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def do_POST(self):
        if self.path != "/api":
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        try:
            length = int(self.headers.get("Content-Length", "0"))
            if length <= 0 or length > MAX_REQUEST_BYTES:
                raise ValueError("Tamanho de requisição inválido.")
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            response = dispatch(payload)
            self._send_json(HTTPStatus.OK, response)
        except (ValueError, json.JSONDecodeError) as exc:
            self._send_json(
                HTTPStatus.BAD_REQUEST,
                {"success": False, "error": str(exc)},
            )
        except Exception as exc:  # mantém a API previsível sem expor traceback
            self._send_json(
                HTTPStatus.INTERNAL_SERVER_ERROR,
                {"success": False, "error": str(exc)},
            )

    def _send_json(self, status, payload):
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format_string, *args):
        print(f"[web] {self.address_string()} - {format_string % args}")


def parse_args():
    parser = argparse.ArgumentParser(
        description="Executa o ControLAB no navegador sem gerar build.",
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Use 0.0.0.0 para permitir acesso de outros dispositivos na rede local.",
    )
    parser.add_argument("--port", type=int, default=8501)
    return parser.parse_args()


def main():
    args = parse_args()
    server = ThreadingHTTPServer((args.host, args.port), LGRRequestHandler)
    visible_host = "<IP-DESTE-COMPUTADOR>" if args.host == "0.0.0.0" else args.host
    print(f"ControLAB disponível em http://{visible_host}:{args.port}")
    if args.host == "0.0.0.0":
        print("Mantenha computador e dispositivo móvel na mesma rede local.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServidor encerrado.")
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
