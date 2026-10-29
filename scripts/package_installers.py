#!/usr/bin/env python3
"""Build installers from the native bundle produced by scripts/package_native.py.

Linux: deb (dpkg-deb), rpm (rpmbuild) and AppImage (appimagetool).
Windows: per-user NSIS installer (makensis). The tools must already be installed.
"""
import argparse
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

from set_version import current_version

ROOT = Path(__file__).resolve().parents[1]
NAME = 'controlab'
SUMMARY = 'Interactive suite for control systems engineering'
DESCRIPTION = ('ControLAB: suíte interativa e didática de Engenharia e Sistemas de Controle. '
    'Lugar das raízes, Routh-Hurwitz, respostas no tempo e em frequência, projeto de '
    'controladores, espaço de estados e scripts .m.')
MAINTAINER = 'Enthony Araujo <contato@enthony.com.br>'
HOMEPAGE = 'https://github.com/enthonyaraujo/controlab'
# Written into the bundle by validate_native_bundle.py --ui; never distributed.
EXCLUDED = {'validation-ui.png'}
TOOLS = {'deb': 'dpkg-deb', 'rpm': 'rpmbuild', 'appimage': 'appimagetool', 'nsis': 'makensis'}


def copy_bundle(bundle, destination):
    shutil.copytree(bundle, destination, symlinks=True,
        ignore=lambda folder, names: [n for n in names if Path(folder) == bundle and n in EXCLUDED])


def required_glibc(tree):
    """Highest GLIBC_2.x symbol version referenced by the ELF files in the tree."""
    minor = 17
    for path in tree.rglob('*'):
        if path.is_file() and not path.is_symlink():
            with path.open('rb') as handle:
                if handle.read(4) != b'\x7fELF':
                    continue
                handle.seek(0)
                for match in re.finditer(rb'GLIBC_2\.(\d+)', handle.read()):
                    minor = max(minor, int(match.group(1)))
    return f'2.{minor}'


def stage_linux(bundle, root):
    """FHS tree: application in /opt/controlab, launcher, menu entry and icons."""
    copy_bundle(bundle, root / 'opt' / NAME)
    (root / 'usr/bin').mkdir(parents=True)
    # The UI resolves its engine from the real executable path, so a symlink is enough.
    (root / 'usr/bin' / NAME).symlink_to(f'/opt/{NAME}/{NAME}')
    applications = root / 'usr/share/applications'
    applications.mkdir(parents=True)
    shutil.copy2(ROOT / 'packaging/linux/controlab.desktop', applications / f'{NAME}.desktop')
    for icon in sorted((ROOT / 'packaging/icons/linux').glob('*.png')):
        folder = root / 'usr/share/icons/hicolor' / icon.stem / 'apps'
        folder.mkdir(parents=True)
        shutil.copy2(icon, folder / f'{NAME}.png')


def build_deb(bundle, output, version, arch, temp):
    root = temp / 'deb'
    stage_linux(bundle, root)
    size = sum(p.stat().st_size for p in root.rglob('*') if p.is_file() and not p.is_symlink()) // 1024
    deb_arch = {'x86_64': 'amd64', 'aarch64': 'arm64'}[arch]
    (root / 'DEBIAN').mkdir()
    (root / 'DEBIAN/control').write_text(
        f'Package: {NAME}\nVersion: {version}\nArchitecture: {deb_arch}\nMaintainer: {MAINTAINER}\n'
        f'Installed-Size: {size}\nDepends: libc6 (>= {required_glibc(root)}), libgcc-s1, libgl1, libegl1, libxkbcommon0, libxkbcommon-x11-0\n'
        'Recommends: libwayland-client0, libwayland-egl1, xdg-desktop-portal\n'
        f'Section: science\nPriority: optional\nHomepage: {HOMEPAGE}\n'
        f'Description: {SUMMARY}\n {DESCRIPTION}\n', encoding='utf-8')
    package = output / f'ControLAB-v{version}-linux-{deb_arch}.deb'
    subprocess.run(['dpkg-deb', '--root-owner-group', '-Zxz', '--build', str(root), str(package)], check=True)
    return package


def build_rpm(bundle, output, version, arch, temp):
    root = temp / 'rpm-root'
    stage_linux(bundle, root)
    name = f'ControLAB-v{version}-linux-{arch}.rpm'
    spec = temp / f'{NAME}.spec'
    # PyInstaller binaries carry their archive after the ELF data: never strip them,
    # and do not derive requirements from the bundled private libraries.
    spec.write_text(f'''%global debug_package %{{nil}}
%global __os_install_post %{{nil}}
%global __arch_install_post %{{nil}}
%global _build_id_links none
Name: {NAME}
Version: {version}
Release: 1
Summary: {SUMMARY}
License: MIT
URL: {HOMEPAGE}
AutoReqProv: no
Requires: libc.so.6(GLIBC_{required_glibc(root)})(64bit), libgcc_s.so.1()(64bit)
Requires: libGL.so.1()(64bit), libEGL.so.1()(64bit), libxkbcommon.so.0()(64bit), libxkbcommon-x11.so.0()(64bit)
Recommends: xdg-desktop-portal

%description
{DESCRIPTION}

%install
cp -a {root}/. %{{buildroot}}/

%files
/opt/{NAME}
/usr/bin/{NAME}
/usr/share/applications/{NAME}.desktop
/usr/share/icons/hicolor/*/apps/{NAME}.png
''', encoding='utf-8')
    subprocess.run(['rpmbuild', '-bb', '--target', arch, '--define', f'_topdir {temp / "rpmbuild"}',
        '--define', f'_rpmdir {output}', '--define', f'_rpmfilename {name}', str(spec)], check=True)
    return output / name


def build_appimage(bundle, output, version, arch, temp):
    appdir = temp / 'ControLAB.AppDir'
    copy_bundle(bundle, appdir / 'opt' / NAME)
    shutil.copy2(ROOT / 'packaging/linux/controlab.desktop', appdir / f'{NAME}.desktop')
    shutil.copy2(ROOT / 'packaging/icons/linux/256x256.png', appdir / f'{NAME}.png')
    (appdir / '.DirIcon').symlink_to(f'{NAME}.png')
    apprun = appdir / 'AppRun'
    apprun.write_text('#!/bin/sh\nHERE="$(dirname "$(readlink -f "$0")")"\n'
        f'exec "$HERE/opt/{NAME}/{NAME}" "$@"\n', encoding='utf-8')
    apprun.chmod(0o755)
    package = output / f'ControLAB-v{version}-linux-{arch}.AppImage'
    tool = os.environ.get('APPIMAGETOOL', 'appimagetool')
    subprocess.run([tool, '--no-appstream', str(appdir), str(package)], check=True,
        env=dict(os.environ, ARCH=arch))
    return package


def build_nsis(bundle, output, version, arch, temp):
    source = temp / 'ControLAB'
    copy_bundle(bundle, source)
    package = output / f'ControLAB-v{version}-win-{arch}.exe'
    subprocess.run(['makensis', '/V3', '/INPUTCHARSET', 'UTF8', f'/DVERSION={version}',
        f'/DSOURCE_DIR={source}', f'/DICON={ROOT / "packaging/icons/icon.ico"}', f'/DLICENSE={ROOT / "LICENSE"}',
        f'/DOUTFILE={package}', str(ROOT / 'packaging/windows/controlab.nsi')], check=True)
    return package


BUILDERS = {'deb': build_deb, 'rpm': build_rpm, 'appimage': build_appimage, 'nsis': build_nsis}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('formats', nargs='*', metavar='FORMAT',
        help='deb, rpm, appimage or nsis (default: all formats of the current system)')
    parser.add_argument('--bundle', type=Path, default=ROOT / 'dist/native/controlab')
    parser.add_argument('--output', type=Path, default=ROOT / 'release')
    parser.add_argument('--version', default=None, help='Default: native/Cargo.toml')
    args = parser.parse_args()
    formats = args.formats or (['nsis'] if sys.platform == 'win32' else ['deb', 'rpm', 'appimage'])
    if set(formats) - set(BUILDERS):
        parser.error('Formatos aceitos: ' + ', '.join(BUILDERS))
    bundle = args.bundle.resolve()
    if not (bundle / 'bundle.json').is_file():
        parser.error(f'Pacote nativo ausente: {bundle}. Execute scripts/package_native.py primeiro.')
    native = {'nsis'} if sys.platform == 'win32' else {'deb', 'rpm', 'appimage'} if sys.platform == 'linux' else set()
    if set(formats) - native:
        parser.error('Pacotes Linux são gerados no Linux e o instalador NSIS no Windows.')
    tools = {f: os.environ.get('APPIMAGETOOL', TOOLS[f]) if f == 'appimage' else TOOLS[f] for f in formats}
    missing = [tool for tool in tools.values() if shutil.which(tool) is None]
    if missing:
        parser.error('Ferramentas ausentes: ' + ', '.join(missing))
    machine = platform.machine().lower()
    arch = {'amd64': 'x64', 'x86_64': 'x64', 'arm64': 'arm64'}[machine] if sys.platform == 'win32' else machine
    version = (args.version or current_version()).removeprefix('v')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    for name in formats:
        with tempfile.TemporaryDirectory(prefix=f'controlab-{name}-') as temp:
            print(BUILDERS[name](bundle, output, version, arch, Path(temp)))


if __name__ == '__main__':
    main()
