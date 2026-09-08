"""
Script Principal / Launcher do Lugar Geométrico das Raízes (LGR)
Permite iniciar tanto a Interface Web (Streamlit) quanto a Interface Desktop (Tkinter).
"""

import sys
import os
import subprocess
import argparse


def launch_web():
    """Inicia a interface web interativa do Streamlit."""
    print("Iniciando a Interface Web (Streamlit)...")
    app_path = os.path.join(os.path.dirname(__file__), "app.py")
    subprocess.run([sys.executable, "-m", "streamlit", "run", app_path])


def launch_gui():
    """Inicia a interface gráfica desktop (Tkinter)."""
    print("Iniciando a Interface Desktop (Tkinter)...")
    from gui import run_gui
    run_gui()


def main():
    parser = argparse.ArgumentParser(
        description="Lugar Geométrico das Raízes (LGR) - Interface Intuitiva"
    )
    parser.add_argument(
        "--web",
        action="store_true",
        help="Inicia a interface web via Streamlit",
    )
    parser.add_argument(
        "--gui",
        action="store_true",
        help="Inicia a interface desktop nativa (Tkinter)",
    )

    args = parser.parse_args()

    if args.web:
        launch_web()
    elif args.gui:
        launch_gui()
    else:
        # Modo padrão: Se executado diretamente sem flags, inicia a Web UI
        print("=" * 60)
        print("LGR Explorer - Lugar Geométrico das Raízes")
        print("=" * 60)
        print("1. Iniciar Interface Web Moderna (Streamlit)")
        print("2. Iniciar Interface Desktop Nativa (Tkinter)")
        print("Pressione [1] para Web ou [2] para Desktop (Padrão: 1)")
        
        try:
            choice = input("Escolha (1/2): ").strip()
            if choice == "2":
                launch_gui()
            else:
                launch_web()
        except (KeyboardInterrupt, EOFError):
            launch_web()


if __name__ == "__main__":
    main()
