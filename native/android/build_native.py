#!/usr/bin/env python3
"""Build Android JNI artifacts using installed Cargo/NDK tools; never install targets."""

import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
TARGETS = {"arm64-v8a": "aarch64-linux-android", "x86_64": "x86_64-linux-android"}


def find_ndk():
    for name in ("ANDROID_NDK_HOME", "ANDROID_NDK_ROOT", "ANDROID_NDK_LATEST_HOME", "ANDROID_NDK"):
        if os.environ.get(name):
            return Path(os.environ[name])
    sdk_paths = [os.environ.get("ANDROID_HOME"), os.environ.get("ANDROID_SDK_ROOT")]
    properties = Path(__file__).parent / "local.properties"
    if properties.exists():
        for line in properties.read_text().splitlines():
            if line.startswith("sdk.dir="):
                sdk_paths.append(line.split("=", 1)[1].replace("\\:", ":").replace("\\\\", "\\"))
    sdk_paths.append(str(Path.home() / "Android" / "Sdk"))
    for sdk in filter(None, sdk_paths):
        folder = Path(sdk) / "ndk"
        if folder.is_dir():
            versions = sorted(folder.iterdir(), key=lambda path: tuple(int(p) for p in path.name.split(".") if p.isdigit()))
            if versions:
                return versions[-1]
    raise RuntimeError("NDK não encontrado. Configure ANDROID_NDK_HOME ou ANDROID_HOME.")


def build(args):
    abis = args.abis.split(",")
    if any(abi not in TARGETS for abi in abis):
        raise RuntimeError("ABIs aceitas: " + ", ".join(TARGETS))
    installed = subprocess.check_output(["rustup", "target", "list", "--installed"], text=True).splitlines()
    missing = [TARGETS[abi] for abi in abis if TARGETS[abi] not in installed]
    if missing:
        raise RuntimeError("Alvos Rust ausentes (instale explicitamente antes de compilar): " + ", ".join(missing))
    ndk = find_ndk()
    host = {"Linux": "linux-x86_64", "Darwin": "darwin-x86_64", "Windows": "windows-x86_64"}.get(platform.system())
    if host is None:
        raise RuntimeError("Host NDK não suportado: " + platform.system())
    toolchain = ndk / "toolchains" / "llvm" / "prebuilt" / host / "bin"
    target_dir = ROOT / "build" / "native-android"
    output = Path(args.output).resolve()
    for abi in abis:
        triple = TARGETS[abi]
        linker = toolchain / (triple + "24-clang" + (".cmd" if os.name == "nt" else ""))
        if not linker.is_file():
            raise RuntimeError("Linker NDK ausente: " + str(linker))
        env = dict(os.environ)
        env["CARGO_TARGET_" + triple.upper().replace("-", "_") + "_LINKER"] = str(linker)
        env["CC_" + triple.replace("-", "_")] = str(linker)
        env["AR_" + triple.replace("-", "_")] = str(toolchain / ("llvm-ar.exe" if os.name == "nt" else "llvm-ar"))
        # NativeActivity glue is compiled by cc-rs, which also needs the NDK toolchain.
        env["PATH"] = str(toolchain) + os.pathsep + env.get("PATH", "")
        common = ["--locked", "--target", triple, "--target-dir", str(target_dir)]
        if args.profile == "release":
            common.append("--release")
        if args.offline:
            common.append("--offline")
        subprocess.run(["cargo", "build", *common, "-p", "controlab-native", "--lib"], cwd=ROOT, env=env, check=True)
        built = target_dir / triple / args.profile
        destination = output / abi
        destination.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built / "libcontrolab_native.so", destination / "libcontrolab_native.so")
        # Builds with older NDKs may require libc++_shared; include it when present.
        cxx = toolchain.parent / "sysroot" / "usr" / "lib" / triple / "libc++_shared.so"
        if cxx.exists():
            shutil.copy2(cxx, destination / cxx.name)
        print(f"[ControLAB] {abi}: interface Rust e interpretador .m em {destination}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--abis", default="arm64-v8a")
    parser.add_argument("--profile", choices=("debug", "release"), default="debug")
    parser.add_argument("--output", required=True)
    parser.add_argument("--offline", action="store_true")
    try:
        build(parser.parse_args())
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print(f"[ControLAB] {error}", file=sys.stderr)
        sys.exit(1)
