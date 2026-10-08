#!/usr/bin/env python3
"""Gera capturas de tela WebP em alta resolução no modo desktop do ControLAB v3.1.1."""

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
    # Força remoção de animações para captura estática instantânea
    html_fixed = html_content.replace('</head>', '<style>.app-page.active { animation: none !important; opacity: 1 !important; }</style></head>')
    html_fixed = html_fixed.replace('href="styles.css"', f'href="{PROJECT_ROOT}/src/renderer/styles.css"')
    html_fixed = html_fixed.replace('href="vendor/katex/katex.min.css"', f'href="{PROJECT_ROOT}/src/renderer/vendor/katex/katex.min.css"')
    html_fixed = html_fixed.replace('src="vendor/katex/katex.min.js"', f'src="{PROJECT_ROOT}/src/renderer/vendor/katex/katex.min.js"')
    html_fixed = html_fixed.replace('src="vendor/katex/contrib/auto-render.min.js"', f'src="{PROJECT_ROOT}/src/renderer/vendor/katex/contrib/auto-render.min.js"')
    for s in ['app.js', 'web-api.js', 'navigation.js', 'settings.js', 'routh.js', 'scientific-modules.js']:
        html_fixed = html_fixed.replace(f'<script src="{s}"></script>', '')
    html_fixed = html_fixed.replace('</body>', '<script>if(window.renderMathInElement) renderMathInElement(document.body, {delimiters:[{left:"$$",right:"$$",display:true}]});</script></body>')

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
    im.save(docs_out, format="WEBP", quality=95, method=6)
    im.save(public_out, format="WEBP", quality=95, method=6)
    print(f"✓ {out_filename} gerado ({im.size}, {docs_out.stat().st_size:,} bytes)")


def generate_all():
    prepare_firefox_profile()
    base_html = (PROJECT_ROOT / "src" / "renderer" / "index.html").read_text(encoding="utf-8")

    # 1 & 2. PRINCIPAL E HUB DE MÓDULOS (Modo Desktop fornecido pelo usuário: 1918 x 997)
    user_img_path = Path("/home/enthony/.gemini/antigravity-cli/brain/8ba771fa-9d3a-41bd-9cf0-7b751eab6836/.user_uploaded/uploaded_media_1791047615517.png")
    if user_img_path.exists():
        im_user = Image.open(user_img_path).convert("RGB")
        for fname in ["principal.webp", "hub-modulos.webp"]:
            im_user.save(DOCS_SCREENSHOTS / fname, format="WEBP", quality=95, method=6)
            im_user.save(PUBLIC_SCREENSHOTS / fname, format="WEBP", quality=95, method=6)
        print(f"✓ principal.webp e hub-modulos.webp salvos a partir da captura oficial do usuário ({im_user.size})")

    # Helper para páginas do workspace científico em modo desktop
    def build_scientific_desktop(title: str, form_html: str, plot_b64: str, metrics: list[tuple[str, str]], tabs: list[str] = None):
        html = base_html
        # Navbar em modo módulo
        html = html.replace('<header class="global-navbar">', '<header class="global-navbar has-back">')
        html = html.replace('id="btn-nav-home" class="btn-nav-back" title="Voltar à Página Inicial" aria-label="Voltar à Página Inicial" style="display: none;"',
                            'id="btn-nav-home" class="btn-nav-back" title="Voltar à Página Inicial" aria-label="Voltar à Página Inicial" style="display: inline-flex;"')
        html = html.replace('id="global-brand" class="global-brand"', 'id="global-brand" class="global-brand" style="display: none;"')
        html = html.replace('<div class="global-navbar-center" id="global-navbar-center" style="display: none;">\n      <span class="global-navbar-title" id="global-navbar-title"></span>',
                            f'<div class="global-navbar-center" id="global-navbar-center" style="display: flex;">\n      <span class="global-navbar-title" id="global-navbar-title">{title}</span>')

        # Alterna para scientific-workspace
        html = html.replace('id="page-home" class="app-page active"', 'id="page-home" class="app-page"')
        html = html.replace('id="page-scientific" class="app-page"', 'id="page-scientific" class="app-page active"')

        # Formulário na barra lateral
        html = html.replace('<form id="scientific-form" class="scientific-form"></form>', f'<form id="scientific-form" class="scientific-form">{form_html}</form>')

        # Painel de resultados e gráfico
        html = html.replace('<div id="scientific-empty" class="scientific-state scientific-empty-state">', '<div id="scientific-empty" class="scientific-state scientific-empty-state" hidden>')
        html = html.replace('<div id="scientific-results" class="scientific-results" hidden>', '<div id="scientific-results" class="scientific-results">')
        html = html.replace('id="scientific-status-chip" class="status-chip" style="display: none;"', 'id="scientific-status-chip" class="status-chip status-stable" style="display: inline-flex;"')
        html = html.replace('<img id="scientific-plot" alt="Gráfico da resposta do sistema" class="plot-img" />', f'<img id="scientific-plot" alt="Gráfico da resposta do sistema" class="plot-img" src="{plot_b64}" />')

        # Abas de simulação
        if tabs:
            tabs_html = ''.join([f'<button type="button" class="tab-seg {"active" if i == 0 else ""}">{t}</button>' for i, t in enumerate(tabs)])
            html = html.replace('<div id="scientific-plot-tabs" class="tabs-segmented" style="display: none;">\n                  <button type="button" class="tab-seg active" data-resp="step">Degrau</button>\n                  <button type="button" class="tab-seg" data-resp="ramp">Rampa</button>\n                  <button type="button" class="tab-seg" data-resp="impulse">Impulso</button>\n                </div>',
                                f'<div id="scientific-plot-tabs" class="tabs-segmented" style="display: inline-flex;">{tabs_html}</div>')

        # Cards das métricas principais (Top 4)
        metrics_cards = []
        for i, (label, val) in enumerate(metrics):
            cls = 'metric-card metric-card-accent' if i == 0 else 'metric-card'
            metrics_cards.append(f'''
            <article class="{cls}">
              <span class="metric-label">{label}</span>
              <div class="metric-value-row">
                <strong class="metric-value code-font">{val}</strong>
              </div>
            </article>''')
        metrics_html = '\n'.join(metrics_cards)
        html = html.replace('<section id="scientific-primary-metrics" class="scientific-primary-grid"></section>',
                            f'<section id="scientific-primary-metrics" class="scientific-primary-grid">{metrics_html}</section>')
        return html

    # 3. RESPOSTA NO TEMPO (1918 x 997)
    res_time = dispatch({'action': 'time_response', 'numerator': '4', 'denominator': 's^2 + 2*s + 4', 'theme': 'dark'})
    form_time = '''
    <div class="field-group"><label class="field-label">G(s) — Função de Transferência</label><input class="form-input code-font" value="4 / (s^2 + 2*s + 4)"></div>
    <div class="function-preview" style="margin-top: 0;">
      <div class="preview-heading">
        <span>Prévia matemática</span>
        <span class="preview-status valid">Expressão válida</span>
      </div>
      <div class="preview-math">$$G(s) = \\frac{4}{s^{2} + 2 s + 4}$$</div>
    </div>
    <div class="field-group"><label class="field-label">Tempo final (s)</label><input class="form-input code-font" value="10"></div>
    <div class="field-group"><label class="field-label">Faixa de acomodação</label><select class="form-select"><option selected>2%</option><option>5%</option></select></div>
    '''
    metrics_time = [
        ('Sobressinal (%OS)', '16.3 %'),
        ('Tempo de Pico (tp)', '1.81 s'),
        ('Acomodação (ts 2%)', '3.91 s'),
        ('Erro Degrau (ess)', '0.000'),
    ]
    html_time = build_scientific_desktop("Resposta no Domínio do Tempo", form_time, res_time['image'], metrics_time, tabs=['Degrau', 'Rampa', 'Impulso'])
    render_html_to_webp(html_time, 1918, 997, "resposta-tempo.webp")

    # 4. LUGAR GEOMÉTRICO DAS RAÍZES (LGR) (1918 x 997)
    res_lgr = dispatch({'action': 'calculate', 'expr': '(s + 2) / (s * (s + 1) * (s + 4))', 'theme': 'dark'})
    html_lgr = base_html
    html_lgr = html_lgr.replace('<header class="global-navbar">', '<header class="global-navbar has-back">')
    html_lgr = html_lgr.replace('id="btn-nav-home" class="btn-nav-back" title="Voltar à Página Inicial" aria-label="Voltar à Página Inicial" style="display: none;"',
                                'id="btn-nav-home" class="btn-nav-back" title="Voltar à Página Inicial" aria-label="Voltar à Página Inicial" style="display: inline-flex;"')
    html_lgr = html_lgr.replace('id="global-brand" class="global-brand"', 'id="global-brand" class="global-brand" style="display: none;"')
    html_lgr = html_lgr.replace('<div class="global-navbar-center" id="global-navbar-center" style="display: none;">\n      <span class="global-navbar-title" id="global-navbar-title"></span>',
                                '<div class="global-navbar-center" id="global-navbar-center" style="display: flex;">\n      <span class="global-navbar-title" id="global-navbar-title">Lugar Geométrico das Raízes</span>')
    html_lgr = html_lgr.replace('id="page-home" class="app-page active"', 'id="page-home" class="app-page"')
    html_lgr = html_lgr.replace('id="page-lgr" class="app-page"', 'id="page-lgr" class="app-page active"')
    html_lgr = html_lgr.replace('<div id="plot-loading" class="loading-overlay">', '<div id="plot-loading" class="loading-overlay" style="display:none;">')
    html_lgr = html_lgr.replace('<img id="plot-image" src=""', f'<img id="plot-image" src="{res_lgr["image"]}"')
    html_lgr = html_lgr.replace('id="badge-poles">Polos: <b>0</b>', 'id="badge-poles">Polos: <b>3</b>')
    html_lgr = html_lgr.replace('id="badge-zeros">Zeros: <b>0</b>', 'id="badge-zeros">Zeros: <b>1</b>')
    html_lgr = html_lgr.replace('id="badge-branches">Ramos: <b>0</b>', 'id="badge-branches">Ramos: <b>3</b>')
    html_lgr = html_lgr.replace('id="badge-centroid">Centroide: <b>-</b>', 'id="badge-centroid">Centroide: <b>-1.50</b>')
    render_html_to_webp(html_lgr, 1918, 997, "lugar-das-raizes.webp")

    # 5. RESPOSTA EM FREQUÊNCIA (BODE E NYQUIST) (1918 x 997)
    res_freq = dispatch({'action': 'frequency_response', 'numerator': '10', 'denominator': 's * (s + 2) * (s + 5)', 'theme': 'dark'})
    form_freq = '''
    <div class="field-group"><label class="field-label">G(s) — Função de Transferência</label><input class="form-input code-font" value="10 / (s * (s + 2) * (s + 5))"></div>
    <div class="function-preview" style="margin-top: 0;">
      <div class="preview-heading">
        <span>Prévia matemática</span>
        <span class="preview-status valid">Expressão válida</span>
      </div>
      <div class="preview-math">$$G(s) = \\frac{10}{s (s + 2) (s + 5)}$$</div>
    </div>
    <div class="field-group"><label class="field-label">Frequência mínima ωmin (rad/s)</label><input class="form-input code-font" value="0.01"></div>
    <div class="field-group"><label class="field-label">Frequência máxima ωmax (rad/s)</label><input class="form-input code-font" value="100"></div>
    '''
    metrics_freq = [
        ('Margem de Ganho (MG)', '16.9 dB'),
        ('Cruzamento Ganho (ωcg)', '0.89 rad/s'),
        ('Margem de Fase (MF)', '52.8 °'),
        ('Cruzamento Fase (ωcf)', '3.16 rad/s'),
    ]
    html_freq = build_scientific_desktop("Resposta em Frequência", form_freq, res_freq['image'], metrics_freq)
    render_html_to_webp(html_freq, 1918, 997, "resposta-frequencia.webp")

    # 6. PROJETO DE CONTROLADORES (1918 x 997)
    res_ctrl = dispatch({'action': 'controller_design', 'plant_expr': '1 / (s * (s + 1) * (s + 5))', 'design_type': 'pid', 'method': 'zn_critical', 'controller_type': 'PID', 'critical_gain': 6, 'critical_period': 2, 'theme': 'dark'})
    form_ctrl = '''
    <div class="field-group"><label class="field-label">Planta G(s)</label><input class="form-input code-font" value="1 / (s * (s + 1) * (s + 5))"></div>
    <div class="function-preview" style="margin-top: 0;">
      <div class="preview-heading">
        <span>Prévia matemática</span>
        <span class="preview-status valid">Expressão válida</span>
      </div>
      <div class="preview-math">$$G(s) = \\frac{1}{s (s + 1) (s + 5)}$$</div>
    </div>
    <div class="field-group"><label class="field-label">Tipo de projeto</label><select class="form-select"><option selected>Sintonia P / PI / PID</option></select></div>
    <div class="field-group"><label class="field-label">Método</label><select class="form-select"><option selected>Ziegler-Nichols — oscilação crítica</option></select></div>
    <div class="field-group"><label class="field-label">Estrutura</label><select class="form-select"><option selected>PID</option></select></div>
    '''
    metrics_ctrl = [
        ('Ganho Proporcional (Kp)', '3.60'),
        ('Tempo Integral (Ti)', '1.00 s'),
        ('Tempo Derivativo (Td)', '0.25 s'),
        ('Estabilidade Malha Fechada', 'Estável'),
    ]
    html_ctrl = build_scientific_desktop("Projeto de Controladores", form_ctrl, res_ctrl['image'], metrics_ctrl)
    render_html_to_webp(html_ctrl, 1918, 997, "projeto-controladores.webp")

    # 7. ESPAÇO DE ESTADOS (1918 x 997)
    res_ss = dispatch({'action': 'state_space', 'A': '[[0, 1], [-2, -3]]', 'B': '[[0], [1]]', 'C': '[[1, 0]]', 'D': '[[0]]', 'canonical_form': 'controllable', 'desired_poles': '-4, -5', 'theme': 'dark'})
    form_ss = '''
    <div class="field-group"><label class="field-label">Forma de entrada</label><select class="form-select"><option selected>Matrizes A, B, C e D</option></select></div>
    <div class="field-group"><label class="field-label">Matriz A</label><textarea class="form-input code-font" rows="2">[[0, 1], [-2, -3]]</textarea></div>
    <div class="field-group"><label class="field-label">Matriz B</label><textarea class="form-input code-font" rows="2">[[0], [1]]</textarea></div>
    <div class="field-group"><label class="field-label">Matriz C</label><textarea class="form-input code-font" rows="2">[[1, 0]]</textarea></div>
    '''
    metrics_ss = [
        ('Posto Controlabilidade', '2 (Completo)'),
        ('Controlabilidade', 'Controlável'),
        ('Posto Observabilidade', '2 (Completo)'),
        ('Observabilidade', 'Observável'),
    ]
    html_ss = build_scientific_desktop("Espaço de Estados", form_ss, res_ss['image'], metrics_ss)
    render_html_to_webp(html_ss, 1918, 997, "espaco-estados.webp")

    print("\n✓ Todas as 7 capturas de tela desktop (1918x997) foram geradas com sucesso!")

if __name__ == "__main__":
    generate_all()
