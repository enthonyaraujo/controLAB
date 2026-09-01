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


def lgr_completo(num, den, titulo="Lugar Geométrico das Raízes", show_plot=False):
    """
    Gera um gráfico do LGR autossuficiente com cálculos automáticos 
    para todos os 7 passos clássicos da análise, incluindo legendas dinâmicas.
    
    Retorna: (fig, ax, detalhes_dos_passos)
    """
    sys = ct.tf(num, den)
    
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
    kvect = np.logspace(-3, 3, 5000)
    kvect = np.insert(kvect, 0, 0)
    rlist, klist = ct.root_locus(sys, kvect=kvect, plot=False)
    
    # Configuração da Figura
    fig, ax = plt.subplots(figsize=(11, 7.5), dpi=100)
    ax.grid(True, linestyle=':', alpha=0.7)
    ax.axhline(0, color='black', linewidth=1.2) # Eixo Real
    ax.axvline(0, color='black', linewidth=1.2) # Eixo jw
    
    # ========================================================
    # PASSO 1, 2 e 3: Ramos, Polos, Zeros e Simetria
    # ========================================================
    # Traça os ramos (usamos label apenas no primeiro para não duplicar na legenda)
    for i in range(ramos):
        ax.plot(np.real(rlist[:, i]), np.imag(rlist[:, i]), color='red', linewidth=2, 
                label='Ramos do LGR' if i == 0 else "_nolegend_")
        
    # Marca polos (x)
    ax.plot(np.real(polos), np.imag(polos), 'kx', markersize=8, markeredgewidth=2, label='Polos (Início)')
    
    # Marca zeros (o) garantindo que a legenda apareça mesmo se não houver zeros
    if Z > 0:
        ax.plot(np.real(zeros), np.imag(zeros), 'ko', markerfacecolor='white', markersize=8, markeredgewidth=2, label='Zeros (Término)')
    else:
        ax.plot([], [], 'ko', markerfacecolor='white', markersize=8, markeredgewidth=2, label='Zeros (Nenhum nesta FT)')

    for p in polos:
        ax.text(np.real(p) - 0.2, np.imag(p) + 0.3, f'p={np.round(p,2)}', fontsize=10)
    for z in zeros:
        ax.text(np.real(z) - 0.2, np.imag(z) + 0.3, f'z={np.round(z,2)}', fontsize=10)

    # ========================================================
    # PASSO 4: Assíntotas e Centroide
    # ========================================================
    centroide = None
    if P > Z:
        centroide = np.real((np.sum(polos) - np.sum(zeros)) / (P - Z))
        detalhes["centroide"] = centroide
        ax.plot(centroide, 0, 'k+', markersize=10, label=rf'Centroide ($\sigma_a = {centroide:.2f}$)')
        ax.text(centroide, -0.5, rf'Centroide' + '\n' + rf'$\sigma_a = {centroide:.2f}$', ha='center')
        
        raio = 50
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
            ax.plot([centroide, x_end], [0, y_end], color='grey', linestyle='--', zorder=0,
                    label='Assíntotas' if k == 0 else "_nolegend_")
            
            # Anotação do ângulo na borda
            ax.text(centroide + 5*np.cos(ang_rad), 5*np.sin(ang_rad), f'{ang_deg:.0f}°', 
                    bbox=dict(facecolor='white', edgecolor='none', alpha=0.7))

    # ========================================================
    # PASSO 5: Pontos de Entrada e Saída (Break-in / Breakaway)
    # ========================================================
    N = np.poly1d(num)
    D = np.poly1d(den)
    dN = np.polyder(N)
    dD = np.polyder(D)
    eq_break = np.polysub(np.polymul(dN, D), np.polymul(N, dD))
    
    raizes_break = np.roots(eq_break)
    break_adicionado = False
    
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
                    lbl = f'Break-in/out (K={K_break:.1f})' if not break_adicionado else "_nolegend_"
                    ax.plot(np.real(r), 0, 'bs', markersize=6, label=lbl)
                    ax.text(np.real(r), 0.5, f'Break\n$s={np.real(r):.2f}$\n$K={K_break:.1f}$', ha='center', color='blue')
                    break_adicionado = True

    # ========================================================
    # PASSO 6: Cruzamento do eixo jw
    # ========================================================
    cruz_adicionado = False
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
                    # Adiciona a legenda apenas para o primeiro par conjugado encontrado
                    lbl = rf'Cruz. j$\omega$ ($\omega=\pm${abs(w_cruz):.2f}, K={K_cruz:.1f})' if not cruz_adicionado else "_nolegend_"
                    ax.plot(0, w_cruz, 'ro', markerfacecolor='white', markersize=6, label=lbl)
                    ax.text(0.2, w_cruz, rf'$j\omega$ Cruz.' + '\n' + rf'$\omega={w_cruz:.2f}$' + '\n' + rf'$K={K_cruz:.1f}$', color='maroon')
                    cruz_adicionado = True

    # ========================================================
    # PASSO 7: Ângulos de Partida (Polos) e Chegada (Zeros)
    # ========================================================
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
            
            arco = Arc((np.real(p), np.imag(p)), 2, 2, angle=0, theta1=min(0, angulo), theta2=max(0, angulo), color='purple', lw=1.5)
            ax.add_patch(arco)
            ax.plot([np.real(p)-1, np.real(p)+1], [np.imag(p), np.imag(p)], 'k:', alpha=0.5)
            ax.text(np.real(p) + 1.2, np.imag(p), rf'$\theta_d={angulo:.1f}^\circ$', color='purple')

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
            
            arco = Arc((np.real(z), np.imag(z)), 2, 2, angle=0, theta1=min(0, angulo), theta2=max(0, angulo), color='teal', lw=1.5)
            ax.add_patch(arco)
            ax.plot([np.real(z)-1, np.real(z)+1], [np.imag(z), np.imag(z)], 'k:', alpha=0.5)
            ax.text(np.real(z) + 1.2, np.imag(z), rf'$\theta_a={angulo:.1f}^\circ$', color='teal')

    # ========================================================
    # AJUSTES FINAIS DE ESTÉTICA E LEGENDA
    # ========================================================
    todas_raizes = np.concatenate([polos, zeros])
    if P > Z and centroide is not None:
        todas_raizes = np.append(todas_raizes, centroide)
    
    if len(todas_raizes) > 0:
        ax.set_xlim(np.min(np.real(todas_raizes)) - 4, max(2, np.max(np.real(todas_raizes)) + 2))
        max_y = max(4, np.max(np.abs(np.imag(todas_raizes))) + 2)
        ax.set_ylim(-max_y, max_y)

    ax.set_title(titulo, fontsize=14, pad=15)
    ax.set_xlabel(r'Eixo Real ($\sigma$)', fontsize=12)
    ax.set_ylabel(r'Eixo Imaginário ($j\omega$)', fontsize=12)
    
    # Posiciona a legenda fora dos ramos principais
    ax.legend(loc='lower left', bbox_to_anchor=(1.02, 0.5), borderaxespad=0, title="Componentes do LGR")
    
    plt.tight_layout()
    if show_plot:
        plt.show()
    return fig, ax, detalhes


if __name__ == "__main__":
    num = [1, 2]
    den = [1, 5, 4, 0]
    lgr_completo(num, den, show_plot=True)
