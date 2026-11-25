#!/usr/bin/env bash
# Script de compilación cruzada de Rust (1.100.0) hacia Android (32 y 64 bits)
# y exportación de APKs Debug separados por arquitectura en Godot 4.7.2.
#
# Lógica: Configura los enlazadores del Android NDK (API 24), descarga/compila
# `colonia_core` en modo debug o release para `aarch64-linux-android` (64 bits)
# y `armv7-linux-androideabi` (32 bits), copia los `.so` a `godot/bin/` y, si
# Godot CLI está presente, exporta dos APKs Debug independientes por ABI.

set -euo pipefail

BUILD_MODE="${BUILD_MODE:-debug}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
GODOT_DIR="$ROOT_DIR/godot"

NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -z "$NDK_HOME" && -n "${ANDROID_HOME:-}" && -d "${ANDROID_HOME}/ndk" ]]; then
  NDK_HOME="$(find "${ANDROID_HOME}/ndk" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)"
fi

if [[ -z "$NDK_HOME" || ! -d "$NDK_HOME" ]]; then
  echo "Error: Define ANDROID_NDK_HOME (o ANDROID_HOME con NDK instalado) antes de ejecutar este script." >&2
  exit 1
fi

HOST_TAG="linux-x86_64"
TOOLCHAIN_BIN="$NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"

export CC_aarch64_linux_android="$TOOLCHAIN_BIN/aarch64-linux-android24-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN_BIN/aarch64-linux-android24-clang"

export CC_armv7_linux_androideabi="$TOOLCHAIN_BIN/armv7a-linux-androideabi24-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$TOOLCHAIN_BIN/armv7a-linux-androideabi24-clang"

export AR_aarch64_linux_android="$TOOLCHAIN_BIN/llvm-ar"
export AR_armv7_linux_androideabi="$TOOLCHAIN_BIN/llvm-ar"
export CLANG_PATH="$TOOLCHAIN_BIN/clang"

CARGO_FLAGS=()
TARGET_SUBDIR="debug"
GODOT_EXPORT_FLAG="--export-debug"
if [[ "$BUILD_MODE" == "release" ]]; then
  CARGO_FLAGS+=(--release)
  TARGET_SUBDIR="release"
  GODOT_EXPORT_FLAG="--export-release"
fi

pushd "$RUST_DIR" >/dev/null

echo "[1/4] Descargando dependencias de Cargo..."
cargo fetch

echo "[2/4] Compilando colonia_core ($BUILD_MODE) para Android ARM64 (64 bits)..."
cargo build --target aarch64-linux-android "${CARGO_FLAGS[@]}"

echo "[3/4] Compilando colonia_core ($BUILD_MODE) para Android ARMv7 (32 bits)..."
cargo build --target armv7-linux-androideabi "${CARGO_FLAGS[@]}"

popd >/dev/null

echo "[4/5] Compilando librería anfitrión Linux para validación GDExtension en Godot..."
pushd "$RUST_DIR" >/dev/null
cargo build "${CARGO_FLAGS[@]}"
popd >/dev/null

echo "[5/5] Copiando binarios .so al proyecto Godot..."
mkdir -p "$GODOT_DIR/bin/aarch64" "$GODOT_DIR/bin/armv7" "$GODOT_DIR/bin/linux"
cp "$RUST_DIR/target/aarch64-linux-android/${TARGET_SUBDIR}/libcolonia_core.so" "$GODOT_DIR/bin/aarch64/"
cp "$RUST_DIR/target/armv7-linux-androideabi/${TARGET_SUBDIR}/libcolonia_core.so" "$GODOT_DIR/bin/armv7/"
cp "$RUST_DIR/target/${TARGET_SUBDIR}/libcolonia_core.so" "$GODOT_DIR/bin/linux/"

if command -v godot >/dev/null 2>&1; then
  mkdir -p "$ROOT_DIR/build" "$GODOT_DIR/.godot"
  echo "Importando recursos SVG, ETC2/ASTC y GDExtension en Godot..."
  godot --headless --path "$GODOT_DIR" --import || true

  EXT_LIST_FILE="$GODOT_DIR/.godot/extension_list.cfg"
  EXT_ENTRY="res://addons/colonia_core/colonia_core.gdextension"
  if [[ ! -f "$EXT_LIST_FILE" ]] || ! grep -qxF "$EXT_ENTRY" "$EXT_LIST_FILE"; then
    echo "Registrando $EXT_ENTRY en .godot/extension_list.cfg..."
    echo "$EXT_ENTRY" >> "$EXT_LIST_FILE"
    godot --headless --path "$GODOT_DIR" --import || true
  fi

  APK_ARM64="$ROOT_DIR/build/colonia_centinela-arm64-v8a-${BUILD_MODE}.apk"
  APK_ARMV7="$ROOT_DIR/build/colonia_centinela-armeabi-v7a-${BUILD_MODE}.apk"

  echo "Exportando APK para arm64-v8a (64 bits)..."
  godot --headless --path "$GODOT_DIR" "$GODOT_EXPORT_FLAG" "Android-arm64-v8a" "$APK_ARM64"

  echo "Exportando APK para armeabi-v7a (32 bits)..."
  godot --headless --path "$GODOT_DIR" "$GODOT_EXPORT_FLAG" "Android-armeabi-v7a" "$APK_ARMV7"

  ensure_so_in_apk() {
    local apk_file="$1"
    local abi="$2"
    local so_src="$3"
    if ! unzip -l "$apk_file" | grep -q "lib/${abi}/libcolonia_core.so"; then
      echo "Aviso: lib/${abi}/libcolonia_core.so no fue empaquetado por el template; inyectando binario nativo sin comprimir..."
      local stage_dir
      stage_dir="$(mktemp -d)"
      mkdir -p "$stage_dir/lib/${abi}"
      cp "$so_src" "$stage_dir/lib/${abi}/libcolonia_core.so"
      (cd "$stage_dir" && zip -0 -r "$apk_file" "lib/${abi}/libcolonia_core.so")
      rm -rf "$stage_dir"
    fi
    if ! unzip -l "$apk_file" | grep -q "lib/${abi}/libcolonia_core.so"; then
      echo "Error fatal: falta lib/${abi}/libcolonia_core.so en $apk_file" >&2
      exit 1
    fi
    echo "Verificado lib/${abi}/libcolonia_core.so dentro de $(basename "$apk_file")."
  }

  ensure_so_in_apk "$APK_ARM64" "arm64-v8a" "$GODOT_DIR/bin/aarch64/libcolonia_core.so"
  ensure_so_in_apk "$APK_ARMV7" "armeabi-v7a" "$GODOT_DIR/bin/armv7/libcolonia_core.so"
fi
