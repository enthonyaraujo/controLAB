"""Bridge JSON compartilhada pelas interfaces Electron e Web do ControLAB."""

import base64
import io
import json
import os
import sys
import tempfile
import warnings

warnings.filterwarnings("ignore")

# A ponte JSON nunca precisa de uma janela gráfica, inclusive em subprocessos.
os.environ["MPLBACKEND"] = "Agg"

import numpy as np

from lgr_engine import (
    PRESETS,
    format_complex,
    format_transfer_function,
    lgr_completo,
    parse_coeffs_str,
    parse_tf_expression,
    parse_tf_parts,
    parse_zpk,
)

_CONSOLE_WORKSPACE = {
    "G": {"type": "tf", "value": "4/(s^2 + 2*s + 4)"},
    "C": {"type": "pid", "value": "Kp=18, Ti=1.405, Td=0.351"},
    "t": {"type": "vetor", "value": "array([0.0, ..., 10.0])"},
    "y": {"type": "vetor", "value": "array([0.0, ..., 1.0])"},
}

_RECENT_ITEMS = []
_CURRENT_WORKSPACE_FOLDER = None


def get_default_workspace():
    """Retorna e garante a existência da pasta padrão do ControLAB conforme a plataforma."""
    # Android (armazenamento interno /ControLAB/)
    if "ANDROID_ROOT" in os.environ or "ANDROID_DATA" in os.environ or os.path.exists("/sdcard"):
        for candidate in ["/sdcard/ControLAB", "/storage/emulated/0/ControLAB"]:
            try:
                os.makedirs(candidate, exist_ok=True)
                return candidate
            except Exception:
                pass
        fallback = os.path.join(tempfile.gettempdir(), "ControLAB")
        os.makedirs(fallback, exist_ok=True)
        return fallback

    # Windows e Linux (Documentos/ControLAB)
    home = os.path.expanduser("~")
    doc_pt = os.path.join(home, "Documentos", "ControLAB")
    doc_en = os.path.join(home, "Documents", "ControLAB")

    if os.path.isdir(doc_pt):
        target = doc_pt
    elif os.path.isdir(doc_en):
        target = doc_en
    elif os.path.isdir(os.path.join(home, "Documentos")):
        target = doc_pt
    else:
        target = doc_pt

    try:
        os.makedirs(target, exist_ok=True)
    except Exception:
        pass
    return target


def format_display_path(path_str):
    """Formata caminho para exibição amigável (~/Documentos/ControLAB ou /ControLAB)."""
    if not path_str:
        return ""
    home = os.path.expanduser("~")
    norm = os.path.normpath(path_str)
    if norm.startswith(home):
        return "~" + norm[len(home):]
    if "sdcard" in norm or "storage/emulated" in norm:
        return "/ControLAB"
    return norm


def list_workspace_files(folder_path):
    """Lista arquivos reais da pasta de trabalho atual classificando por extensão/tipo."""
    if not folder_path or not os.path.isdir(folder_path):
        return []

    files = []
    try:
        for entry in sorted(os.listdir(folder_path)):
            if entry.startswith(".") or entry.endswith("~") or entry.endswith(".pyc"):
                continue
            full_path = os.path.join(folder_path, entry)
            if os.path.isfile(full_path):
                ext = os.path.splitext(entry)[1].lower()
                file_type = "arquivo"
                if ext == ".m":
                    try:
                        with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                            header = f.read(512).lower()
                            file_type = "função" if "function" in header else "script"
                    except Exception:
                        file_type = "script"
                elif ext == ".mat":
                    file_type = "dados"
                elif ext == ".blk":
                    file_type = "blocos"
                elif ext in (".csv", ".txt", ".dat", ".json"):
                    file_type = "dados"
                elif ext == ".py":
                    file_type = "python"
                elif ext in (".slx", ".mdl", ".sim"):
                    file_type = "modelo"
                files.append({
                    "name": entry,
                    "type": file_type,
                    "path": full_path,
                })
    except Exception:
        pass
    return files


def _format_workspace():
    return [
        {"name": k, "type": v["type"], "value": v["value"]}
        for k, v in _CONSOLE_WORKSPACE.items()
    ]


def _eval_console_command(cmd):
    import math
    import sympy as sp

    env = {
        "np": np,
        "sp": sp,
        "math": math,
        "s": sp.Symbol("s"),
    }
    for k, v in _CONSOLE_WORKSPACE.items():
        if v["type"] == "tf":
            try:
                env[k] = sp.sympify(v["value"].replace("^", "**"))
            except Exception:
                env[k] = v["value"]

    if "=" in cmd and not cmd.strip().startswith("=="):
        parts = cmd.split("=", 1)
        var_name = parts[0].strip()
        expr_str = parts[1].strip()
        if var_name.isidentifier():
            try:
                val = eval(expr_str.replace("^", "**"), {"__builtins__": {}}, env)
                val_type = "escalar"
                if isinstance(val, (np.ndarray, list)):
                    val_type = "vetor"
                elif hasattr(val, "free_symbols") or "s" in expr_str:
                    val_type = "tf"
                _CONSOLE_WORKSPACE[var_name] = {
                    "type": val_type,
                    "value": str(val),
                }
                return f"{var_name} = {val}", None
            except Exception as e:
                try:
                    val = sp.sympify(expr_str.replace("^", "**"))
                    _CONSOLE_WORKSPACE[var_name] = {
                        "type": "tf",
                        "value": str(val),
                    }
                    return f"{var_name} = {val}", None
                except Exception:
                    return None, str(e)

    try:
        val = eval(cmd.replace("^", "**"), {"__builtins__": {}}, env)
        return f"ans = {val}", None
    except Exception:
        try:
            val = sp.sympify(cmd.replace("^", "**"))
            return f"ans = {val}", None
        except Exception as e:
            return None, str(e)



def get_transfer_function(payload):
    """Converte qualquer modo de entrada no formato usado pelo motor científico."""
    mode = payload.get("mode", "expr")

    if mode == "expr":
        return parse_tf_expression(payload.get("expr", "").strip())

    if mode == "parts":
        return parse_tf_parts(
            payload.get("numerator", "1").strip(),
            payload.get("denominator", "s(s + 1)").strip(),
        )

    if mode == "coeffs":
        num = parse_coeffs_str(payload.get("num", "1, 2"))
        den = parse_coeffs_str(payload.get("den", "1, 5, 4, 0"))
        return format_transfer_function(num, den)

    if mode == "zpk":
        num, den = parse_zpk(
            payload.get("zeros", ""),
            payload.get("poles", ""),
            float(payload.get("k", 1.0)),
        )
        return format_transfer_function(num, den)

    if mode == "preset":
        preset_key = payload.get("preset_key")
        if preset_key not in PRESETS:
            preset_key = next(iter(PRESETS))
        preset = PRESETS[preset_key]
        return format_transfer_function(preset["num"], preset["den"])

    raise ValueError(f"Modo desconhecido: {mode}")


def get_presets_response():
    return {
        "success": True,
        "presets": [
            {
                "id": key,
                "title": key,
                "expr": value["expr"],
                "num": value["num"],
                "den": value["den"],
                "desc": value["desc"],
            }
            for key, value in PRESETS.items()
        ],
    }


def handle_preview(payload):
    """Valida a entrada e devolve LaTeX sem executar o traçado do LGR."""
    num, den, latex_exp, latex_fac = get_transfer_function(payload)
    return {
        "success": True,
        "latex_exp": latex_exp,
        "latex_fac": latex_fac,
        "num": num,
        "den": den,
    }


def handle_calculate(payload):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    num, den, latex_exp, latex_fac = get_transfer_function(payload)
    title = payload.get("title", "Lugar Geométrico das Raízes")

    theme = payload.get("theme", "dark")
    fig, _ax, detalhes = lgr_completo(num, den, titulo=title, show_plot=False, theme=theme)

    from control_utils import figure_payload
    img_base64, svg_text = figure_payload(fig, dpi=160, theme=theme)

    polos_serializados = [
        {"re": float(np.real(p)), "im": float(np.imag(p)), "str": format_complex(p)}
        for p in detalhes["polos"]
    ]
    zeros_serializados = [
        {"re": float(np.real(z)), "im": float(np.imag(z)), "str": format_complex(z)}
        for z in detalhes["zeros"]
    ]
    angulos_partida_serializados = [
        {"polo_str": format_complex(ap["polo"]), "angulo": float(ap["angulo"])}
        for ap in detalhes.get("angulos_partida", [])
    ]
    angulos_chegada_serializados = [
        {"zero_str": format_complex(ac["zero"]), "angulo": float(ac["angulo"])}
        for ac in detalhes.get("angulos_chegada", [])
    ]

    return {
        "success": True,
        "image": img_base64,
        "svg": svg_text,
        "latex_exp": latex_exp,
        "latex_fac": latex_fac,
        "detalhes": {
            "P": detalhes["P"],
            "Z": detalhes["Z"],
            "ramos": detalhes["ramos"],
            "polos": polos_serializados,
            "zeros": zeros_serializados,
            "centroide": (
                float(detalhes["centroide"])
                if detalhes["centroide"] is not None
                else None
            ),
            "angulos_assintotas": detalhes["angulos_assintotas"],
            "break_points": detalhes["break_points"],
            "jw_cruzamentos": detalhes["jw_cruzamentos"],
            "angulos_partida": angulos_partida_serializados,
            "angulos_chegada": angulos_chegada_serializados,
            "passo_a_passo": detalhes.get("passo_a_passo", {}),
        },
    }


def dispatch(data):
    action = data.get("action", "calculate")
    theme = data.get("theme", "dark")
    if action == "presets":
        return get_presets_response()
    if action == "preview":
        return handle_preview(data)
    if action == "calculate":
        return handle_calculate(data)
    if action == "routh_presets":
        from routh_hurwitz import ROUTH_PRESETS
        return {
            "success": True,
            "presets": [
                {
                    "id": key,
                    "title": key,
                    "expr": val["expr"],
                    "desc": val["desc"],
                }
                for key, val in ROUTH_PRESETS.items()
            ],
        }
    if action == "routh_hurwitz":
        from routh_hurwitz import analyze_routh_hurwitz
        expr = data.get("expr") or data.get("input") or data.get("den") or "s^3 + 2s^2 + s + 1"
        has_k = data.get("has_k", True)
        return analyze_routh_hurwitz(expr, has_k_loop=has_k, theme=theme)
    if action == "routh_evaluate_k":
        from routh_hurwitz import evaluate_k_point
        expr = data.get("expr") or "s^3 + 3s^2 + 2s + K"
        k_val = float(data.get("k_val", 1.0))
        return evaluate_k_point(expr, k_val)
    if action == "time_response_presets":
        from time_response import get_time_response_presets
        return {"success": True, "presets": get_time_response_presets()}
    if action == "time_response":
        from time_response import analyze_time_response
        return analyze_time_response(
            data.get("expr") or "4 / (s^2 + 2*s + 4)",
            final_time=data.get("final_time"),
            points=data.get("points", 900),
            settling_threshold=data.get("settling_threshold", 0.02),
            theme=theme,
            response_type=data.get("response_type", "step"),
            width=data.get("width"),
            dpi=data.get("dpi"),
        )
    if action == "frequency_response_presets":
        from frequency_response import get_frequency_presets
        return {"success": True, "presets": get_frequency_presets()}
    if action == "frequency_response":
        from frequency_response import analyze_frequency_response
        return analyze_frequency_response(
            data.get("expr") or "10 / (s * (s + 2) * (s + 5))",
            omega_min=data.get("omega_min", 0.01),
            omega_max=data.get("omega_max", 100.0),
            points=data.get("points", 900),
            theme=theme,
        )
    if action == "controller_presets":
        from controller_design import get_controller_presets
        return {"success": True, "presets": get_controller_presets()}
    if action == "controller_design":
        from controller_design import design_controller
        return design_controller(
            data.get("plant_expr") or "1 / (s * (s + 1) * (s + 5))",
            design_type=data.get("design_type", "pid"),
            method=data.get("method", "zn_critical"),
            controller_type=data.get("controller_type", "PID"),
            process_gain=data.get("process_gain", 1.0),
            delay=data.get("delay", 1.0),
            time_constant=data.get("time_constant", 4.0),
            critical_gain=data.get("critical_gain", 6.0),
            critical_period=data.get("critical_period", 2.0),
            chr_response=data.get("chr_response", "0"),
            compensator_type=data.get("compensator_type", "lead"),
            compensator_zero=data.get("compensator_zero", 1.0),
            compensator_pole=data.get("compensator_pole", 5.0),
            compensator_gain=data.get("compensator_gain", 1.0),
            design_domain=data.get("design_domain", "frequency"),
            final_time=data.get("final_time"),
            points=data.get("points", 900),
            theme=theme,
        )
    if action == "state_space_presets":
        from state_space import get_state_space_presets
        return {"success": True, "presets": get_state_space_presets()}
    if action == "state_space":
        from state_space import analyze_state_space
        return analyze_state_space(
            mode=data.get("mode", "matrices"),
            expression=data.get("expr") or data.get("expression") or "1 / (s^2 + 3*s + 2)",
            a=data.get("A", "[[0, 1], [-2, -3]]"),
            b=data.get("B", "[[0], [1]]"),
            c=data.get("C", "[[1, 0]]"),
            d=data.get("D", "[[0]]"),
            canonical_form=data.get("canonical_form", "controllable"),
            desired_poles=data.get("desired_poles", ""),
            observer_poles=data.get("observer_poles", ""),
            theme=theme,
        )
    if action == "home_get_state":
        global _CURRENT_WORKSPACE_FOLDER
        req_folder = (data.get("folder") or "").strip()
        if req_folder:
            _CURRENT_WORKSPACE_FOLDER = os.path.abspath(os.path.expanduser(req_folder))
        if not _CURRENT_WORKSPACE_FOLDER:
            _CURRENT_WORKSPACE_FOLDER = get_default_workspace()

        try:
            os.makedirs(_CURRENT_WORKSPACE_FOLDER, exist_ok=True)
        except Exception:
            pass

        return {
            "success": True,
            "folder": format_display_path(_CURRENT_WORKSPACE_FOLDER),
            "folderPath": _CURRENT_WORKSPACE_FOLDER,
            "files": list_workspace_files(_CURRENT_WORKSPACE_FOLDER),
            "recent": data.get("recent") or _RECENT_ITEMS,
            "workspace": _format_workspace(),
        }
    if action == "console_exec":
        cmd = (data.get("command") or "").strip()
        if not cmd:
            return {"success": True, "output": "", "workspace": _format_workspace()}

        if cmd in ("clc", "clear"):
            return {"success": True, "output": "__CLEAR__", "workspace": _format_workspace()}

        if cmd in ("help", "ajuda"):
            help_text = (
                "ControLAB Console v3.1.1\n"
                "Comandos disponíveis: help, clc, clear, whos, pwd, ls, dir, cd <pasta>\n"
                "Exemplos: 4/(s^2+2*s+4), K = 10, 2 + 2"
            )
            return {"success": True, "output": help_text, "workspace": _format_workspace()}

        if cmd == "pwd":
            cur = _CURRENT_WORKSPACE_FOLDER or get_default_workspace()
            return {"success": True, "output": cur, "workspace": _format_workspace()}

        if cmd in ("ls", "dir"):
            cur = _CURRENT_WORKSPACE_FOLDER or get_default_workspace()
            files = list_workspace_files(cur)
            if not files:
                out = "(pasta vazia)"
            else:
                out = "\n".join(f"{f['name']:<24} {f['type']}" for f in files)
            return {"success": True, "output": out, "workspace": _format_workspace()}

        if cmd == "cd" or cmd.startswith("cd "):
            target_path = cmd[3:].strip() if len(cmd) > 3 else get_default_workspace()
            target_path = os.path.expanduser(target_path)
            if not os.path.isabs(target_path):
                target_path = os.path.join(_CURRENT_WORKSPACE_FOLDER or get_default_workspace(), target_path)
            target_path = os.path.abspath(target_path)
            if os.path.isdir(target_path):
                _CURRENT_WORKSPACE_FOLDER = target_path
                disp = format_display_path(target_path)
                return {
                    "success": True,
                    "output": f"Pasta alterada para {disp}",
                    "folder": disp,
                    "folderPath": target_path,
                    "files": list_workspace_files(target_path),
                    "workspace": _format_workspace(),
                }
            else:
                return {
                    "success": False,
                    "error": f"Pasta não encontrada: {target_path}",
                    "workspace": _format_workspace(),
                }

        if cmd == "whos":
            lines = ["Nome\tTipo\tValor"]
            for item in _format_workspace():
                lines.append(f"{item['name']}\t{item['type']}\t{item['value']}")
            return {"success": True, "output": "\n".join(lines), "workspace": _format_workspace()}

        out, err = _eval_console_command(cmd)
        return {
            "success": True,
            "output": out if out is not None else f"Erro: {err}",
            "workspace": _format_workspace(),
        }
    raise ValueError(f"Ação desconhecida: {action}")


def main():
    try:
        if len(sys.argv) > 1:
            raw_input = sys.argv[1]
        elif not sys.stdin.isatty():
            raw_input = sys.stdin.read().strip()
        else:
            raw_input = '{"action":"presets"}'

        if not raw_input:
            raw_input = '{"action":"presets"}'
        print(json.dumps(dispatch(json.loads(raw_input))))
    except Exception as exc:
        print(json.dumps({"success": False, "error": str(exc)}))


if __name__ == "__main__":
    main()
