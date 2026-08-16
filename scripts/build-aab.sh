#!/usr/bin/env bash
set -euo pipefail

# Ensure ANDROID_HOME is set
if [ -z "${ANDROID_HOME:-}" ]; then
  export ANDROID_HOME="/opt/homebrew/share/android-commandlinetools"
fi

# Locate build-tools and android.jar
BUILD_TOOLS_DIR="${ANDROID_HOME}/build-tools/35.0.0"
if [ ! -d "$BUILD_TOOLS_DIR" ]; then
  BUILD_TOOLS_DIR=$(ls -d "${ANDROID_HOME}/build-tools/"* 2>/dev/null | tail -n 1)
fi

AAPT2="${BUILD_TOOLS_DIR}/aapt2"
ANDROID_JAR="${ANDROID_HOME}/platforms/android-35/android.jar"
if [ ! -f "$ANDROID_JAR" ]; then
  ANDROID_JAR=$(ls "${ANDROID_HOME}/platforms/android-"*/android.jar 2>/dev/null | tail -n 1)
fi

# Locate R8 compiler
R8_CMD=""
if command -v r8 &>/dev/null; then
  R8_CMD="r8"
elif [ -x "${ANDROID_HOME}/cmdline-tools/latest/bin/r8" ]; then
  R8_CMD="${ANDROID_HOME}/cmdline-tools/latest/bin/r8"
elif [ -f "${BUILD_TOOLS_DIR}/lib/d8.jar" ]; then
  R8_CMD="java -cp ${BUILD_TOOLS_DIR}/lib/d8.jar com.android.tools.r8.R8"
fi

if [ -z "$R8_CMD" ]; then
  echo "Warning: R8 compiler not found. Falling back to d8 classes.dex." >&2
fi

# Locate NDK LLVM tools (llvm-objcopy and llvm-strip)
NDK_DIR="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-${NDK_HOME:-}}}"
if [ -z "$NDK_DIR" ] || [ ! -d "$NDK_DIR" ]; then
  NDK_DIR=$(ls -d "${ANDROID_HOME}/ndk/"* 2>/dev/null | tail -n 1)
fi

LLVM_BIN=""
if [ -n "$NDK_DIR" ] && [ -d "$NDK_DIR" ]; then
  LLVM_BIN=$(ls -d "${NDK_DIR}/toolchains/llvm/prebuilt/"*/bin 2>/dev/null | head -n 1)
fi

OBJCOPY=""
STRIP=""
if [ -n "$LLVM_BIN" ] && [ -d "$LLVM_BIN" ]; then
  [ -x "${LLVM_BIN}/llvm-objcopy" ] && OBJCOPY="${LLVM_BIN}/llvm-objcopy"
  [ -x "${LLVM_BIN}/llvm-strip" ] && STRIP="${LLVM_BIN}/llvm-strip"
fi
if [ -z "$OBJCOPY" ] && command -v llvm-objcopy &>/dev/null; then
  OBJCOPY="llvm-objcopy"
fi
if [ -z "$STRIP" ] && command -v llvm-strip &>/dev/null; then
  STRIP="llvm-strip"
fi

if ! command -v bundletool &>/dev/null; then
  echo "Error: bundletool is required but not found in PATH." >&2
  echo "Install it via: brew install bundletool" >&2
  exit 1
fi

PROJECT_ROOT="$(pwd)"
BIN_DIR="${PROJECT_ROOT}/target/android-artifacts/release/bin/foxandhounds"
APK_OUT_DIR="${PROJECT_ROOT}/target/android-artifacts/release/apk"
TMP_DIR="${PROJECT_ROOT}/target/android-artifacts/release/aab_tmp"
PROGUARD_RULES="${PROJECT_ROOT}/res/proguard-rules.pro"

if [ ! -d "$BIN_DIR" ]; then
  echo "Error: Android build output directory '$BIN_DIR' does not exist." >&2
  echo "Please run 'cargo quad-apk build --release' first." >&2
  exit 1
fi

echo "==> Packaging Android App Bundle (.aab)..."
rm -rf "$TMP_DIR"
mkdir -p "$TMP_DIR/bundle_root/manifest" "$TMP_DIR/bundle_root/dex" "$TMP_DIR/r8_out" "$TMP_DIR/symbols" "$APK_OUT_DIR"

METADATA_ARGS=()

# 1. Compile resources into proto format for AAB
"$AAPT2" compile --dir res -o "$TMP_DIR/compiled_res.zip"
"$AAPT2" link --proto-format -o "$TMP_DIR/base_linked.apk" \
  -I "$ANDROID_JAR" \
  --manifest "$BIN_DIR/AndroidManifest.xml" \
  "$TMP_DIR/compiled_res.zip" -A assets

# 2. Run R8 optimizer on Java class files (shrinking, optimization & obfuscation)
if [ -n "$R8_CMD" ] && [ -f "$PROGUARD_RULES" ] && [ -d "$BIN_DIR/build/obj" ]; then
  echo "==> Running R8 code optimizer & shrinker..."
  CLASS_FILES=$(find "$BIN_DIR/build/obj" -name "*.class")
  
  $R8_CMD --release \
    --min-api 23 \
    --lib "$ANDROID_JAR" \
    --pg-conf "$PROGUARD_RULES" \
    --pg-map-output "$APK_OUT_DIR/mapping.txt" \
    --output "$TMP_DIR/r8_out" \
    $CLASS_FILES

  cp "$TMP_DIR/r8_out/classes.dex" "$TMP_DIR/bundle_root/dex/classes.dex"
  METADATA_ARGS+=(--metadata-file="com.android.tools.build.obfuscation/proguard.map:$APK_OUT_DIR/mapping.txt")
  echo "==> R8 optimization complete (mapping saved to $APK_OUT_DIR/mapping.txt)"
else
  echo "==> Using unoptimized classes.dex from build"
  cp "$BIN_DIR/classes.dex" "$TMP_DIR/bundle_root/dex/"
fi

# 3. Unpack proto-linked APK and structure bundle module directory
unzip -q "$TMP_DIR/base_linked.apk" -d "$TMP_DIR/bundle_root/"
mv "$TMP_DIR/bundle_root/AndroidManifest.xml" "$TMP_DIR/bundle_root/manifest/"

# 4. Process native libraries and extract DWARF debug symbols
if [ -d "$BIN_DIR/lib" ]; then
  cp -R "$BIN_DIR/lib" "$TMP_DIR/bundle_root/"

  if [ -n "$OBJCOPY" ] && [ -n "$STRIP" ]; then
    echo "==> Extracting native debug symbols and stripping release binaries..."
    for abi_dir in "$BIN_DIR/lib/"*; do
      if [ -d "$abi_dir" ]; then
        abi=$(basename "$abi_dir")
        mkdir -p "$TMP_DIR/symbols/$abi"
        for so_file in "$abi_dir/"*.so; do
          if [ -f "$so_file" ]; then
            soname=$(basename "$so_file")
            # Extract unstripped debug symbols
            "$OBJCOPY" --only-keep-debug "$so_file" "$TMP_DIR/symbols/$abi/${soname}.dbg"
            # Strip the library embedded in the APK/AAB
            "$STRIP" --strip-unneeded "$TMP_DIR/bundle_root/lib/$abi/$soname"
            # Add metadata file argument for bundletool
            METADATA_ARGS+=(--metadata-file="com.android.tools.build.debugsymbols/$abi/${soname}.dbg:$TMP_DIR/symbols/$abi/${soname}.dbg")
          fi
        done
      fi
    done

    # Archive native debug symbols into zip
    rm -f "$APK_OUT_DIR/native-debug-symbols.zip"
    (cd "$TMP_DIR/symbols" && zip -q -r "$APK_OUT_DIR/native-debug-symbols.zip" .)
    echo "==> Native debug symbols archived to $APK_OUT_DIR/native-debug-symbols.zip"
  else
    echo "Warning: llvm-objcopy / llvm-strip not found; skipping symbol extraction." >&2
  fi
fi

# 5. Zip module structure and build final AAB using bundletool
(cd "$TMP_DIR/bundle_root" && zip -q -r "$TMP_DIR/base.zip" .)
bundletool build-bundle \
  --overwrite \
  --modules="$TMP_DIR/base.zip" \
  --output="$APK_OUT_DIR/foxandhounds.aab" \
  "${METADATA_ARGS[@]}"

# Cleanup temporary files
rm -rf "$TMP_DIR"

echo "==> Successfully created AAB: $APK_OUT_DIR/foxandhounds.aab"
