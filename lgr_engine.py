"""
Módulo de Cálculo e Plotagem do Lugar Geométrico das Raízes (LGR)
Implementa rigorosamente os 7 passos clássicos da análise de controle.
"""

import re
import numpy as np
import matplotlib.pyplot as plt
import control as ct
from matplotlib.patches import Arc
import sympy as sp
from sympy.parsing.sympy_parser import (
    parse_expr,
    standard_transformations,
    implicit_multiplication_application,
    convert_xor,
)

PRESETS = {
    "Exemplo 1: 3ª Ordem com Breakaway e Cruzamento jω (Ogata)": {
        "expr": "(s + 2) / (s * (s + 1) * (s + 4))",
        "num": [1.0, 2.0],
        "den": [1.0, 5.0, 4.0, 0.0],
        "desc": "Polos em 0, -1, -4 e Zero em -2. Apresenta ponto de partida (breakaway) e cruzamento com o eixo imaginário.",
    },
    "Exemplo 2: Polos Complexos Conjugados e Ângulo de Partida": {
        "expr": "1 / (s * (s^2 + 2*s + 2))",
        "num": [1.0],
        "den": [1.0, 2.0, 2.0, 0.0],
        "desc": "Polo na origem e par complexo conjugado (-1 ± 1j). Demonstra o cálculo exato do ângulo de partida theta_d.",
    },
    "Exemplo 3: Zero de Fase Não-Mínima (Semiplano Direito)": {
        "expr": "(s - 2) / (s * (s + 1) * (s + 3))",
        "num": [1.0, -2.0],
        "den": [1.0, 4.0, 3.0, 0.0],
        "desc": "Sistema com zero de fase não-mínima no semiplano direito (s = +2), atraindo um ramo para a região instável.",
    },
    "Exemplo 4: Sistema com Break-in e Breakaway": {
        "expr": "(s + 2) / (s * (s + 1))",
        "num": [1.0, 2.0],
        "den": [1.0, 1.0, 0.0],
        "desc": "Dois polos (0, -1) e um zero (-2). Os ramos saem do eixo real e retornam a ele (break-in).",
    },
    "Exemplo 5: 4ª Ordem com 4 Assíntotas": {
        "expr": "1 / (s * (s + 1) * (s + 2) * (s + 3))",
        "num": [1.0],
        "den": [1.0, 6.0, 11.0, 6.0, 0.0],
        "desc": "4 polos reais na origem e em -1, -2, -3. Quatro assíntotas simétricas a 45°, 135°, 225° e 315°.",
    },
    "Exemplo 6: Sistema de 2ª Ordem com Amortecimento": {
        "expr": "1 / (s^2 + 2*s + 5)",
        "num": [1.0],
        "den": [1.0, 2.0, 5.0],
        "desc": "Par de polos conjugados em -1 ± 2j com ramos paralelos assintóticos verticais.",
    },
}


_SUPERSCRIPT_DIGITS = str.maketrans("⁰¹²³⁴⁵⁶⁷⁸⁹⁻", "0123456789-")
_SCIENTIFIC_NUMBER = re.compile(r"(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?")


def normalize_tf_expression(expr_str: str) -> str:
    """Normaliza variações comuns de escrita sem mudar a expressão matemática."""
    expr = (expr_str or "").strip()
    if not expr:
        raise ValueError("A expressão não pode estar vazia.")

    # Permite colar expressões como "G(s) = ..." ou "G(s)H(s) = ...".
    expr = re.sub(
        r"^\s*(?:(?:G|H|L)\s*\(\s*s\s*\)\s*)+(?:=|:)\s*",
        "",
        expr,
        flags=re.IGNORECASE,
    )
    expr = (
        expr.replace("−", "-")
        .replace("–", "-")
        .replace("×", "*")
        .replace("·", "*")
        .replace("÷", "/")
        .replace("[", "(")
        .replace("]", ")")
    )

    def replace_superscript(match):
        exponent = match.group(0).translate(_SUPERSCRIPT_DIGITS)
        return f"^({exponent})"

    expr = re.sub(r"[⁰¹²³⁴⁵⁶⁷⁸⁹⁻]+", replace_superscript, expr)
    if not expr.strip():
        raise ValueError("Informe a expressão após G(s) =.")
    return expr


def _parse_polynomial_expression(expr_str: str, field_name: str = "expressão"):
    """Interpreta somente polinômios reais em ``s`` com uma gramática restrita."""
    normalized = normalize_tf_expression(expr_str)
    remainder = _SCIENTIFIC_NUMBER.sub("", normalized)
    if not re.fullmatch(r"[sS+\-*/^().\s]*", remainder):
        raise ValueError(
            f"A {field_name} contém símbolos não suportados. Use números, s, "
            "parênteses e os operadores +, -, *, / ou ^."
        )
    if "//" in normalized:
        raise ValueError("Use / para divisão; o operador // não é suportado.")

    s = sp.Symbol("s")
    transformations = standard_transformations + (
        implicit_multiplication_application,
        convert_xor,
    )
    safe_globals = {
        "__builtins__": {},
        "Integer": sp.Integer,
        "Float": sp.Float,
        "Rational": sp.Rational,
    }
    try:
        return parse_expr(
            normalized,
            local_dict={"s": s, "S": s},
            global_dict=safe_globals,
            transformations=transformations,
        )
    except Exception as exc:
        raise ValueError(f"Não foi possível interpretar a {field_name}: {exc}") from exc


def _coefficients_and_latex(numer, denom):
    s = sp.Symbol("s")
    numer = sp.expand(numer)
    denom = sp.expand(denom)
    if sp.simplify(denom) == 0:
        raise ValueError("O denominador não pode ser zero.")

    try:
        poly_num = sp.Poly(numer, s)
        poly_den = sp.Poly(denom, s)
    except sp.PolynomialError as exc:
        raise ValueError("A função deve ser uma razão de polinômios em s.") from exc

    num_coeffs = [float(c) for c in poly_num.all_coeffs()]
    den_coeffs = [float(c) for c in poly_den.all_coeffs()]
    latex_expanded = rf"G(s) = \frac{{{sp.latex(poly_num.as_expr())}}}{{{sp.latex(poly_den.as_expr())}}}"
    latex_factored = rf"G(s) = \frac{{{sp.latex(sp.factor(numer))}}}{{{sp.latex(sp.factor(denom))}}}"
    return num_coeffs, den_coeffs, latex_expanded, latex_factored


def parse_tf_expression(expr_str: str):
    """
    Interpreta uma expressão algébrica simbólica de Função de Transferência
    Exemplos aceitos:
      - '(s + 2) / (s * (s + 1) * (s + 4))'
      - '10*(s+1) / (s^3 + 2s^2 + 2s)'
      - '1 / (s(s+1)(s+2))'
      - 'G(s) = (s + 2) / (s³ + 5s² + 4s)'
    Retorna (num_coeffs, den_coeffs, latex_expanded, latex_factored)
    """
    expr = _parse_polynomial_expression(expr_str)
    frac = sp.together(expr)
    numer, denom = sp.fraction(frac)
    return _coefficients_and_latex(numer, denom)


def parse_tf_parts(numerator_str: str, denominator_str: str):
    """Interpreta numerador e denominador escritos como polinômios separados."""
    numerator = _parse_polynomial_expression(numerator_str, "numerador")
    denominator = _parse_polynomial_expression(denominator_str, "denominador")
    return _coefficients_and_latex(numerator, denominator)


def format_transfer_function(num, den):
    """Gera as formas LaTeX expandida e fatorada a partir de coeficientes."""
    s = sp.Symbol("s")
    numerator = sp.Poly.from_list(num, gens=s).as_expr()
    denominator = sp.Poly.from_list(den, gens=s).as_expr()
    return _coefficients_and_latex(numerator, denominator)


def parse_coeffs_str(coeffs_str: str):
    """
    Converte uma string de coeficientes separados por espaço ou vírgula em lista de floats.
    Ex: '1, 5, 4, 0' ou '[1 5 4 0]' -> [1.0, 5.0, 4.0, 0.0]
    """
    cleaned = (
        coeffs_str.replace("[", "").replace("]", "").replace(",", " ").strip()
    )
    if not cleaned:
        raise ValueError("A lista de coeficientes não pode estar vazia.")
    vals = [float(x) for x in cleaned.split() if x]
    return vals


def parse_zpk(zeros_str: str, poles_str: str, k_gain: float = 1.0):
    """
    Converte listas de zeros e polos e ganho K em coeficientes polinomiais.
    """
    def parse_complex_item(s):
        s = s.strip().replace("i", "j")
        if not s:
            return None
        return complex(s)

    z_list = []
    if zeros_str.strip():
        for item in zeros_str.split(","):
            val = parse_complex_item(item)
            if val is not None:
                z_list.append(val)

    p_list = []
    if poles_str.strip():
        for item in poles_str.split(","):
            val = parse_complex_item(item)
            if val is not None:
                p_list.append(val)

    num_poly = np.poly1d([k_gain])
    for z in z_list:
        num_poly = np.polymul(num_poly, np.poly1d([1, -z]))

    den_poly = np.poly1d([1.0])
    for p in p_list:
        den_poly = np.polymul(den_poly, np.poly1d([1, -p]))

    num_coeffs = [float(np.real(c)) for c in num_poly.coeffs]
    den_coeffs = [float(np.real(c)) for c in den_poly.coeffs]

    return num_coeffs, den_coeffs


def format_complex(val, decimals=2):
    """Formata número complexo de forma clara para apresentação."""
    re = np.real(val)
    im = np.imag(val)
    if abs(im) < 1e-5:
        return f"{re:.{decimals}f}"
    if abs(re) < 1e-5:
        return f"{im:+.{decimals}f}j"
    return f"{re:.{decimals}f} {im:+.{decimals}f}j"


def format_real_num(val, decimals=2):
    """Formata número real sem notação científica desnecessária e sem zeros supérfluos."""
    val_f = float(val)
    if abs(val_f - round(val_f)) < 1e-5:
        return f"{int(round(val_f))}"
    return f"{val_f:.{decimals}f}".rstrip("0").rstrip(".")


def gerar_passo_a_passo(
    num,
    den,
    polos,
    zeros,
    centroide,
    angulos_assintotas,
    break_points,
    jw_cruzamentos,
    angulos_partida,
    angulos_chegada,
):
    """Gera o memorial algébrico formal com desenvolvimento analítico passo a passo."""
    s = sp.Symbol("s")
    poly_N = sp.Poly.from_list([float(c) for c in num], gens=s)
    poly_D = sp.Poly.from_list([float(c) for c in den], gens=s)

    P = int(len(polos))
    Z = int(len(zeros))
    ramos = int(max(P, Z))

    # --- PASSOS 1, 2 e 3: Singularidades e Segmentos Reais ---
    expr_N_latex = sp.latex(poly_N.as_expr())
    expr_D_latex = sp.latex(poly_D.as_expr())
    fact_N_latex = sp.latex(sp.factor(poly_N.as_expr()))
    fact_D_latex = sp.latex(sp.factor(poly_D.as_expr()))

    sing_reais = []
    for p in polos:
        if abs(np.imag(p)) < 1e-4:
            sing_reais.append((float(np.real(p)), "polo"))
    for z in zeros:
        if abs(np.imag(z)) < 1e-4:
            sing_reais.append((float(np.real(z)), "zero"))

    sing_reais.sort(key=lambda item: item[0], reverse=True)

    segmentos = []
    if sing_reais:
        v_max = format_real_num(sing_reais[0][0])
        segmentos.append({
            "intervalo": f"({v_max}, +\\infty)",
            "contagem_direita": 0,
            "pertence": False,
            "motivo": "0 singularidades reais à direita (quantidade par)",
        })
        for i in range(len(sing_reais) - 1):
            val_esq = format_real_num(sing_reais[i + 1][0])
            val_dir = format_real_num(sing_reais[i][0])
            count_dir = i + 1
            pertence = bool(count_dir % 2 == 1)
            segmentos.append({
                "intervalo": f"({val_esq}, {val_dir})",
                "contagem_direita": count_dir,
                "pertence": pertence,
                "motivo": (
                    f"{count_dir} singularidade(s) real(is) à direita (ímpar: no LGR)"
                    if pertence
                    else f"{count_dir} singularidade(s) real(is) à direita (par: fora do LGR)"
                ),
            })
        count_final = len(sing_reais)
        pertence_final = bool(count_final % 2 == 1)
        v_min = format_real_num(sing_reais[-1][0])
        segmentos.append({
            "intervalo": f"(-\\infty, {v_min})",
            "contagem_direita": count_final,
            "pertence": pertence_final,
            "motivo": (
                f"{count_final} singularidade(s) real(is) à direita (ímpar: no LGR)"
                if pertence_final
                else f"{count_final} singularidade(s) real(is) à direita (par: fora do LGR)"
            ),
        })

    passo_1_2_3 = {
        "eq_malha_aberta": rf"G(s) = \frac{{{expr_N_latex}}}{{{expr_D_latex}}}",
        "eq_fatorada": rf"G(s) = \frac{{{fact_N_latex}}}{{{fact_D_latex}}}",
        "eq_caracteristica": rf"1 + K \, G(s) = 0 \iff ({expr_D_latex}) + K ({expr_N_latex}) = 0",
        "P": P,
        "Z": Z,
        "ramos": ramos,
        "segmentos_eixo_real": segmentos,
        "simetria_justificativa": (
            "Como os coeficientes de N(s) e D(s) são estritamente reais, "
            "as raízes complexas de D(s) + K N(s) = 0 ocorrem obrigatoriamente "
            "em pares conjugados, garantindo simetria reflexiva ao longo do eixo real."
        ),
    }

    # --- PASSO 4: Assíntotas e Centroide ---
    passo_4 = {
        "tem_assintotas": bool(P > Z),
        "n_assintotas": int(max(0, P - Z)),
        "formula_centroide": r"\sigma_a = \frac{\sum_{i=1}^P p_i - \sum_{j=1}^Z z_j}{P - Z}",
        "formula_angulos": r"\theta_k = \frac{(2k + 1) \cdot 180^\circ}{P - Z}, \quad k = 0, \dots, (P - Z - 1)",
        "soma_polos_num": float(np.real(np.sum(polos))) if P > 0 else 0.0,
        "soma_zeros_num": float(np.real(np.sum(zeros))) if Z > 0 else 0.0,
        "substituicao_centroide": "",
        "angulos_deduzidos": [],
    }
    if P > Z:
        diff_order = P - Z
        s_p = passo_4["soma_polos_num"]
        s_z = passo_4["soma_zeros_num"]
        s_p_str = f"{s_p:.2f}".rstrip("0").rstrip(".")
        s_z_str = f"{s_z:.2f}".rstrip("0").rstrip(".")
        cent_val = (
            float(centroide)
            if centroide is not None
            else float((s_p - s_z) / diff_order)
        )
        passo_4["substituicao_centroide"] = (
            rf"\sigma_a = \frac{{({s_p_str}) - ({s_z_str})}}{{{diff_order}}} = "
            rf"\frac{{{s_p - s_z:.2f}}}{{{diff_order}}} = {cent_val:.3f}"
        )
        for k in range(diff_order):
            deg = ((2 * k + 1) * 180.0) / diff_order
            passo_4["angulos_deduzidos"].append({
                "k": int(k),
                "formula": (
                    rf"\theta_{{{k}}} = \frac{{(2({k}) + 1) \cdot 180^\circ}}{{{diff_order}}} = "
                    rf"\frac{{{ (2*k+1)*180 }^\circ}}{{{diff_order}}} = {deg:.1f}^\circ"
                ),
                "graus": float(deg % 360),
            })

    # --- PASSO 5: Pontos de Quebra no Eixo Real ---
    dN = poly_N.diff(s)
    dD = poly_D.diff(s)
    break_poly_expr = sp.expand(
        dN.as_expr() * poly_D.as_expr() - poly_N.as_expr() * dD.as_expr()
    )

    N_func = np.poly1d(num)
    D_func = np.poly1d(den)
    eq_break_np = np.polysub(
        np.polymul(np.polyder(N_func), D_func),
        np.polymul(N_func, np.polyder(D_func)),
    )
    raizes_break = np.roots(eq_break_np) if len(eq_break_np) > 0 else []

    raizes_analisadas = []
    for r in raizes_break:
        r_re = float(np.real(r))
        r_im = float(np.imag(r))
        if abs(r_im) > 1e-4:
            raizes_analisadas.append({
                "s_str": f"{r_re:.2f} ± j{abs(r_im):.2f}",
                "tipo": "complexa",
                "valido": False,
                "motivo": "Raiz com parte imaginária (fora do eixo real)",
            })
        else:
            val_N = float(N_func(r_re))
            if abs(val_N) < 1e-8:
                K_calc = None
                valido = False
                motivo = "Raiz coincide com zero finito do numerador"
            else:
                K_calc = float(-D_func(r_re) / val_N)
                count_dir = sum(1 for s_val, _ in sing_reais if s_val > r_re)
                no_segmento = bool(count_dir % 2 == 1)
                if K_calc > 0 and no_segmento:
                    valido = True
                    motivo = f"Ponto de Quebra Válido ($K = {K_calc:.2f} > 0$ e pertence ao LGR)"
                elif K_calc <= 0:
                    valido = False
                    motivo = f"Descartado: $K = {K_calc:.2f} \\le 0$ (requer ganho positivo)"
                else:
                    valido = False
                    motivo = f"Descartado: ponto $s = {r_re:.2f}$ fora dos ramos reais do LGR"

            raizes_analisadas.append({
                "s_str": f"{r_re:.3f}",
                "s_val": r_re,
                "tipo": "real",
                "K": K_calc,
                "valido": valido,
                "motivo": motivo,
            })

    passo_5 = {
        "formula_derivada": (
            r"K(s) = -\frac{D(s)}{N(s)} \implies "
            r"\frac{dK}{ds} = -\frac{D'(s)N(s) - D(s)N'(s)}{N(s)^2} = 0"
        ),
        "condicao": r"N'(s)D(s) - N(s)D'(s) = 0",
        "derivada_N": sp.latex(dN.as_expr()),
        "derivada_D": sp.latex(dD.as_expr()),
        "polinomio_break": sp.latex(break_poly_expr) + " = 0",
        "raizes_analisadas": raizes_analisadas,
    }

    # --- PASSO 6: Cruzamento com o Eixo Imaginário (jw) ---
    w = sp.Symbol("w", real=True)
    K_sym = sp.Symbol("K", real=True)
    eq_char_sym = sp.expand(poly_D.as_expr() + K_sym * poly_N.as_expr())
    eq_jw_sym = eq_char_sym.subs(s, sp.I * w)
    re_part = sp.re(eq_jw_sym)
    im_part = sp.im(eq_jw_sym)

    cruzamentos_deduzidos = []
    seen = set()
    for item in jw_cruzamentos:
        w_val = abs(float(item["w"]))
        k_val = float(item["K"])
        key = f"{w_val:.2f}_{k_val:.2f}"
        if key not in seen:
            seen.add(key)
            cruzamentos_deduzidos.append({
                "w": w_val,
                "K": k_val,
                "s_str": rf"s = \pm j{w_val:.2f}",
                "K_str": f"K_{{crit}} = {k_val:.2f}",
            })

    passo_6 = {
        "eq_caracteristica": sp.latex(eq_char_sym) + " = 0",
        "substituicao_jw": r"s = j\omega \implies D(j\omega) + K N(j\omega) = 0",
        "parte_real_eq": sp.latex(re_part) + " = 0",
        "parte_imaginaria_eq": sp.latex(im_part) + " = 0",
        "tem_cruzamento": bool(len(cruzamentos_deduzidos) > 0),
        "cruzamentos": cruzamentos_deduzidos,
    }

    # --- PASSO 7: Ângulos de Partida e Chegada ---
    polos_complexos_sup = [p for p in polos if np.imag(p) > 1e-4]
    zeros_complexos_sup = [z for z in zeros if np.imag(z) > 1e-4]

    deducoes_partida = []
    for p in polos_complexos_sup:
        termos_zeros = []
        soma_zeros_ang = 0.0
        for z in zeros:
            diff = p - z
            ang = float(np.degrees(np.angle(diff)))
            termos_zeros.append({
                "zero_str": format_complex(z),
                "vetor_str": format_complex(diff),
                "angulo": float(ang),
            })
            soma_zeros_ang += ang

        termos_polos = []
        soma_polos_ang = 0.0
        for p_outro in polos:
            if abs(p - p_outro) > 1e-4:
                diff = p - p_outro
                ang = float(np.degrees(np.angle(diff)))
                termos_polos.append({
                    "polo_str": format_complex(p_outro),
                    "vetor_str": format_complex(diff),
                    "angulo": float(ang),
                })
                soma_polos_ang += ang

        ang_total = 180.0 + soma_zeros_ang - soma_polos_ang
        ang_norm = float((ang_total + 180.0) % 360.0 - 180.0)

        deducoes_partida.append({
            "polo_str": format_complex(p),
            "termos_zeros": termos_zeros,
            "termos_polos": termos_polos,
            "soma_zeros_graus": float(soma_zeros_ang),
            "soma_polos_graus": float(soma_polos_ang),
            "formula_aplicada": r"\theta_d = 180^\circ + \sum \phi_z - \sum \theta_p",
            "calculo_substituicao": (
                rf"\theta_d = 180^\circ + ({soma_zeros_ang:.1f}^\circ) - ({soma_polos_ang:.1f}^\circ) = {ang_norm:.1f}^\circ"
            ),
            "angulo_final": ang_norm,
        })

    deducoes_chegada = []
    for z in zeros_complexos_sup:
        termos_polos = []
        soma_polos_ang = 0.0
        for p in polos:
            diff = z - p
            ang = float(np.degrees(np.angle(diff)))
            termos_polos.append({
                "polo_str": format_complex(p),
                "vetor_str": format_complex(diff),
                "angulo": float(ang),
            })
            soma_polos_ang += ang

        termos_zeros = []
        soma_zeros_ang = 0.0
        for z_outro in zeros:
            if abs(z - z_outro) > 1e-4:
                diff = z - z_outro
                ang = float(np.degrees(np.angle(diff)))
                termos_zeros.append({
                    "zero_str": format_complex(z_outro),
                    "vetor_str": format_complex(diff),
                    "angulo": float(ang),
                })
                soma_zeros_ang += ang

        ang_total = 180.0 + soma_polos_ang - soma_zeros_ang
        ang_norm = float((ang_total + 180.0) % 360.0 - 180.0)

        deducoes_chegada.append({
            "zero_str": format_complex(z),
            "termos_polos": termos_polos,
            "termos_zeros": termos_zeros,
            "soma_polos_graus": float(soma_polos_ang),
            "soma_zeros_graus": float(soma_zeros_ang),
            "formula_aplicada": r"\theta_a = 180^\circ + \sum \phi_p - \sum \theta_z",
            "calculo_substituicao": (
                rf"\theta_a = 180^\circ + ({soma_polos_ang:.1f}^\circ) - ({soma_zeros_ang:.1f}^\circ) = {ang_norm:.1f}^\circ"
            ),
            "angulo_final": ang_norm,
        })

    passo_7 = {
        "tem_partida": bool(len(deducoes_partida) > 0),
        "tem_chegada": bool(len(deducoes_chegada) > 0),
        "deducoes_partida": deducoes_partida,
        "deducoes_chegada": deducoes_chegada,
    }

    return {
        "passo_1_2_3": passo_1_2_3,
        "passo_4": passo_4,
        "passo_5": passo_5,
        "passo_6": passo_6,
        "passo_7": passo_7,
    }


def lgr_completo(num, den=None, titulo="Lugar Geométrico das Raízes", show_plot=False):
    """
    Gera um gráfico do LGR autossuficiente com cálculos automáticos 
    para todos os 7 passos clássicos da análise, incluindo legendas dinâmicas.
    
    Aceita múltiplos formatos de entrada:
      1. String única: lgr_completo("(s+1)/(s+2)") ou lgr_completo("1/(s*(s+3)*(s^2+6s+64))")
      2. Objeto TransferFunction direto: lgr_completo(sys)
      3. Listas ou arrays numéricos: lgr_completo([1, 2], [1, 5, 4, 0])
      4. Strings separadas para numerador e denominador: lgr_completo("s+2", "s^3 + 5s^2 + 4s")
    
    Retorna: (fig, ax, detalhes_dos_passos)
    """

    # --------------------------------------------------------
    # TRATAMENTO DE ENTRADA (Múltiplos formatos)
    # --------------------------------------------------------
    if isinstance(num, str) and isinstance(den, str):
        # CASO 0: Strings separadas para numerador e denominador
        n_coeffs, d_coeffs, _, _ = parse_tf_parts(num, den)
        sys = ct.tf(n_coeffs, d_coeffs)

    elif isinstance(num, str):
        # CASO 1: Passou uma string (ex: "(s+1)/(s+2)")
        try:
            s = ct.tf('s')
            sys = eval(num)
            if not isinstance(sys, ct.TransferFunction):
                raise ValueError("A expressão não resultou em TransferFunction.")
        except Exception:
            # Fallback para parser algébrico simbólico (suporta ^, potências Unicode, multiplicação implícita)
            n_coeffs, d_coeffs, _, _ = parse_tf_expression(num)
            sys = ct.tf(n_coeffs, d_coeffs)

    elif isinstance(num, ct.TransferFunction):
        # CASO 2: Passou o sistema direto (ex: sys_algebrico)
        sys = num
        
    elif den is not None:
        # CASO 3: Passou as listas para num e den
        sys = ct.tf(num, den)

    else:
        raise ValueError("Entrada inválida! Faltou passar o denominador, ou passe a Função de Transferência direto.")

    # Sobrescreve as variáveis 'num' e 'den' para que elas voltem a ser listas numéricas
    num = [float(c) for c in sys.num[0][0]]
    den = [float(c) for c in sys.den[0][0]]
    
    # 1. PREPARAÇÃO E EXTRAÇÃO DE DADOS
    polos = ct.poles(sys)
    zeros = ct.zeros(sys)
    P = len(polos)
    Z = len(zeros)
    ramos = max(P, Z)
    
    # Estrutura para armazenar detalhes matemáticos dos 7 passos para o frontend
    detalhes = {
        "num": num,
        "den": den,
        "polos": polos,
        "zeros": zeros,
        "P": P,
        "Z": Z,
        "ramos": ramos,
        "centroide": None,
        "angulos_assintotas": [],
        "break_points": [],
        "jw_cruzamentos": [],
        "angulos_partida": [],
        "angulos_chegada": []
    }
    
    # Gera os dados exatos do LGR (lista de raízes e ganhos K)
    # Aumentamos o limite para 10^4 e os pontos para 10000 para manter a alta precisão
    kvect = np.logspace(-3, 4, 10000)
    kvect = np.insert(kvect, 0, 0)
    rlist, klist = ct.root_locus(sys, kvect=kvect, plot=False)
    
    # Configuração da Figura
    fig, ax = plt.subplots(figsize=(11, 7.5), dpi=100)
    ax.grid(True, linestyle=':', alpha=0.6, color='#94a3b8')
    ax.axhline(0, color='#334155', linewidth=1.2) # Eixo Real
    ax.axvline(0, color='#334155', linewidth=1.2) # Eixo jw
    
    # ========================================================
    # PASSO 1, 2 e 3: Ramos, Polos, Zeros e Simetria
    # ========================================================
    # Traça os ramos
    for i in range(ramos):
        ax.plot(np.real(rlist[:, i]), np.imag(rlist[:, i]), color='#dc2626', linewidth=2.2, 
                label='Ramos do LGR' if i == 0 else "_nolegend_")
        
    # Marca polos (x)
    ax.plot(np.real(polos), np.imag(polos), 'kx', markersize=9, markeredgewidth=2.2, label='Polos (Início)', zorder=5)
    
    # Marca zeros (o)
    if Z > 0:
        ax.plot(np.real(zeros), np.imag(zeros), 'ko', markerfacecolor='white', markersize=8.5, markeredgewidth=2.2, label='Zeros (Término)', zorder=5)
    else:
        ax.plot([], [], 'ko', markerfacecolor='white', markersize=8.5, markeredgewidth=2.2, label='Zeros (Nenhum nesta FT)')

    # ========================================================
    # PASSO 4: Assíntotas e Centroide
    # ========================================================
    centroide = None
    if P > Z:
        centroide = float(np.real((np.sum(polos) - np.sum(zeros)) / (P - Z)))
        detalhes["centroide"] = centroide
        ax.plot(centroide, 0, 'k+', markersize=11, markeredgewidth=2, label=rf'Centroide ($\sigma_a = {centroide:.2f}$)', zorder=6)
        
        raio = 100
        for k in range(P - Z):
            ang_rad = np.pi * (2 * k + 1) / (P - Z)
            ang_deg = np.degrees(ang_rad) % 360
            detalhes["angulos_assintotas"].append({
                "k": k,
                "graus": ang_deg,
                "rad": ang_rad
            })
            x_end = centroide + raio * np.cos(ang_rad)
            y_end = raio * np.sin(ang_rad)
            ax.plot([centroide, x_end], [0, y_end], color='#64748b', linestyle='--', linewidth=1.2, zorder=0,
                    label='Assíntotas' if k == 0 else "_nolegend_")

    # ========================================================
    # PASSO 5: Pontos de Entrada e Saída (Break-in / Breakaway)
    # ========================================================
    N = np.poly1d(num)
    D = np.poly1d(den)
    dN = np.polyder(N)
    dD = np.polyder(D)
    eq_break = np.polysub(np.polymul(dN, D), np.polymul(N, dD))
    
    raizes_break = np.roots(eq_break)
    break_pts = []
    
    for r in raizes_break:
        if abs(np.imag(r)) < 1e-5:
            val_N = N(np.real(r))
            if abs(val_N) > 1e-8:
                K_break = -D(np.real(r)) / val_N
                if K_break > 0:
                    detalhes["break_points"].append({
                        "s": float(np.real(r)),
                        "K": float(K_break)
                    })
                    break_pts.append((float(np.real(r)), float(K_break)))

    for idx, (r_val, K_b) in enumerate(break_pts):
        lbl = f'Break-in/out (K={K_b:.1f})' if idx == 0 else "_nolegend_"
        ax.plot(r_val, 0, 's', color='#2563eb', markersize=7, label=lbl, zorder=6)

    # ========================================================
    # PASSO 6: Cruzamento do eixo jw
    # ========================================================
    jw_pts = []
    for i in range(ramos):
        ramo = rlist[:, i]
        for j in range(len(ramo)-1):
            if np.real(ramo[j]) * np.real(ramo[j+1]) < 0:
                t = abs(np.real(ramo[j])) / (abs(np.real(ramo[j])) + abs(np.real(ramo[j+1])))
                w_cruz = np.imag(ramo[j]) + t * (np.imag(ramo[j+1]) - np.imag(ramo[j]))
                K_cruz = klist[j] + t * (klist[j+1] - klist[j])
                
                if abs(w_cruz) > 1e-2:
                    detalhes["jw_cruzamentos"].append({
                        "w": float(w_cruz),
                        "K": float(K_cruz)
                    })
                    jw_pts.append((float(w_cruz), float(K_cruz)))

    for idx, (w_cruz, K_c) in enumerate(jw_pts):
        lbl = rf'Cruz. j$\omega$ ($\omega=\pm${abs(w_cruz):.2f}, K={K_c:.1f})' if idx == 0 else "_nolegend_"
        ax.plot(0, w_cruz, 'o', color='#b91c1c', markerfacecolor='white', markeredgewidth=2, markersize=7, label=lbl, zorder=6)

    # ========================================================
    # CÁLCULO INTELIGENTE DOS LIMITES DO GRÁFICO
    # ========================================================
    all_x = [float(np.real(p)) for p in polos] + [float(np.real(z)) for z in zeros] + [0.0]
    all_y = [abs(float(np.imag(p))) for p in polos] + [abs(float(np.imag(z))) for z in zeros] + [abs(w) for w, _ in jw_pts]
    if centroide is not None:
        all_x.append(centroide)
    for r_val, _ in break_pts:
        all_x.append(r_val)

    x_min, x_max = min(all_x), max(all_x)
    y_max = max(all_y) if all_y else 2.0
    y_max = max(y_max, 2.0)

    span_x = max(x_max - x_min, 4.0)
    span_y = max(2 * y_max, 4.0)
    pad_x = span_x * 0.18
    pad_y = span_y * 0.15

    ax.set_xlim(min(x_min - pad_x, -1.0), max(x_max + pad_x, 1.5))
    ax.set_ylim(-y_max - pad_y, y_max + pad_y)

    # ========================================================
    # ANOTAÇÕES ESTRUTURADAS ANTI-COLISÃO (COM BADGES)
    # ========================================================
    # Anotações dos Polos
    for p in polos:
        re_p, im_p = np.real(p), np.imag(p)
        if abs(im_p) < 1e-5:
            txt = f"p={re_p:.2g}"
            ax.annotate(txt, xy=(re_p, 0), xytext=(0, 14), textcoords="offset points",
                        ha="center", va="bottom", fontsize=9, fontweight="bold",
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="white", edgecolor="#cbd5e1", alpha=0.9, lw=0.6))
        else:
            sgn = "+" if im_p >= 0 else "-"
            txt = f"p={re_p:.2g}{sgn}j{abs(im_p):.2g}"
            offset_y = 12 if im_p > 0 else -18
            ax.annotate(txt, xy=(re_p, im_p), xytext=(-10, offset_y), textcoords="offset points",
                        ha="right" if re_p < 0 else "left", va="center", fontsize=9,
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="white", edgecolor="#cbd5e1", alpha=0.9, lw=0.6))

    # Anotações dos Zeros
    for z in zeros:
        re_z, im_z = np.real(z), np.imag(z)
        if abs(im_z) < 1e-5:
            txt = f"z={re_z:.2g}"
            ax.annotate(txt, xy=(re_z, 0), xytext=(0, -18), textcoords="offset points",
                        ha="center", va="top", fontsize=9, fontweight="bold",
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="white", edgecolor="#cbd5e1", alpha=0.9, lw=0.6))
        else:
            sgn = "+" if im_z >= 0 else "-"
            txt = f"z={re_z:.2g}{sgn}j{abs(im_z):.2g}"
            offset_y = 12 if im_z > 0 else -18
            ax.annotate(txt, xy=(re_z, im_z), xytext=(10, offset_y), textcoords="offset points",
                        ha="left", va="center", fontsize=9,
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="white", edgecolor="#cbd5e1", alpha=0.9, lw=0.6))

    # Anotação do Centroide
    if centroide is not None:
        ax.annotate(rf"Centroide $\sigma_a={centroide:.2f}$", xy=(centroide, 0), xytext=(0, -22), textcoords="offset points",
                    ha="center", va="top", fontsize=9, color="#1e293b",
                    bbox=dict(boxstyle="round,pad=0.25", facecolor="#f8fafc", edgecolor="#94a3b8", alpha=0.95, lw=0.7))

        # Anotações de Ângulo das Assíntotas na Periferia
        asymp_radius = min(span_x, span_y) * 0.44
        for item in detalhes["angulos_assintotas"]:
            ang_deg = item["graus"]
            ang_rad = item["rad"]
            ax.text(centroide + asymp_radius * np.cos(ang_rad), asymp_radius * np.sin(ang_rad), f"{ang_deg:.0f}°",
                    ha="center", va="center", fontsize=8.5, color="#475569",
                    bbox=dict(boxstyle="round,pad=0.2", facecolor="white", edgecolor="#cbd5e1", alpha=0.9, lw=0.5))

    # Anotações de Break-in / Breakaway
    for r_val, K_b in break_pts:
        ax.annotate(f"Break: {r_val:.2f}\n(K={K_b:.1f})", xy=(r_val, 0), xytext=(0, 26), textcoords="offset points",
                    ha="center", va="bottom", fontsize=8.5, color="#1d4ed8", fontweight="bold",
                    bbox=dict(boxstyle="round,pad=0.25", facecolor="#eff6ff", edgecolor="#3b82f6", alpha=0.95, lw=0.8),
                    arrowprops=dict(arrowstyle="->", color="#3b82f6", lw=1))

    # Anotações de Cruzamento com jw
    for w_cruz, K_c in jw_pts:
        ax.annotate(rf"$j\omega={w_cruz:+.2f}$" + "\n" + rf"K={K_c:.1f}", xy=(0, w_cruz),
                    xytext=(16, 0), textcoords="offset points",
                    ha="left", va="center", fontsize=8.5, color="#991b1b",
                    bbox=dict(boxstyle="round,pad=0.25", facecolor="#fef2f2", edgecolor="#ef4444", alpha=0.95, lw=0.8),
                    arrowprops=dict(arrowstyle="->", color="#ef4444", lw=0.8))

    # ========================================================
    # PASSO 7: Ângulos de Partida (Polos) e Chegada (Zeros)
    # ========================================================
    arc_r = min(1.6, span_y * 0.08)
    
    # Ângulos de Partida para Polos Complexos
    for p in polos:
        if np.imag(p) > 1e-4:
            angulo = 180
            for z in zeros:
                angulo += np.degrees(np.angle(p - z))
            for p_outro in polos:
                if np.abs(p - p_outro) > 1e-4:
                    angulo -= np.degrees(np.angle(p - p_outro))
            
            angulo = (angulo + 180) % 360 - 180
            detalhes["angulos_partida"].append({
                "polo": p,
                "angulo": angulo
            })
            
            arco = Arc((np.real(p), np.imag(p)), 2*arc_r, 2*arc_r, angle=0, theta1=min(0, angulo), theta2=max(0, angulo), color='#7c3aed', lw=1.6)
            ax.add_patch(arco)
            ax.plot([np.real(p)-arc_r*1.2, np.real(p)+arc_r*1.2], [np.imag(p), np.imag(p)], color='#6b7280', linestyle=':', alpha=0.6)
            ax.annotate(rf"$\theta_d={angulo:.1f}^\circ$", xy=(np.real(p), np.imag(p)),
                        xytext=(arc_r*14, 10), textcoords="offset points",
                        ha="left", va="bottom", fontsize=8.5, color="#6d28d9", fontweight="bold",
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="#f5f3ff", edgecolor="#8b5cf6", alpha=0.95, lw=0.8))

    # Ângulos de Chegada para Zeros Complexos
    for z in zeros:
        if np.imag(z) > 1e-4:
            angulo = 180
            for p in polos:
                angulo += np.degrees(np.angle(z - p))
            for z_outro in zeros:
                if np.abs(z - z_outro) > 1e-4:
                    angulo -= np.degrees(np.angle(z - z_outro))
            
            angulo = (angulo + 180) % 360 - 180
            detalhes["angulos_chegada"].append({
                "zero": z,
                "angulo": angulo
            })
            
            arco = Arc((np.real(z), np.imag(z)), 2*arc_r, 2*arc_r, angle=0, theta1=min(0, angulo), theta2=max(0, angulo), color='green', lw=1.6)
            ax.add_patch(arco)
            ax.plot([np.real(z)-arc_r*1.2, np.real(z)+arc_r*1.2], [np.imag(z), np.imag(z)], color='#6b7280', linestyle=':', alpha=0.6)
            ax.annotate(rf"$\theta_a={angulo:.1f}^\circ$", xy=(np.real(z), np.imag(z)),
                        xytext=(arc_r*14, 10), textcoords="offset points",
                        ha="left", va="bottom", fontsize=8.5, color="#15803d", fontweight="bold",
                        bbox=dict(boxstyle="round,pad=0.2", facecolor="#f0fdf4", edgecolor="#16a34a", alpha=0.95, lw=0.8))

    # ========================================================
    # TÍTULOS E LEGENDA FINAL
    # ========================================================
    ax.set_title(titulo, fontsize=14, pad=15, fontweight="bold")
    ax.set_xlabel(r'Eixo Real ($\sigma$)', fontsize=11, labelpad=8)
    ax.set_ylabel(r'Eixo Imaginário ($j\omega$)', fontsize=11, labelpad=8)
    
    # Posiciona a legenda com fundo semi-transparente fora dos ramos principais
    ax.legend(loc='lower left', bbox_to_anchor=(1.02, 0.4), borderaxespad=0, title="Componentes do LGR", framealpha=0.95)
    
    plt.tight_layout()
    if show_plot:
        plt.show()

    detalhes["passo_a_passo"] = gerar_passo_a_passo(
        num,
        den,
        polos,
        zeros,
        centroide,
        detalhes["angulos_assintotas"],
        detalhes["break_points"],
        detalhes["jw_cruzamentos"],
        detalhes["angulos_partida"],
        detalhes["angulos_chegada"],
    )
    return fig, ax, detalhes


if __name__ == "__main__":
    num = [1, 2]
    den = [1, 5, 4, 0]
    lgr_completo(num, den, show_plot=True)
