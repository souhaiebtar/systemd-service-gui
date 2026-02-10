#!/usr/bin/env bash
set -euo pipefail

APP_NAME="systemd-service-gui"
PROJECT_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_MANIFEST="$PROJECT_ROOT/Cargo.toml"
DESKTOP_FILE="$PROJECT_ROOT/packaging/appimage/${APP_NAME}.desktop"
ICON_FILE="$PROJECT_ROOT/packaging/appimage/${APP_NAME}.svg"
BUILD_ROOT="${BUILD_ROOT:-$PROJECT_ROOT/dist/appimage}"
TOOLS_DIR="$BUILD_ROOT/tools"
APPDIR="$BUILD_ROOT/${APP_NAME}.AppDir"
SKIP_TOOL_DOWNLOAD=0

usage() {
  cat <<'EOF'
Build a Linux AppImage for this project.

Usage:
  ./scripts/build-appimage.sh [--arch x86_64|aarch64] [--skip-tool-download]

Environment overrides:
  BUILD_ROOT=<path>   Output directory (default: dist/appimage)
  OUTPUT_FILE=<path>  Explicit AppImage output file
EOF
}

need_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

map_arch() {
  local input="$1"
  case "$input" in
    x86_64|amd64)
      echo "x86_64"
      ;;
    aarch64|arm64)
      echo "aarch64"
      ;;
    *)
      echo "Unsupported architecture: $input" >&2
      exit 1
      ;;
  esac
}

download_tool() {
  local url="$1"
  local output="$2"

  if [[ -x "$output" ]]; then
    return
  fi

  if command -v curl >/dev/null 2>&1; then
    curl -fL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -O "$output" "$url"
  else
    echo "Need curl or wget to download AppImage tooling." >&2
    exit 1
  fi

  chmod +x "$output"
}

run_appimage() {
  local bin="$1"
  shift
  "$bin" --appimage-extract-and-run "$@"
}

read_version() {
  sed -nE 's/^version = "(.*)"/\1/p' "$CARGO_MANIFEST" | head -n 1
}

ARCH_OVERRIDE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --arch)
      if [[ $# -lt 2 ]]; then
        echo "--arch requires a value." >&2
        exit 1
      fi
      ARCH_OVERRIDE="$2"
      shift 2
      ;;
    --skip-tool-download)
      SKIP_TOOL_DOWNLOAD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

need_cmd cargo
need_cmd sed
need_cmd install
need_cmd cp
need_cmd ln
need_cmd rm
need_cmd mkdir
need_cmd patchelf

if [[ ! -f "$DESKTOP_FILE" ]]; then
  echo "Missing desktop file: $DESKTOP_FILE" >&2
  exit 1
fi

if [[ ! -f "$ICON_FILE" ]]; then
  echo "Missing icon file: $ICON_FILE" >&2
  exit 1
fi

RAW_ARCH="${ARCH_OVERRIDE:-$(uname -m)}"
APPIMAGE_ARCH="$(map_arch "$RAW_ARCH")"
VERSION="$(read_version)"
if [[ -z "$VERSION" ]]; then
  echo "Failed to read crate version from Cargo.toml." >&2
  exit 1
fi

OUTPUT_FILE="${OUTPUT_FILE:-$BUILD_ROOT/${APP_NAME}-${VERSION}-${APPIMAGE_ARCH}.AppImage}"
LINUXDEPLOY="$TOOLS_DIR/linuxdeploy-${APPIMAGE_ARCH}.AppImage"
APPIMAGETOOL="$TOOLS_DIR/appimagetool-${APPIMAGE_ARCH}.AppImage"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${APPIMAGE_ARCH}.AppImage"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-${APPIMAGE_ARCH}.AppImage"

echo "==> Building release binary"
cargo build --release --manifest-path "$CARGO_MANIFEST"

echo "==> Preparing AppDir layout"
rm -rf "$APPDIR"
mkdir -p \
  "$APPDIR/usr/bin" \
  "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/icons/hicolor/scalable/apps" \
  "$TOOLS_DIR"
mkdir -p "$(dirname -- "$OUTPUT_FILE")"

install -m 755 "$PROJECT_ROOT/target/release/$APP_NAME" "$APPDIR/usr/bin/$APP_NAME"
install -m 644 "$DESKTOP_FILE" "$APPDIR/usr/share/applications/$APP_NAME.desktop"
install -m 644 "$ICON_FILE" "$APPDIR/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"
cp "$APPDIR/usr/share/applications/$APP_NAME.desktop" "$APPDIR/$APP_NAME.desktop"
ln -snf "usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg" "$APPDIR/.DirIcon"

if [[ "$SKIP_TOOL_DOWNLOAD" -eq 0 ]]; then
  echo "==> Downloading packaging tools (linuxdeploy + appimagetool)"
  download_tool "$LINUXDEPLOY_URL" "$LINUXDEPLOY"
  download_tool "$APPIMAGETOOL_URL" "$APPIMAGETOOL"
fi

if [[ ! -x "$LINUXDEPLOY" || ! -x "$APPIMAGETOOL" ]]; then
  echo "Tooling not found. Run without --skip-tool-download first." >&2
  exit 1
fi

echo "==> Bundling runtime dependencies with linuxdeploy"
run_appimage "$LINUXDEPLOY" \
  --appdir "$APPDIR" \
  -e "$APPDIR/usr/bin/$APP_NAME" \
  -d "$APPDIR/usr/share/applications/$APP_NAME.desktop" \
  -i "$APPDIR/usr/share/icons/hicolor/scalable/apps/$APP_NAME.svg"

echo "==> Building final AppImage"
rm -f "$OUTPUT_FILE"
ARCH="$APPIMAGE_ARCH" VERSION="$VERSION" run_appimage "$APPIMAGETOOL" "$APPDIR" "$OUTPUT_FILE"

echo "AppImage created:"
echo "  $OUTPUT_FILE"
