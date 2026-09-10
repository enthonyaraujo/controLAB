"""
Módulo de Análise de Estabilidade pelo Critério de Routh-Hurwitz
Calcula a tabela de Routh para funções de transferência e polinômios característicos
de qualquer grau (1 a n), tratando casos especiais (epsilon e linha nula) e determinando
a faixa analítica de ganho K para estabilidade.
"""

import json
import re
import sys
import sympy as sp
from sympy.parsing.sympy_parser import (
    parse_expr,
    standard_transformations,
    implicit_multiplication_application,
    convert_xor,
)

s_sym = sp.Symbol("s")
K_sym = sp.Symbol("K", real=True)
eps_sym = sp.Symbol("epsilon", positive=True)

_SUPERSCRIPT_DIGITS = str.maketrans("⁰¹²³⁴⁵⁶⁷⁸⁹⁻", "0123456789-")

ROUTH_PRESETS = {
    "Exemplo 1: 3ª Ordem com Faixa de Ganho K (Ogata)": {
        "expr": "1 / (s * (s + 1) * (s + 2))",
        "desc": "Sistema de 3ª ordem em malha fechada. Determina a faixa analítica 0 < K < 6 e ganho crítico K_crit = 6.",
    },
    "Exemplo 2: 3ª Ordem Estável (Numérico)": {
        "expr": "s^3 + 6*s^2 + 11*s + 6",
        "desc": "Polinômio cúbico sem trocas de sinal na 1ª coluna. Todos os polos no semiplano esquerdo (LHP).",
    },
    "Exemplo 3: 3ª Ordem Instável (2 Raízes no RHP)": {
        "expr": "s^3 + s^2 + 2*s + 24",
        "desc": "Duas trocas de sinal na primeira coluna (+, +, -, +), indicando 2 polos no semiplano direito.",
    },
    "Exemplo 4: 4ª Ordem com Primeiro Elemento Nulo (Caso ε)": {
        "expr": "s^4 + s^3 + 2*s^2 + 2*s + 3",
        "desc": "Elemento zero na linha s², contornado via limite infinitesimal com ε > 0.",
    },
    "Exemplo 5: 4ª Ordem com Linha de Zeros (Marginalmente Estável)": {
        "expr": "s^4 + 2*s^3 + 6*s^2 + 8*s + 8",
        "desc": "Linha nula em s¹, gerando polinômio auxiliar A(s) = 2s² + 8 e polos puramente imaginários em ±2j.",
    },
    "Exemplo 6: 5ª Ordem de Alta Complexidade": {
        "expr": "s^5 + 2*s^4 + 2*s^3 + 4*s^2 + 11*s + 10",
        "desc": "Sistema de 5ª ordem com caso especial de elemento nulo e 2 trocas de sinal.",
    },
}


def normalize_expression(expr_str: str) -> str:
    """Normaliza expressões matemáticas usuais permitindo '^', '2s', etc."""
    expr = (expr_str or "").strip()
    if not expr:
        raise ValueError("A expressão não pode estar vazia.")

    # Remove prefixos como G(s) =, P(s) =, etc.
    expr = re.sub(
        r"^\s*(?:[A-Za-z]\w*\s*(?:\([A-Za-z0-9\s,_]*\))?|[A-Za-z]\s*(?:\([A-Za-z0-9\s,_]*\))?\s*[A-Za-z]\s*(?:\([A-Za-z0-9\s,_]*\))?)\s*=\s*",
        "",
        expr,
    )
    # Remove '= 0' no final se houver
    expr = re.sub(r"\s*=\s*0\s*$", "", expr)

    expr = expr.translate(_SUPERSCRIPT_DIGITS)
    expr = expr.replace("^", "**")
    expr = expr.replace("–", "-").replace("—", "-")
    expr = re.sub(r"(\d)\s*,\s*(\d)", r"\1.\2", expr)

    # Multiplicação implícita básica: 2s -> 2*s, 3K -> 3*K
    expr = re.sub(r"(\d+)\s*([sSkK])", r"\1*\2", expr)
    expr = re.sub(r"([sSkK])\s*(\()", r"\1*\2", expr)
    expr = re.sub(r"(\))\s*([sSkK])", r"\1*\2", expr)

    return expr


def parse_poly_or_tf(input_str: str, has_k_loop: bool = True):
    """
    Interpreta uma entrada como polinômio direto ou como função de transferência G(s).
    Se for uma função de transferência G(s) = N(s)/D(s), formula a equação característica
    em malha fechada: 1 + K*G(s) = 0  =>  D(s) + K*N(s) = 0 (se has_k_loop=True e K estiver presente)
    ou D(s) + N(s) = 0 se malha fechada unitária.
    """
    normalized = normalize_expression(input_str)
    transformations = standard_transformations + (
        implicit_multiplication_application,
        convert_xor,
    )
    local_dict = {
        "s": s_sym,
        "S": s_sym,
        "k": K_sym,
        "K": K_sym,
        "j": sp.I,
        "i": sp.I,
    }

    try:
        expr = parse_expr(
            normalized, local_dict=local_dict, transformations=transformations
        )
    except Exception as exc:
        raise ValueError(f"Erro ao analisar a expressão matemática: {exc}")

    # Verifica se é uma fração racional N(s)/D(s)
    numer, denom = sp.fraction(sp.together(expr))
    is_fraction = denom != 1

    if is_fraction:
        # Se for fração racional G(s) = N(s)/D(s)
        # Se contiver K na expressão, 1 + G(s) = 0 => D(s) + N(s) = 0
        # Se não contiver K e has_k_loop for True, adiciona K*N(s)
        has_k_in_expr = bool(expr.free_symbols & {K_sym})
        if has_k_in_expr:
            char_poly = sp.expand(denom + numer)
        elif has_k_loop:
            char_poly = sp.expand(denom + K_sym * numer)
        else:
            char_poly = sp.expand(denom + numer)
    else:
        char_poly = sp.expand(expr)

    # Coleta termos em relação a 's'
    poly_s = sp.Poly(char_poly, s_sym)
    degree = poly_s.degree()

    if degree < 1:
        raise ValueError("O grau do polinômio característico deve ser pelo menos 1.")

    coeffs = poly_s.all_coeffs()
    return char_poly, poly_s, degree, coeffs, is_fraction, numer, denom


def format_sympy_latex(expr) -> str:
    """Formata expressão SymPy para KaTeX limpo sem notações indesejadas."""
    if expr is None:
        return ""
    ltx = sp.latex(expr)
    ltx = ltx.replace(r"\epsilon", r"\varepsilon")
    return ltx


def format_poly_descending(poly_s) -> str:
    """Formata polinômio em ordem estritamente decrescente das potências de s."""
    deg = poly_s.degree()
    coeffs = poly_s.all_coeffs()
    terms = []
    for i, c in enumerate(coeffs):
        p = deg - i
        if c == 0:
            continue
        c_latex = format_sympy_latex(c)
        if p == 0:
            terms.append((c, c_latex, 0))
        elif p == 1:
            if c == 1:
                t = "s"
            elif c == -1:
                t = "-s"
            elif c.is_Add:
                t = f"({c_latex}) s"
            else:
                t = f"{c_latex} s"
            terms.append((c, t, 1))
        else:
            if c == 1:
                t = f"s^{{{p}}}"
            elif c == -1:
                t = f"-s^{{{p}}}"
            elif c.is_Add:
                t = f"({c_latex}) s^{{{p}}}"
            else:
                t = f"{c_latex} s^{{{p}}}"
            terms.append((c, t, p))

    if not terms:
        return "0"

    out = terms[0][1]
    for c, t, p in terms[1:]:
        t_clean = t.strip()
        if t_clean.startswith("-"):
            out += f" - {t_clean[1:].strip()}"
        else:
            out += f" + {t_clean}"
    return out


def build_routh_table(coeffs, degree):
    """
    Constrói a tabela de Routh completa para um polinômio com coeficientes dados.
    Retorna a matriz de termos, detalhes de cada célula, casos especiais e etapas.
    """
    n = degree
    m = (n + 2) // 2

    # Inicializa as linhas da tabela
    # rows[i] corresponderá a s^(n - i) para i = 0 .. n
    rows = []
    cell_details = []
    special_cases = []
    steps = []

    # Linha 0: a_n, a_{n-2}, a_{n-4}, ...
    r0 = []
    for j in range(0, n + 1, 2):
        r0.append(coeffs[j])
    while len(r0) < m:
        r0.append(sp.Integer(0))

    # Linha 1: a_{n-1}, a_{n-3}, a_{n-5}, ...
    r1 = []
    for j in range(1, n + 1, 2):
        r1.append(coeffs[j])
    while len(r1) < m:
        r1.append(sp.Integer(0))

    rows.append(r0)
    rows.append(r1)

    steps.append({
        "power": f"s^{{{n}}}",
        "desc": f"Primeira linha formada pelos coeficientes de potências de mesma paridade de s^{{{n}}}.",
        "values": [format_sympy_latex(c) for c in r0],
    })
    steps.append({
        "power": f"s^{{{n-1}}}",
        "desc": f"Segunda linha formada pelos coeficientes de potências alternadas a partir de s^{{{n-1}}}.",
        "values": [format_sympy_latex(c) for c in r1],
    })

    # Calcula linhas subsequentes de s^(n-2) até s^0
    for i in range(2, n + 1):
        power_num = n - i
        power_label = f"s^{{{power_num}}}" if power_num > 1 else ("s" if power_num == 1 else "s^{0}")

        prev_row = rows[i - 1]
        prev_prev_row = rows[i - 2]

        # Caso especial 2: Toda a linha anterior é identicamente nula
        is_all_zero = all(c == 0 for c in prev_row)
        if is_all_zero:
            # Polinômio auxiliar formado pela linha i - 2
            aux_power = n - (i - 2)
            aux_terms = []
            cur_p = aux_power
            for c in prev_prev_row:
                if cur_p >= 0 and c != 0:
                    aux_terms.append(c * s_sym**cur_p)
                cur_p -= 2

            aux_poly = sum(aux_terms)
            aux_deriv = sp.diff(aux_poly, s_sym)
            deriv_poly = sp.Poly(aux_deriv, s_sym)
            deriv_coeffs = deriv_poly.all_coeffs()

            # Preenche a linha anterior com os coeficientes da derivada
            new_prev_row = []
            for c in deriv_coeffs[::2]:  # potências decrescem de 2 em 2
                new_prev_row.append(c)
            while len(new_prev_row) < m:
                new_prev_row.append(sp.Integer(0))

            rows[i - 1] = new_prev_row
            prev_row = new_prev_row

            note = (
                f"Linha de zeros detectada em s^{{{n - i + 1}}}. "
                f"Polinômio auxiliar: A(s) = {format_sympy_latex(aux_poly)}. "
                f"Derivada substituída: dA/ds = {format_sympy_latex(aux_deriv)}."
            )
            special_cases.append({
                "type": "row_of_zeros",
                "power": f"s^{{{n - i + 1}}}",
                "aux_poly_latex": format_sympy_latex(aux_poly),
                "aux_deriv_latex": format_sympy_latex(aux_deriv),
                "desc": note,
            })
            steps.append({
                "power": f"s^{{{n - i + 1}}} (Ajustada)",
                "desc": note,
                "values": [format_sympy_latex(c) for c in prev_row],
            })

        # Caso especial 1: Primeiro elemento da linha anterior é zero, mas a linha não é toda nula
        elif prev_row[0] == 0:
            prev_row[0] = eps_sym
            note = (
                f"Primeiro elemento nulo em s^{{{n - i + 1}}}. "
                f"Substituído por termo infinitesimal positivo ε > 0."
            )
            special_cases.append({
                "type": "zero_first_element",
                "power": f"s^{{{n - i + 1}}}",
                "desc": note,
            })

        # Calcula a nova linha i
        cur_row = []
        piv = prev_row[0]

        for j in range(m):
            if j + 1 < m:
                a1 = prev_prev_row[0]
                a2 = prev_prev_row[j + 1]
                b1 = prev_row[0]
                b2 = prev_row[j + 1]

                # Fórmula clássica de Routh: (b1*a2 - a1*b2) / b1
                num = b1 * a2 - a1 * b2
                val = sp.simplify(num / piv)
                cur_row.append(val)

                formula_ltx = (
                    rf"\frac{{({format_sympy_latex(b1)})({format_sympy_latex(a2)}) - "
                    rf"({format_sympy_latex(a1)})({format_sympy_latex(b2)})}}"
                    rf"{{{format_sympy_latex(b1)}}}"
                )
                cell_details.append({
                    "row": i,
                    "col": j,
                    "power": power_label,
                    "formula": formula_ltx,
                    "value_latex": format_sympy_latex(val),
                })
            else:
                cur_row.append(sp.Integer(0))

        rows.append(cur_row)
        steps.append({
            "power": power_label,
            "desc": f"Linha calculada a partir dos determinantes das duas linhas anteriores.",
            "values": [format_sympy_latex(c) for c in cur_row],
        })

    return rows, special_cases, steps, cell_details


def evaluate_signs_and_stability(first_col, special_cases):
    """
    Avalia os sinais da primeira coluna da tabela de Routh,
    calcula o número de trocas de sinal e determina o veredito de estabilidade.
    """
    evaluated_signs = []
    sign_changes = 0

    for idx, term in enumerate(first_col):
        # Se contiver epsilon, avalia o limite quando epsilon -> 0+
        if term.has(eps_sym):
            val_limit = sp.limit(term, eps_sym, 0, dir="+")
        else:
            val_limit = term

        if val_limit.is_number:
            v_float = float(val_limit.evalf())
            if abs(v_float) < 1e-12:
                sign = "0"
            elif v_float > 0:
                sign = "+"
            else:
                sign = "-"
        else:
            # Expressão simbólica contendo K
            sign = "expr"

        evaluated_signs.append({
            "term_latex": format_sympy_latex(term),
            "sign": sign,
            "is_symbolic": not val_limit.is_number,
        })

    # Conta trocas de sinal se todos forem numéricos
    has_symbolic = any(s["is_symbolic"] for s in evaluated_signs)
    if not has_symbolic:
        # Filtra elementos estritamente não nulos para contagem clássica
        non_zero_signs = [s["sign"] for s in evaluated_signs if s["sign"] in ("+", "-")]
        for i in range(len(non_zero_signs) - 1):
            if non_zero_signs[i] != non_zero_signs[i + 1]:
                sign_changes += 1

        has_row_of_zeros = any(sc["type"] == "row_of_zeros" for sc in special_cases)
        has_zeros_in_col = any(s["sign"] == "0" for s in evaluated_signs)

        if sign_changes == 0 and not has_row_of_zeros and not has_zeros_in_col:
            verdict = "Estável"
            summary = (
                "Todos os elementos da primeira coluna possuem o mesmo sinal (sem trocas de sinal). "
                "O sistema é BIBO estável e todas as raízes situam-se estritamente no semiplano esquerdo (LHP)."
            )
        elif sign_changes == 0 and has_row_of_zeros:
            verdict = "Marginalmente Estável"
            summary = (
                "Não ocorrem trocas de sinal fora da linha de zeros, indicando a existência de pares de polos "
                "puramente imaginários sobre o eixo jω (sem amortecimento). O sistema oscila com amplitude sustentada."
            )
        else:
            verdict = "Instável"
            rhp_count = sign_changes
            summary = (
                f"Ocorrem {sign_changes} troca(s) de sinal na primeira coluna da tabela de Routh, "
                f"o que indica exatamente {rhp_count} polo(s) em malha fechada no semiplano direito (RHP)."
            )
    else:
        verdict = "Dependente de K"
        summary = (
            "A estabilidade depende do ganho escalar K. "
            "Consulte a seção de Faixa de K para as desigualdades analíticas."
        )

    return evaluated_signs, sign_changes, verdict, summary


def compute_k_range(first_col, poly_s):
    """
    Determina a faixa analítica de K para estabilidade resolvendo o sistema
    de inequações onde todos os termos da 1ª coluna devem ter o mesmo sinal do coeficiente líder.
    """
    k_terms = [term for term in first_col if term.has(K_sym)]
    if not k_terms:
        return {
            "has_k": False,
            "range_latex": "Não aplicável (sem ganho K variável)",
            "conditions": [],
            "critical_k": [],
        }

    # Coeficiente líder (a_n)
    lead_coeff = first_col[0]
    lead_positive = lead_coeff.is_positive if lead_coeff.is_number else True

    conditions = []
    critical_k = []

    for term in first_col:
        if term.has(K_sym):
            # Condição term > 0 (ou < 0 se lead_coeff < 0)
            ineq_latex = f"{format_sympy_latex(term)} > 0" if lead_positive else f"{format_sympy_latex(term)} < 0"
            conditions.append({
                "term_latex": format_sympy_latex(term),
                "inequality_latex": ineq_latex,
            })

            # Encontra zeros do termo para determinar valores críticos de K
            try:
                roots_k = sp.solve(term, K_sym)
                for rk in roots_k:
                    if rk.is_real:
                        r_float = float(rk.evalf())
                        if r_float > 0:
                            critical_k.append({
                                "k_val": r_float,
                                "k_latex": format_sympy_latex(rk),
                            })
            except Exception:
                pass

    # Resolve o sistema de inequações usando intervalos de teste
    # Coleta todos os pontos críticos positivos de K
    crit_points = sorted(list(set([ck["k_val"] for ck in critical_k if ck["k_val"] > 0])))
    all_points = [0.0] + crit_points

    valid_intervals = []
    for idx in range(len(all_points)):
        p_low = all_points[idx]
        p_high = all_points[idx + 1] if idx + 1 < len(all_points) else None

        # Ponto de teste no interior do intervalo
        if p_high is not None:
            k_test = (p_low + p_high) / 2.0
        else:
            k_test = p_low + 10.0

        all_ok = True
        for term in first_col:
            val = term.subs(K_sym, k_test)
            if val.has(eps_sym):
                val = sp.limit(val, eps_sym, 0, dir="+")
            try:
                v_num = float(val.evalf())
                if (lead_positive and v_num <= 1e-12) or (not lead_positive and v_num >= -1e-12):
                    all_ok = False
                    break
            except Exception:
                all_ok = False
                break

        if all_ok:
            if p_high is not None:
                if p_low == 0.0:
                    valid_intervals.append(f"0 < K < {p_high:.4g}")
                else:
                    valid_intervals.append(f"{p_low:.4g} < K < {p_high:.4g}")
            else:
                valid_intervals.append(f"K > {p_low:.4g}")

    if valid_intervals:
        range_latex = r" \quad \text{ou} \quad ".join(valid_intervals)
    else:
        range_latex = r"\text{Nenhuma faixa estável com } K > 0"

    # Frequência de oscilação nos ganhos críticos
    for ck in critical_k:
        k_val = ck["k_val"]
        try:
            # Avalia raízes do polinômio no ganho crítico
            crit_poly = poly_s.as_expr().subs(K_sym, k_val)
            roots = sp.solve(crit_poly, s_sym)
            jw_roots = []
            for r in roots:
                re_val = float(sp.re(r).evalf())
                im_val = float(sp.im(r).evalf())
                if abs(re_val) < 1e-5 and abs(im_val) > 1e-5:
                    jw_roots.append(abs(im_val))
            if jw_roots:
                omega_osc = min(jw_roots)
                ck["omega_osc"] = round(omega_osc, 4)
                ck["omega_latex"] = f"\\omega_{{osc}} = {omega_osc:.4g} \\text{{ rad/s}}"
        except Exception:
            pass

    return {
        "has_k": True,
        "range_latex": range_latex,
        "conditions": conditions,
        "critical_k": critical_k,
    }


def analyze_routh_hurwitz(input_str: str, has_k_loop: bool = True):
    """
    Função principal de análise completa do Critério de Routh-Hurwitz.
    Retorna estrutura serializável em JSON com tabela, deduções e conclusões.
    """
    char_poly, poly_s, degree, coeffs, is_fraction, numer, denom = parse_poly_or_tf(
        input_str, has_k_loop=has_k_loop
    )

    rows, special_cases, steps, cell_details = build_routh_table(coeffs, degree)

    first_col = [r[0] for r in rows]
    evaluated_signs, sign_changes, verdict, summary = evaluate_signs_and_stability(
        first_col, special_cases
    )

    k_info = compute_k_range(first_col, poly_s)

    # Formata as linhas para serialização
    formatted_table = []
    n = degree
    for i, row in enumerate(rows):
        power_num = n - i
        power_label = (
            f"s^{{{power_num}}}" if power_num > 1 else ("s" if power_num == 1 else "s^{0}")
        )
        cells = []
        for j, val in enumerate(row):
            cells.append({
                "col": j,
                "latex": format_sympy_latex(val),
                "is_first_col": j == 0,
                "sign": evaluated_signs[i]["sign"] if j == 0 else "",
            })
        formatted_table.append({
            "power": power_label,
            "row_index": i,
            "cells": cells,
        })

    # Raízes exatas para referência (quando puramente numérico)
    exact_roots = []
    if not any(c.has(K_sym) for c in coeffs):
        try:
            poly_numeric = [float(sp.N(c)) for c in coeffs]
            import numpy as np
            roots_np = np.roots(poly_numeric)
            for r in roots_np:
                exact_roots.append({
                    "real": round(float(np.real(r)), 4),
                    "imag": round(float(np.imag(r)), 4),
                    "str": f"{np.real(r):.4g}" + (f" + {np.imag(r):.4g}j" if np.imag(r) >= 0 else f" - {abs(np.imag(r)):.4g}j") if abs(np.imag(r)) > 1e-6 else f"{np.real(r):.4g}",
                })
        except Exception:
            pass

    return {
        "success": True,
        "input_str": input_str,
        "is_fraction": is_fraction,
        "char_poly_latex": format_poly_descending(poly_s) + " = 0",
        "degree": degree,
        "coeffs_latex": [format_sympy_latex(c) for c in coeffs],
        "routh_table": formatted_table,
        "special_cases": special_cases,
        "steps": steps,
        "cell_details": cell_details,
        "first_col_signs": evaluated_signs,
        "sign_changes": sign_changes,
        "rhp_poles": sign_changes,
        "verdict": verdict,
        "summary": summary,
        "k_range": k_info,
        "exact_roots": exact_roots,
    }


def main():
    """CLI para execução direta ou teste via terminal."""
    try:
        if len(sys.argv) > 1:
            raw_input = sys.argv[1]
        elif not sys.stdin.isatty():
            raw_input = sys.stdin.read().strip()
        else:
            raw_input = "s^4 + 2*s^3 + 3*s^2 + 4*s + 5"

        if not raw_input:
            raw_input = "s^4 + 2*s^3 + 3*s^2 + 4*s + 5"

        # Suporta tanto JSON {"input": "..."} quanto string direta
        if raw_input.startswith("{"):
            data = json.loads(raw_input)
            expr = data.get("input") or data.get("expr") or data.get("den") or "s^3 + 2s^2 + s + 1"
            has_k = data.get("has_k", True)
        else:
            expr = raw_input
            has_k = True

        result = analyze_routh_hurwitz(expr, has_k_loop=has_k)
        print(json.dumps(result, indent=2))
    except Exception as exc:
        print(json.dumps({"success": False, "error": str(exc)}))
        sys.exit(1)


if __name__ == "__main__":
    main()
