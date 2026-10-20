"""Persistent transport to the copied openMAT runtime (desktop and local Web)."""
import atexit
import json
from pathlib import Path
import queue
import subprocess
import sys
import threading


class ScriptService:
    def __init__(self, executable=None):
        self.executable = executable
        self.process = None
        self.replies = None
        self.lock = threading.Lock()

    def runtime_path(self):
        if self.executable:
            return Path(self.executable)
        name = "controlab-runtime.exe" if sys.platform == "win32" else "controlab-runtime"
        if getattr(sys, "frozen", False):
            return Path(sys.executable).parent / name
        return Path(__file__).resolve().parents[2] / "build" / "runtime" / name

    def _start(self):
        executable = self.runtime_path()
        if not executable.is_file():
            raise RuntimeError("Motor .m indisponível nesta instalação. No desenvolvimento, execute npm run build:openmat.")
        self.process = subprocess.Popen(
            [str(executable)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, encoding="utf-8",
            creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0,
        )
        self.replies = queue.Queue(maxsize=1)
        process, replies = self.process, self.replies

        def read_replies():
            try:
                while True:
                    line = process.stdout.readline(16 * 1024 * 1024 + 1)
                    if not line or len(line) > 16 * 1024 * 1024:
                        replies.put(None)
                        break
                    replies.put(line)
            except (OSError, ValueError):
                try:
                    replies.put_nowait(None)
                except queue.Full:
                    pass
        threading.Thread(target=read_replies, daemon=True).start()

    def request(self, data):
        with self.lock:
            # Never silently reset variables after a crash. Let the caller acknowledge it.
            if self.process is not None and self.process.poll() is not None:
                self.close()
                return {"success": False, "error": "O motor .m foi encerrado e a sessão foi perdida. Execute novamente para iniciar uma nova sessão."}
            try:
                if self.process is None:
                    self._start()
                body = json.dumps(data, ensure_ascii=False) + "\n"
                if len(body.encode("utf-8")) > 8 * 1024 * 1024:
                    raise ValueError("Requisição de script muito grande.")
                self.process.stdin.write(body)
                self.process.stdin.flush()
                line = self.replies.get(timeout=12)
                if line is None:
                    raise RuntimeError("O motor .m encerrou a sessão inesperadamente.")
                return json.loads(line)
            except (OSError, ValueError, RuntimeError, queue.Empty) as error:
                self.close()
                return {"success": False, "error": str(error) or "Tempo de execução excedido. A sessão foi reiniciada."}

    def close(self):
        process, self.process = self.process, None
        if process is not None:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=3)
            for stream in (process.stdin, process.stdout):
                if stream:
                    stream.close()


_service = ScriptService()
atexit.register(_service.close)


def dispatch_script(data):
    request = {key: value for key, value in data.items() if key != "action"}
    if request.get("operation") == "list" and not request.get("path"):
        # Match the application's existing default workspace.
        from python_bridge import get_default_workspace
        request["path"] = get_default_workspace()
    return _service.request(request)


def main():
    for line in sys.stdin:
        try:
            data = json.loads(line)
            response = dispatch_script(data)
        except Exception as error:
            response = {"success": False, "error": str(error)}
        print(json.dumps(response, ensure_ascii=False), flush=True)


if __name__ == "__main__":
    main()
