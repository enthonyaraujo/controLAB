"""Módulo unificado de tema para gráficos Matplotlib do ControLAB.

Garante consistência visual com os tokens CSS (Dark/Light) do frontend,
registra fontes locais (IBM Plex Sans e JetBrains Mono) e remove
molduras (spines superior e direita), títulos redundantes e fundos opacos.
"""

from __future__ import annotations

import io
import base64
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterable

import matplotlib
import matplotlib.pyplot as plt
from matplotlib import font_manager

# Garante backend headless
matplotlib.use("Agg")

# Paletas unificadas no padrão científico acadêmico preto e branco (para ambos os temas)
THEME_TOKENS: dict[str, dict[str, Any]] = {
    "dark": {
        "bg_fig": "#FFFFFF",
        "bg_card": "#FFFFFF",
        "text": "#000000",
        "text_secondary": "#111827",
        "text_muted": "#475569",
        "grid": "#CBD5E1",
        "grid_alpha": 0.75,
        "spine": "#000000",
        "accent": "#000000",
        "accent_secondary": "#334155",
        "reference": "#64748B",
        "success": "#0F766E",
        "danger": "#B91C1C",
        "warning": "#B45309",
        "lgr_branch": "#000000",
        "lgr_pole": "#000000",
        "lgr_zero": "#000000",
        "lgr_break": "#334155",
        "font_sans": "IBM Plex Sans",
        "font_mono": "JetBrains Mono",
    },
    "light": {
        "bg_fig": "#FFFFFF",
        "bg_card": "#FFFFFF",
        "text": "#000000",
        "text_secondary": "#111827",
        "text_muted": "#475569",
        "grid": "#CBD5E1",
        "grid_alpha": 0.75,
        "spine": "#000000",
        "accent": "#000000",
        "accent_secondary": "#334155",
        "reference": "#64748B",
        "success": "#0F766E",
        "danger": "#B91C1C",
        "warning": "#B45309",
        "lgr_branch": "#000000",
        "lgr_pole": "#000000",
        "lgr_zero": "#000000",
        "lgr_break": "#334155",
        "font_sans": "IBM Plex Sans",
        "font_mono": "JetBrains Mono",
    },
}

_FONTS_REGISTERED = False


def register_local_fonts() -> bool:
    """Registra fontes locais (IBM Plex Sans e JetBrains Mono) no Matplotlib."""
    global _FONTS_REGISTERED
    if _FONTS_REGISTERED:
        return True

    search_roots = [
        Path(__file__).resolve().parent / "src" / "renderer" / "vendor" / "fonts",
        Path(__file__).resolve().parent / "vendor" / "fonts",
        Path(__file__).resolve().parent / "public" / "vendor" / "fonts",
    ]

    registered_any = False
    for root in search_roots:
        if root.is_dir():
            for ttf_path in root.rglob("*.ttf"):
                try:
                    font_manager.fontManager.addfont(str(ttf_path))
                    registered_any = True
                except Exception:
                    pass

    _FONTS_REGISTERED = True
    return registered_any


# Inicializa fontes locais ao carregar o módulo
register_local_fonts()


def get_theme_tokens(theme: str = "dark") -> dict[str, Any]:
    """Retorna os tokens de cores e estilos para o tema solicitado."""
    theme_key = "light" if theme and str(theme).lower() == "light" else "dark"
    return THEME_TOKENS[theme_key]


def apply_plot_theme(fig, axes: Any = None, theme: str = "dark") -> None:
    """Aplica o tema visual profissional aos eixos e à figura."""
    tokens = get_theme_tokens(theme)

    fig.patch.set_facecolor(tokens["bg_fig"])
    fig.patch.set_alpha(1.0)

    # Identifica todos os eixos se não fornecidos
    if axes is None:
        target_axes = fig.axes
    elif isinstance(axes, (list, tuple)):
        target_axes = list(axes)
    elif hasattr(axes, "flat"):
        target_axes = list(axes.flat)
    else:
        target_axes = [axes]

    for ax in target_axes:
        ax.set_facecolor(tokens["bg_fig"])
        ax.patch.set_alpha(1.0)

        # Caixa de eixos completa (todas as 4 bordas visíveis em 1.0px)
        for spine_name in ("top", "right", "bottom", "left"):
            if spine_name in ax.spines:
                ax.spines[spine_name].set_visible(True)
                ax.spines[spine_name].set_color(tokens["spine"])
                ax.spines[spine_name].set_linewidth(1.0)

        # Ticks e rótulos
        ax.tick_params(
            colors=tokens["text_secondary"],
            labelsize=11,
            labelcolor=tokens["text_secondary"],
            width=0.9,
            length=4.5,
        )

        ax.xaxis.label.set_color(tokens["text"])
        ax.yaxis.label.set_color(tokens["text"])
        ax.xaxis.label.set_fontsize(11.5)
        ax.yaxis.label.set_fontsize(11.5)
        ax.xaxis.label.set_fontweight("500")
        ax.yaxis.label.set_fontweight("500")

        # Grade sutil
        ax.grid(
            True,
            linestyle="--",
            linewidth=0.6,
            color=tokens["grid"],
            alpha=tokens["grid_alpha"],
        )

        # Legenda interna com estilo profissional
        legend = ax.get_legend()
        if legend is not None:
            legend_title = legend.get_title()
            if legend_title is not None:
                legend_title.set_color(tokens["text"])
                legend_title.set_fontsize(10.0)
                legend_title.set_fontweight("bold")

            frame = legend.get_frame()
            if frame is not None:
                frame.set_facecolor(tokens["bg_card"])
                frame.set_edgecolor(tokens["spine"])
                frame.set_linewidth(0.8)
                frame.set_alpha(0.95)

            for text in legend.get_texts():
                text.set_color(tokens["text"])
                text.set_fontsize(9.5)


@contextmanager
def theme_context(theme: str = "dark"):
    """Context manager para configurar rcParams durante a geração do gráfico."""
    register_local_fonts()
    tokens = get_theme_tokens(theme)

    rc_settings = {
        "figure.facecolor": tokens["bg_fig"],
        "axes.facecolor": tokens["bg_fig"],
        "axes.edgecolor": tokens["spine"],
        "axes.labelcolor": tokens["text"],
        "axes.linewidth": 1.0,
        "axes.grid": True,
        "grid.color": tokens["grid"],
        "grid.linestyle": "--",
        "grid.linewidth": 0.6,
        "grid.alpha": tokens["grid_alpha"],
        "xtick.color": tokens["text_secondary"],
        "ytick.color": tokens["text_secondary"],
        "xtick.labelsize": 11,
        "ytick.labelsize": 11,
        "font.family": [tokens["font_sans"], "DejaVu Sans", "sans-serif"],
        "font.size": 11,
        "lines.linewidth": 1.75,
        "text.color": tokens["text"],
        "legend.frameon": True,
        "legend.facecolor": tokens["bg_card"],
        "legend.edgecolor": tokens["spine"],
        "legend.framealpha": 0.95,
        "legend.fontsize": 9.5,
        "legend.title_fontsize": 10.0,
        "legend.labelcolor": tokens["text"],
        "svg.fonttype": "none",
    }

    with plt.rc_context(rc_settings):
        yield tokens

