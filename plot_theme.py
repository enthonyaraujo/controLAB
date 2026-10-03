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

# Paletas espelhando os tokens CSS do ControLAB
THEME_TOKENS: dict[str, dict[str, Any]] = {
    "dark": {
        "bg_fig": "none",
        "bg_card": "#0F151C",
        "text": "#E6EDF3",
        "text_secondary": "#8693A3",
        "text_muted": "#6B7785",
        "grid": "#1F2933",
        "grid_alpha": 0.35,
        "spine": "#3A4654",
        "accent": "#3DD6F5",
        "accent_secondary": "#2C7A8E",
        "reference": "#3A4654",
        "success": "#10b981",
        "danger": "#ef4444",
        "warning": "#f59e0b",
        "font_sans": "IBM Plex Sans",
        "font_mono": "JetBrains Mono",
    },
    "light": {
        "bg_fig": "none",
        "bg_card": "#FFFFFF",
        "text": "#0E141B",
        "text_secondary": "#526071",
        "text_muted": "#788796",
        "grid": "#CBD3DC",
        "grid_alpha": 0.45,
        "spine": "#9DA8B5",
        "accent": "#0891B2",
        "accent_secondary": "#0E7490",
        "reference": "#9DA8B5",
        "success": "#059669",
        "danger": "#dc2626",
        "warning": "#d97706",
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

    fig.patch.set_facecolor("none")
    fig.patch.set_alpha(0.0)

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
        ax.set_facecolor("none")
        ax.patch.set_alpha(0.0)

        # Spines: remove superior e direita, suaviza inferior e esquerda
        if "top" in ax.spines:
            ax.spines["top"].set_visible(False)
        if "right" in ax.spines:
            ax.spines["right"].set_visible(False)
        if "left" in ax.spines:
            ax.spines["left"].set_color(tokens["spine"])
            ax.spines["left"].set_linewidth(1.0)
        if "bottom" in ax.spines:
            ax.spines["bottom"].set_color(tokens["spine"])
            ax.spines["bottom"].set_linewidth(1.0)

        # Ticks e rótulos
        ax.tick_params(
            colors=tokens["text_secondary"],
            labelsize=9,
            labelcolor=tokens["text_secondary"],
            width=0.8,
            length=4,
        )

        ax.xaxis.label.set_color(tokens["text"])
        ax.yaxis.label.set_color(tokens["text"])
        ax.xaxis.label.set_fontsize(9.5)
        ax.yaxis.label.set_fontsize(9.5)

        # Grade sutil
        ax.grid(
            True,
            linestyle="--",
            linewidth=0.6,
            color=tokens["grid"],
            alpha=tokens["grid_alpha"],
        )

        # Legenda minimalista e sem borda
        legend = ax.get_legend()
        if legend is not None:
            legend.get_frame().set_linewidth(0.0)
            legend.get_frame().set_alpha(0.0)
            for text in legend.get_texts():
                text.set_color(tokens["text"])
                text.set_fontsize(8.5)


@contextmanager
def theme_context(theme: str = "dark"):
    """Context manager para configurar rcParams durante a geração do gráfico."""
    register_local_fonts()
    tokens = get_theme_tokens(theme)

    rc_settings = {
        "figure.facecolor": "none",
        "axes.facecolor": "none",
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
        "xtick.labelsize": 9,
        "ytick.labelsize": 9,
        "font.family": [tokens["font_sans"], "DejaVu Sans", "sans-serif"],
        "font.size": 9.5,
        "lines.linewidth": 2.0,
        "legend.frameon": False,
        "legend.fontsize": 8.5,
        "legend.labelcolor": tokens["text"],
        "svg.fonttype": "none",  # Exporta texto como elemento <text> nativo
    }

    with plt.rc_context(rc_settings):
        yield tokens
