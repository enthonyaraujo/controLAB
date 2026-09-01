"""Bridge JSON compartilhada pelas interfaces Electron e Web do LGR Studio."""

import base64
import io
import json
import sys
import warnings

warnings.filterwarnings("ignore")

import matplotlib

matplotlib.use("Agg")
import matplotlib.backends.backend_agg
import matplotlib.backends.backend_svg
import matplotlib.pyplot as plt
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
    num, den, latex_exp, latex_fac = get_transfer_function(payload)
    title = payload.get("title", "Lugar Geométrico das Raízes")

    # Executa o LGR preservando rigorosamente os 7 passos originais.
    fig, _ax, detalhes = lgr_completo(num, den, titulo=title, show_plot=False)

    buf = io.BytesIO()
    fig.savefig(buf, format="png", dpi=160, bbox_inches="tight", facecolor="white")
    buf.seek(0)
    img_base64 = "data:image/png;base64," + base64.b64encode(buf.read()).decode(
        "utf-8"
    )

    svg_buf = io.BytesIO()
    fig.savefig(svg_buf, format="svg", bbox_inches="tight", facecolor="white")
    svg_buf.seek(0)
    svg_text = svg_buf.read().decode("utf-8")
    plt.close(fig)

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
        },
    }


def dispatch(data):
    action = data.get("action", "calculate")
    if action == "presets":
        return get_presets_response()
    if action == "preview":
        return handle_preview(data)
    if action == "calculate":
        return handle_calculate(data)
    raise ValueError(f"Ação desconhecida: {action}")


def main():
    try:
        raw_input = sys.stdin.read().strip()
        if not raw_input:
            raw_input = sys.argv[1] if len(sys.argv) > 1 else '{"action":"presets"}'
        print(json.dumps(dispatch(json.loads(raw_input))))
    except Exception as exc:
        print(json.dumps({"success": False, "error": str(exc)}))


if __name__ == "__main__":
    main()
