#!/usr/bin/env bash
# Gera o APK do BlockLab (arm64, assinado com a chave de depuração).
#
#   crates/blocklab-gui/android/build-apk.sh [destino.apk]
#
# Requisitos: Android SDK (platforms;android-34, build-tools;35.0.0) e NDK
# (testado com o r27), `rustup target add aarch64-linux-android` e
# `cargo install cargo-apk`.
set -euo pipefail

export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
if [[ -z "${ANDROID_NDK_ROOT:-}" ]]; then
    ANDROID_NDK_ROOT="$(ls -d "$ANDROID_HOME"/ndk/* | sort -V | tail -1)"
    export ANDROID_NDK_ROOT
fi

# Chave de depuração padrão do Android (criada se não existir).
keystore="$HOME/.android/debug.keystore"
if [[ ! -f "$keystore" ]]; then
    mkdir -p "$(dirname "$keystore")"
    keytool -genkeypair -keystore "$keystore" -storepass android -keypass android \
        -alias androiddebugkey -dname "CN=Android Debug,O=Android,C=US" \
        -keyalg RSA -keysize 2048 -validity 10000
fi
export CARGO_APK_RELEASE_KEYSTORE="$keystore"
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android

# Segmentos alinhados em 16 KB: aparelhos com Android 15+ e páginas de
# 16 KB recusam bibliotecas alinhadas em 4 KB.
export RUSTFLAGS="-Clink-arg=-Wl,-z,max-page-size=16384"

cd "$(dirname "$0")/.."
cargo apk build --release --lib

apk="../../target/release/apk/blocklab.apk"
if [[ $# -ge 1 ]]; then
    cp "$apk" "$1"
    apk="$1"
fi
echo "APK: $(realpath "$apk")"
