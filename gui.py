"""
Interface Gráfica Desktop (Tkinter) para o Lugar Geométrico das Raízes (LGR)
Com visualizador Matplotlib incorporado e barra de ferramentas interativa.
"""

import tkinter as tk
from tkinter import ttk, messagebox
import matplotlib
matplotlib.use("TkAgg")
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg, NavigationToolbar2Tk
import sympy as sp
import numpy as np

from lgr_engine import (
    lgr_completo,
    parse_tf_expression,
    parse_coeffs_str,
    parse_zpk,
    format_complex,
    PRESETS,
)


class LGRExplorerApp(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Lugar Geométrico das Raízes (LGR) - Explorer")
        self.geometry("1200x800")
        self.minsize(950, 650)
        
        # Configuração de Estilo ttk
        style = ttk.Style(self)
        try:
            style.theme_use("clam")
        except Exception:
            pass

        self._create_widgets()
        self.load_preset_example(list(PRESETS.keys())[0])

    def _create_widgets(self):
        # Frame Principal com Divisão Horizontal
        main_paned = ttk.PanedWindow(self, orient=tk.HORIZONTAL)
        main_paned.pack(fill=tk.BOTH, expand=True, padx=8, pady=8)

        # Painel Lateral Esquerdo (Entradas e Controles)
        left_frame = ttk.Frame(main_paned, width=380)
        main_paned.add(left_frame, weight=1)

        # Painel Direito (Gráfico Matplotlib e Detalhes)
        right_frame = ttk.Frame(main_paned)
        main_paned.add(right_frame, weight=3)

        # -------------------------------------------------------------
        # Configuração do Painel Esquerdo
        # -------------------------------------------------------------
        lbl_title = ttk.Label(
            left_frame, 
            text="📈 Configuração da FT", 
            font=("Helvetica", 14, "bold")
        )
        lbl_title.pack(anchor="w", pady=(0, 10))

        # Presets Rápidos
        preset_frame = ttk.LabelFrame(left_frame, text="Exemplos Prontos")
        preset_frame.pack(fill=tk.X, pady=(0, 10))
        
        self.combo_presets = ttk.Combobox(
            preset_frame, 
            values=list(PRESETS.keys()), 
            state="readonly"
        )
        self.combo_presets.pack(fill=tk.X, padx=8, pady=8)
        self.combo_presets.bind("<<ComboboxSelected>>", self.on_preset_selected)

        # Notebook com Modos de Entrada
        self.notebook = ttk.Notebook(left_frame)
        self.notebook.pack(fill=tk.X, pady=(0, 10))

        # Aba 1: Expressão Simbólica
        tab_expr = ttk.Frame(self.notebook, padding=8)
        self.notebook.add(tab_expr, text="Expressão")
        
        ttk.Label(tab_expr, text="Expressão G(s):").pack(anchor="w")
        self.entry_expr = ttk.Entry(tab_expr, font=("Courier", 10))
        self.entry_expr.insert(0, "(s + 2) / (s * (s + 1) * (s + 4))")
        self.entry_expr.pack(fill=tk.X, pady=4)
        ttk.Label(
            tab_expr, 
            text="Ex: (s + 2) / (s*(s+1)*(s+4))\nou 1 / (s^3 + 2s^2 + 2s)", 
            font=("Helvetica", 8), 
            foreground="gray"
        ).pack(anchor="w")

        # Aba 2: Coeficientes
        tab_coeffs = ttk.Frame(self.notebook, padding=8)
        self.notebook.add(tab_coeffs, text="Coeficientes")
        
        ttk.Label(tab_coeffs, text="Numerador N(s):").pack(anchor="w")
        self.entry_num = ttk.Entry(tab_coeffs, font=("Courier", 10))
        self.entry_num.insert(0, "1, 2")
        self.entry_num.pack(fill=tk.X, pady=2)

        ttk.Label(tab_coeffs, text="Denominador D(s):").pack(anchor="w")
        self.entry_den = ttk.Entry(tab_coeffs, font=("Courier", 10))
        self.entry_den.insert(0, "1, 5, 4, 0")
        self.entry_den.pack(fill=tk.X, pady=2)

        # Aba 3: Polos e Zeros
        tab_zpk = ttk.Frame(self.notebook, padding=8)
        self.notebook.add(tab_zpk, text="Polos/Zeros")
        
        ttk.Label(tab_zpk, text="Ganho K:").pack(anchor="w")
        self.entry_k = ttk.Entry(tab_zpk, font=("Courier", 10))
        self.entry_k.insert(0, "1.0")
        self.entry_k.pack(fill=tk.X, pady=2)

        ttk.Label(tab_zpk, text="Zeros (separados por vírgula):").pack(anchor="w")
        self.entry_zeros = ttk.Entry(tab_zpk, font=("Courier", 10))
        self.entry_zeros.insert(0, "-2")
        self.entry_zeros.pack(fill=tk.X, pady=2)

        ttk.Label(tab_zpk, text="Polos (separados por vírgula):").pack(anchor="w")
        self.entry_poles = ttk.Entry(tab_zpk, font=("Courier", 10))
        self.entry_poles.insert(0, "0, -1, -4")
        self.entry_poles.pack(fill=tk.X, pady=2)

        # Título Personalizado
        title_frame = ttk.LabelFrame(left_frame, text="Título do Gráfico")
        title_frame.pack(fill=tk.X, pady=(0, 10))
        self.entry_title = ttk.Entry(title_frame)
        self.entry_title.insert(0, "Lugar Geométrico das Raízes")
        self.entry_title.pack(fill=tk.X, padx=8, pady=8)

        # Botão Principal de Cálculo
        btn_calc = ttk.Button(
            left_frame, 
            text="🚀 Traçar LGR (7 Passos)", 
            command=self.calculate_and_plot
        )
        btn_calc.pack(fill=tk.X, pady=(0, 15), ipady=5)

        # Caixa de Resumo da Função
        ft_info_frame = ttk.LabelFrame(left_frame, text="Função Interpretada")
        ft_info_frame.pack(fill=tk.BOTH, expand=True)

        self.txt_ft_info = tk.Text(ft_info_frame, wrap=tk.WORD, font=("Courier", 9), height=10)
        self.txt_ft_info.pack(fill=tk.BOTH, expand=True, padx=6, pady=6)

        # -------------------------------------------------------------
        # Configuração do Painel Direito (Abas de Gráfico e Passos)
        # -------------------------------------------------------------
        self.right_notebook = ttk.Notebook(right_frame)
        self.right_notebook.pack(fill=tk.BOTH, expand=True)

        # Aba de Gráfico
        self.tab_plot = ttk.Frame(self.right_notebook)
        self.right_notebook.add(self.tab_plot, text="📊 Gráfico do LGR")

        # Aba de Memorial dos 7 Passos
        self.tab_steps = ttk.Frame(self.right_notebook)
        self.right_notebook.add(self.tab_steps, text="📑 Memorial dos 7 Passos")

        self.txt_steps = tk.Text(self.tab_steps, wrap=tk.WORD, font=("Helvetica", 10))
        scroll_steps = ttk.Scrollbar(self.tab_steps, command=self.txt_steps.yview)
        self.txt_steps.configure(yscrollcommand=scroll_steps.set)
        self.txt_steps.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scroll_steps.pack(side=tk.RIGHT, fill=tk.Y)

        # Área do Canvas Matplotlib
        self.canvas = None
        self.toolbar = None

    def on_preset_selected(self, event=None):
        name = self.combo_presets.get()
        if name in PRESETS:
            self.load_preset_example(name)

    def load_preset_example(self, name):
        preset = PRESETS[name]
        self.combo_presets.set(name)
        self.entry_expr.delete(0, tk.END)
        self.entry_expr.insert(0, preset["expr"])
        
        self.entry_num.delete(0, tk.END)
        self.entry_num.insert(0, ", ".join(map(str, preset["num"])))
        
        self.entry_den.delete(0, tk.END)
        self.entry_den.insert(0, ", ".join(map(str, preset["den"])))

        self.calculate_and_plot()

    def get_current_tf_coeffs(self):
        tab_idx = self.notebook.index(self.notebook.select())
        if tab_idx == 0:  # Expressão
            expr_str = self.entry_expr.get().strip()
            num, den, _, _ = parse_tf_expression(expr_str)
            return num, den
        elif tab_idx == 1:  # Coeficientes
            num = parse_coeffs_str(self.entry_num.get())
            den = parse_coeffs_str(self.entry_den.get())
            return num, den
        elif tab_idx == 2:  # ZPK
            k = float(self.entry_k.get().strip() or "1.0")
            z = self.entry_zeros.get().strip()
            p = self.entry_poles.get().strip()
            num, den = parse_zpk(z, p, k)
            return num, den
        return [1.0, 2.0], [1.0, 5.0, 4.0, 0.0]

    def calculate_and_plot(self):
        try:
            num, den = self.get_current_tf_coeffs()
            custom_title = self.entry_title.get().strip() or "Lugar Geométrico das Raízes"
            
            # Formata Polinômios no painel de info
            s = sp.Symbol("s")
            pnum = sp.Poly.from_list(num, gens=s)
            pden = sp.Poly.from_list(den, gens=s)
            
            self.txt_ft_info.delete("1.0", tk.END)
            self.txt_ft_info.insert(tk.END, f"Numerador N(s):\n  {pnum.as_expr()}\n\n")
            self.txt_ft_info.insert(tk.END, f"Denominador D(s):\n  {pden.as_expr()}\n\n")
            self.txt_ft_info.insert(tk.END, f"N(s) Fatorado: {sp.factor(pnum.as_expr())}\n")
            self.txt_ft_info.insert(tk.END, f"D(s) Fatorado: {sp.factor(pden.as_expr())}\n")

            # Executa o LGR
            fig, ax, det = lgr_completo(num, den, titulo=custom_title, show_plot=False)

            # Atualiza o canvas Matplotlib
            if self.canvas is not None:
                self.canvas.get_tk_widget().destroy()
            if self.toolbar is not None:
                self.toolbar.destroy()

            self.canvas = FigureCanvasTkAgg(fig, master=self.tab_plot)
            self.canvas.draw()
            self.toolbar = NavigationToolbar2Tk(self.canvas, self.tab_plot)
            self.toolbar.update()
            self.canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True)

            # Atualiza o Memorial de Cálculo dos 7 Passos
            self._update_steps_text(det)

        except Exception as e:
            messagebox.showerror("Erro no Cálculo", f"Ocorreu um erro:\n{e}")

    def _update_steps_text(self, det):
        self.txt_steps.delete("1.0", tk.END)
        lines = []
        lines.append("=" * 65)
        lines.append("  MEMORIAL DE CÁLCULO - 7 PASSOS CLÁSSICOS DO LGR")
        lines.append("=" * 65 + "\n")

        # Passo 1, 2 e 3
        lines.append("PASSO 1, 2 e 3: Ramos, Polos, Zeros e Simetria")
        lines.append("-" * 55)
        lines.append(f"• Número de Polos (P): {det['P']}")
        lines.append(f"  Polos em: {', '.join([format_complex(p) for p in det['polos']])}")
        lines.append(f"• Número de Zeros (Z): {det['Z']}")
        if det['Z'] > 0:
            lines.append(f"  Zeros em: {', '.join([format_complex(z) for z in det['zeros']])}")
        else:
            lines.append("  Nenhum zero finito nesta FT.")
        lines.append(f"• Total de Ramos (n = max(P, Z)): {det['ramos']}")
        lines.append("• Simetria: O LGR é perfeitamente simétrico em relação ao Eixo Real.\n")

        # Passo 4
        lines.append("PASSO 4: Assíntotas e Centroide")
        lines.append("-" * 55)
        P, Z = det['P'], det['Z']
        if P > Z:
            lines.append(f"• Ramos para o infinito (P - Z): {P - Z}")
            lines.append(f"• Centroide (sigma_a): {det['centroide']:.3f}")
            lines.append("• Ângulos das Assíntotas (theta_k = (2k+1)*180 / (P-Z)):")
            for a in det['angulos_assintotas']:
                lines.append(f"    k = {a['k']}: {a['graus']:.1f}°")
        else:
            lines.append("• Como P <= Z, não há assíntotas direcionadas ao infinito.")
        lines.append("")

        # Passo 5
        lines.append("PASSO 5: Pontos de Partida e Retorno (Break-in / Breakaway)")
        lines.append("-" * 55)
        lines.append("• Condição: dK/ds = 0  =>  N'(s)*D(s) - N(s)*D'(s) = 0")
        if det['break_points']:
            for bp in det['break_points']:
                lines.append(f"  -> Ponto no eixo real: s = {bp['s']:.3f} com Ganho K = {bp['K']:.2f}")
        else:
            lines.append("  Nenhum ponto com K > 0 no eixo real.")
        lines.append("")

        # Passo 6
        lines.append("PASSO 6: Cruzamento do Eixo Imaginário (jw)")
        lines.append("-" * 55)
        lines.append("• Determina o limite de estabilidade em malha fechada:")
        if det['jw_cruzamentos']:
            for jw in det['jw_cruzamentos']:
                lines.append(f"  -> Cruzamento em: s = ±j{abs(jw['w']):.2f} com Ganho Crítico K_crit = {jw['K']:.2f}")
        else:
            lines.append("  Nenhum cruzamento encontrado na faixa analisada.")
        lines.append("")

        # Passo 7
        lines.append("PASSO 7: Ângulos de Partida (theta_d) e Chegada (theta_a)")
        lines.append("-" * 55)
        tem_partida = bool(det.get('angulos_partida'))
        tem_chegada = bool(det.get('angulos_chegada'))
        if tem_partida or tem_chegada:
            if tem_partida:
                for ap in det['angulos_partida']:
                    lines.append(f"  -> Polo {format_complex(ap['polo'])}: Ângulo de partida theta_d = {ap['angulo']:.1f}°")
            if tem_chegada:
                for ac in det['angulos_chegada']:
                    lines.append(f"  -> Zero {format_complex(ac['zero'])}: Ângulo de chegada theta_a = {ac['angulo']:.1f}°")
        else:
            lines.append("  Apenas singularidades reais presentes (sem polos/zeros complexos).")
        lines.append("")

        self.txt_steps.insert(tk.END, "\n".join(lines))


def run_gui():
    app = LGRExplorerApp()
    app.mainloop()


if __name__ == "__main__":
    run_gui()
