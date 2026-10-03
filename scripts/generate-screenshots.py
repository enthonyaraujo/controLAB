#!/usr/bin/env python3
"""Gera capturas de tela WebP em alta resolução de todas as telas principais do ControLAB v3.1.0."""

import os
import subprocess
import sys
from pathlib import Path
from PIL import Image

PROJECT_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

from python_bridge import dispatch

DOCS_SCREENSHOTS = PROJECT_ROOT / "docs" / "screenshots"
PUBLIC_SCREENSHOTS = PROJECT_ROOT / "public" / "screenshots"
FF_PROFILE = Path("/tmp/ffprofile")

def prepare_firefox_profile():
    FF_PROFILE.mkdir(parents=True, exist_ok=True)
    (FF_PROFILE / "user.js").write_text(
        'user_pref("dom.serviceWorkers.enabled", false);\n'
        'user_pref("browser.cache.disk.enable", false);\n'
        'user_pref("browser.cache.memory.enable", false);\n'
    )

def render_html_to_webp(html_content: str, width: int, height: int, out_filename: str):
    tmp_html = Path(f"/tmp/render_{out_filename}.html")
    # Substitui caminhos relativos para absolutos
    html_fixed = html_content.replace('href="styles.css"', f'href="{PROJECT_ROOT}/src/renderer/styles.css"')
    html_fixed = html_fixed.replace('href="vendor/katex/katex.min.css"', f'href="{PROJECT_ROOT}/src/renderer/vendor/katex/katex.min.css"')
    html_fixed = html_fixed.replace('src="vendor/katex/katex.min.js"', f'src="{PROJECT_ROOT}/src/renderer/vendor/katex/katex.min.js"')
    # Remove scripts de comunicação IPC para render estático perfeito
    html_fixed = html_fixed.replace('<script src="app.js"></script>', '')
    html_fixed = html_fixed.replace('<script src="web-api.js"></script>', '')
    html_fixed = html_fixed.replace('<script src="navigation.js"></script>', '')
    html_fixed = html_fixed.replace('<script src="settings.js"></script>', '')
    html_fixed = html_fixed.replace('<script src="routh.js"></script>', '')
    html_fixed = html_fixed.replace('<script src="scientific-modules.js"></script>', '')

    tmp_html.write_text(html_fixed, encoding="utf-8")
    png_tmp = f"/tmp/{out_filename}.png"

    subprocess.run([
        "firefox", "--headless", "--no-remote",
        "-profile", str(FF_PROFILE),
        "--screenshot", png_tmp,
        f"--window-size={width},{height}",
        f"file://{tmp_html}"
    ], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    im = Image.open(png_tmp)
    if im.size != (width, height):
        im = im.crop((0, 0, width, height))

    docs_out = DOCS_SCREENSHOTS / out_filename
    public_out = PUBLIC_SCREENSHOTS / out_filename
    im.save(docs_out, format="WEBP", quality=90, method=6)
    im.save(public_out, format="WEBP", quality=90, method=6)
    print(f"✓ {out_filename} gerado ({docs_out.stat().st_size} bytes)")


def generate_all():
    prepare_firefox_profile()
    base_html = (PROJECT_ROOT / "src" / "renderer" / "index.html").read_text(encoding="utf-8")

    # 1. PRINCIPAL (1024 x 576)
    print("Gerando principal.webp...")
    html_principal = base_html
    render_html_to_webp(html_principal, 1024, 576, "principal.webp")

    # 2. HUB DE MÓDULOS (1024 x 529)
    print("Gerando hub-modulos.webp...")
    html_hub = base_html
    render_html_to_webp(html_hub, 1024, 529, "hub-modulos.webp")

    # Helper para telas de módulos científicos
    def build_scientific_page(title: str, form_inputs: str, plot_b64: str, metrics_html: str, tabs_html: str = ""):
        html = base_html
        # Navbar
        html = html.replace(
            '<div class="global-navbar-center" id="global-navbar-center" style="display: none;">\n      <span class="global-navbar-title" id="global-navbar-title"></span>',
            f'<div class="global-navbar-center" id="global-navbar-center" style="display: flex;">\n      <span class="global-navbar-title" id="global-navbar-title">{title}</span>'
        )
        html = html.replace('id="global-brand" class="global-brand"', 'id="global-brand" class="global-brand" style="display:none;"')
        html = html.replace('id="btn-nav-home" class="nav-icon-btn nav-back-btn" style="display: none;"', 'id="btn-nav-home" class="nav-icon-btn nav-back-btn" style="display: inline-flex;"')

        # Pages
        html = html.replace('id="page-home" class="app-page active"', 'id="page-home" class="app-page"')
        html = html.replace('id="page-scientific" class="app-page"', 'id="page-scientific" class="app-page active"')

        # Form
        html = html.replace('<form id="scientific-form" class="scientific-form"></form>', f'<form id="scientific-form" class="scientific-form">{form_inputs}</form>')

        # Results
        html = html.replace('<div id="scientific-empty" class="scientific-state scientific-empty-state">', '<div id="scientific-empty" class="scientific-state scientific-empty-state" hidden>')
        html = html.replace('<div id="scientific-results" class="scientific-results" hidden>', '<div id="scientific-results" class="scientific-results">')
        html = html.replace('id="scientific-status-chip" class="status-chip" style="display: none;"', 'id="scientific-status-chip" class="status-chip status-stable" style="display: inline-flex;"')
        html = html.replace('<img id="scientific-plot" alt="Gráfico da resposta do sistema" class="plot-img" />', f'<img id="scientific-plot" alt="Gráfico da resposta do sistema" class="plot-img" src="{plot_b64}" />')
        if tabs_html:
            html = html.replace('id="scientific-plot-tabs" class="tabs-segmented" style="display: none;"', f'id="scientific-plot-tabs" class="tabs-segmented" style="display: inline-flex;"')
        html = html.replace('<section id="scientific-primary-metrics" class="scientific-primary-grid"></section>', f'<section id="scientific-primary-metrics" class="scientific-primary-grid">{metrics_html}</section>')
        return html

    # 3. RESPOSTA NO TEMPO (1024 x 529)
    print("Gerando resposta-tempo.webp...")
    res_time = dispatch({'action': 'time_response', 'numerator': '4', 'denominator': 's^2 + 2*s + 4', 'theme': 'dark'})
    form_time = '''
    <div class="field-group"><label class="field-label">G(s)</label><input class="field-input" value="4 / (s^2 + 2*s + 4)"></div>
    <div class="field-group"><label class="field-label">Tempo final (s)</label><input class="field-input" value="10"></div>
    <div class="field-group"><label class="field-label">Acomodação</label><select class="field-select"><option>2%</option></select></div>
    '''
    metrics_time = '''
    <div class="metric-card"><span class="metric-label">Sobressinal (%OS)</span><span class="metric-value">16.3 %</span></div>
    <div class="metric-card"><span class="metric-label">Tempo de Pico (tp)</span><span class="metric-value">1.81 s</span></div>
    <div class="metric-card"><span class="metric-label">Acomodação (ts 2%)</span><span class="metric-value">3.91 s</span></div>
    <div class="metric-card"><span class="metric-label">Erro Degrau (ess)</span><span class="metric-value">0.000</span></div>
    '''
    html_time = build_scientific_page("Resposta no Domínio do Tempo", form_time, res_time['image'], metrics_time, tabs_html="time_tabs")
    render_html_to_webp(html_time, 1024, 529, "resposta-tempo.webp")

    # 4. LUGAR DAS RAÍZES (LGR) (1024 x 529)
    print("Gerando lugar-das-raizes.webp...")
    res_lgr = dispatch({'action': 'calculate', 'expr': '(s + 2) / (s * (s + 1) * (s + 4))', 'theme': 'dark'})
    html_lgr = base_html
    html_lgr = html_lgr.replace(
        '<div class="global-navbar-center" id="global-navbar-center" style="display: none;">\n      <span class="global-navbar-title" id="global-navbar-title"></span>',
        '<div class="global-navbar-center" id="global-navbar-center" style="display: flex;">\n      <span class="global-navbar-title" id="global-navbar-title">Lugar Geométrico das Raízes</span>'
    )
    html_lgr = html_lgr.replace('id="global-brand" class="global-brand"', 'id="global-brand" class="global-brand" style="display:none;"')
    html_lgr = html_lgr.replace('id="btn-nav-home" class="nav-icon-btn nav-back-btn" style="display: none;"', 'id="btn-nav-home" class="nav-icon-btn nav-back-btn" style="display: inline-flex;"')
    html_lgr = html_lgr.replace('id="page-home" class="app-page active"', 'id="page-home" class="app-page"')
    html_lgr = html_lgr.replace('id="page-lgr" class="app-page"', 'id="page-lgr" class="app-page active"')
    html_lgr = html_lgr.replace('<div id="plot-loading" class="loading-overlay">', '<div id="plot-loading" class="loading-overlay" style="display:none;">')
    html_lgr = html_lgr.replace('<img id="plot-img" alt="Lugar Geométrico das Raízes" class="plot-image" />', f'<img id="plot-img" alt="Lugar Geométrico das Raízes" class="plot-image" src="{res_lgr["image"]}" />')
    html_lgr = html_lgr.replace('id="badge-poles">Polos: <b>0</b>', 'id="badge-poles">Polos: <b>3</b>')
    html_lgr = html_lgr.replace('id="badge-zeros">Zeros: <b>0</b>', 'id="badge-zeros">Zeros: <b>1</b>')
    html_lgr = html_lgr.replace('id="badge-branches">Ramos: <b>0</b>', 'id="badge-branches">Ramos: <b>3</b>')
    html_lgr = html_lgr.replace('id="badge-centroid">Centroide: <b>-</b>', 'id="badge-centroid">Centroide: <b>-1.50</b>')
    render_html_to_webp(html_lgr, 1024, 529, "lugar-das-raizes.webp")

    # 5. RESPOSTA EM FREQUÊNCIA (1024 x 529)
    print("Gerando resposta-frequencia.webp...")
    res_freq = dispatch({'action': 'frequency_response', 'numerator': '10', 'denominator': 's * (s + 2) * (s + 5)', 'theme': 'dark'})
    form_freq = '''
    <div class="field-group"><label class="field-label">G(s)</label><input class="field-input" value="10 / (s * (s + 2) * (s + 5))"></div>
    <div class="field-group"><label class="field-label">Frequência mínima (rad/s)</label><input class="field-input" value="0.01"></div>
    <div class="field-group"><label class="field-label">Frequência máxima (rad/s)</label><input class="field-input" value="100"></div>
    '''
    metrics_freq = '''
    <div class="metric-card"><span class="metric-label">Margem de Ganho (MG)</span><span class="metric-value">16.9 dB</span></div>
    <div class="metric-card"><span class="metric-label">Cruzamento Ganho (ωcg)</span><span class="metric-value">0.89 rad/s</span></div>
    <div class="metric-card"><span class="metric-label">Margem de Fase (MF)</span><span class="metric-value">52.8 °</span></div>
    <div class="metric-card"><span class="metric-label">Cruzamento Fase (ωcf)</span><span class="metric-value">3.16 rad/s</span></div>
    '''
    html_freq = build_scientific_page("Resposta em Frequência", form_freq, res_freq['image'], metrics_freq)
    render_html_to_webp(html_freq, 1024, 529, "resposta-frequencia.webp")

    # 6. PROJETO DE CONTROLADORES (1024 x 529)
    print("Gerando projeto-controladores.webp...")
    res_ctrl = dispatch({'action': 'controller_design', 'plant_expr': '1 / (s * (s + 1) * (s + 5))', 'design_type': 'pid', 'method': 'zn_critical', 'controller_type': 'PID', 'critical_gain': 6, 'critical_period': 2, 'theme': 'dark'})
    form_ctrl = '''
    <div class="field-group"><label class="field-label">Planta G(s)</label><input class="field-input" value="1 / (s * (s + 1) * (s + 5))"></div>
    <div class="field-group"><label class="field-label">Tipo de projeto</label><select class="field-select"><option>Sintonia P / PI / PID</option></select></div>
    <div class="field-group"><label class="field-label">Método de sintonia</label><select class="field-select"><option>Ziegler-Nichols — oscilação crítica</option></select></div>
    <div class="field-group"><label class="field-label">Estrutura</label><select class="field-select"><option>PID</option></select></div>
    '''
    metrics_ctrl = '''
    <div class="metric-card"><span class="metric-label">Ganho Proporcional (Kp)</span><span class="metric-value">3.60</span></div>
    <div class="metric-card"><span class="metric-label">Tempo Integral (Ti)</span><span class="metric-value">1.00 s</span></div>
    <div class="metric-card"><span class="metric-label">Tempo Derivativo (Td)</span><span class="metric-value">0.25 s</span></div>
    <div class="metric-card"><span class="metric-label">Estabilidade MF</span><span class="metric-value">Estável</span></div>
    '''
    html_ctrl = build_scientific_page("Projeto de Controladores", form_ctrl, res_ctrl['image'], metrics_ctrl)
    render_html_to_webp(html_ctrl, 1024, 529, "projeto-controladores.webp")

    # 7. ESPAÇO DE ESTADOS (1024 x 529)
    print("Gerando espaco-estados.webp...")
    res_ss = dispatch({'action': 'state_space', 'A': '[[0, 1], [-2, -3]]', 'B': '[[0], [1]]', 'C': '[[1, 0]]', 'D': '[[0]]', 'canonical_form': 'controllable', 'desired_poles': '-4, -5', 'theme': 'dark'})
    form_ss = '''
    <div class="field-group"><label class="field-label">Forma de entrada</label><select class="field-select"><option>Matrizes A, B, C e D</option></select></div>
    <div class="field-group"><label class="field-label">Matriz A</label><textarea class="field-input" rows="2">[[0, 1], [-2, -3]]</textarea></div>
    <div class="field-group"><label class="field-label">Matriz B</label><textarea class="field-input" rows="2">[[0], [1]]</textarea></div>
    <div class="field-group"><label class="field-label">Matriz C</label><textarea class="field-input" rows="2">[[1, 0]]</textarea></div>
    '''
    metrics_ss = '''
    <div class="metric-card"><span class="metric-label">Posto Controlabilidade</span><span class="metric-value">2 (Completo)</span></div>
    <div class="metric-card"><span class="metric-label">Controlável</span><span class="metric-value">Sim</span></div>
    <div class="metric-card"><span class="metric-label">Posto Observabilidade</span><span class="metric-value">2 (Completo)</span></div>
    <div class="metric-card"><span class="metric-label">Observável</span><span class="metric-value">Sim</span></div>
    '''
    html_ss = build_scientific_page("Espaço de Estados", form_ss, res_ss['image'], metrics_ss)
    render_html_to_webp(html_ss, 1024, 529, "espaco-estados.webp")

    print("\n✓ Todas as 7 capturas de tela foram geradas e salvas com sucesso em docs/screenshots e public/screenshots!")

if __name__ == "__main__":
    generate_all()
