#!/usr/bin/env bash
set -euo pipefail

APP_ID="systemd-service-gui"
APP_TITLE="Systemd Service GUI"
REPO_OWNER="${REPO_OWNER:-souhaiebtar}"
REPO_NAME="${REPO_NAME:-systemd-service-gui}"

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
APPIMAGE_PATH="${APPIMAGE_PATH:-$INSTALL_DIR/$APP_ID.AppImage}"
DESKTOP_DIR="${DESKTOP_DIR:-$HOME/.local/share/applications}"
ICON_DIR="${ICON_DIR:-$HOME/.local/share/icons/hicolor/scalable/apps}"
DESKTOP_FILE="$DESKTOP_DIR/$APP_ID.desktop"
ICON_FILE="$ICON_DIR/$APP_ID.svg"

API_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"

need_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

detect_arch_pattern() {
  case "$(uname -m)" in
    x86_64|amd64)
      echo "x86_64|amd64"
      ;;
    aarch64|arm64)
      echo "aarch64|arm64"
      ;;
    *)
      uname -m
      ;;
  esac
}

extract_appimage_urls() {
  sed -n 's/.*"browser_download_url":[[:space:]]*"\([^"]*\.AppImage\)".*/\1/p'
}

pick_asset_url() {
  local arch_pattern="$1"
  local urls="$2"
  local selected

  selected="$(printf '%s\n' "$urls" | grep -Ei "$arch_pattern" | head -n 1 || true)"
  if [[ -n "$selected" ]]; then
    printf '%s\n' "$selected"
    return
  fi

  selected="$(printf '%s\n' "$urls" | head -n 1 || true)"
  if [[ -n "$selected" ]]; then
    printf '%s\n' "$selected"
    return
  fi

  return 1
}

write_icon() {
  cat >"$ICON_FILE" <<'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#12263A"/>
      <stop offset="100%" stop-color="#2E4A62"/>
    </linearGradient>
  </defs>
  <rect width="512" height="512" rx="96" fill="url(#bg)"/>
  <rect x="104" y="124" width="304" height="264" rx="28" fill="#F4F7FA"/>
  <rect x="136" y="168" width="220" height="36" rx="10" fill="#2E4A62"/>
  <rect x="136" y="228" width="180" height="36" rx="10" fill="#3A6A8A"/>
  <rect x="136" y="288" width="248" height="36" rx="10" fill="#4A85AB"/>
  <circle cx="388" cy="246" r="28" fill="#6AA84F"/>
  <path d="M375 246l10 10 18-20" stroke="#FFFFFF" stroke-width="8" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
EOF
}

write_desktop_file() {
  cat >"$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_TITLE
Comment=Systemd Service Manager
Exec=$APPIMAGE_PATH
Icon=$APP_ID
Terminal=false
Categories=System;Utility;
Keywords=systemd;service;manager;
StartupNotify=true
EOF
}

main() {
  need_cmd curl
  need_cmd grep
  need_cmd sed
  need_cmd mktemp
  need_cmd chmod
  need_cmd mv
  need_cmd mkdir
  need_cmd uname

  local arch_pattern
  local release_json
  local urls
  local asset_url
  local tmp_file

  arch_pattern="$(detect_arch_pattern)"

  echo "Fetching latest release from $REPO_OWNER/$REPO_NAME..."
  release_json="$(curl -fsSL "$API_URL")"
  urls="$(printf '%s\n' "$release_json" | extract_appimage_urls)"

  if [[ -z "$urls" ]]; then
    echo "No AppImage assets found in the latest release." >&2
    exit 1
  fi

  asset_url="$(pick_asset_url "$arch_pattern" "$urls" || true)"
  if [[ -z "$asset_url" ]]; then
    echo "Could not select an AppImage asset for architecture pattern: $arch_pattern" >&2
    exit 1
  fi

  mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR" "$ICON_DIR"
  tmp_file="$(mktemp "${TMPDIR:-/tmp}/${APP_ID}.XXXXXX.AppImage")"

  echo "Downloading AppImage:"
  echo "  $asset_url"
  curl -fL "$asset_url" -o "$tmp_file"
  mv "$tmp_file" "$APPIMAGE_PATH"
  chmod +x "$APPIMAGE_PATH"

  write_icon
  write_desktop_file

  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true
  fi

  echo
  echo "Installed $APP_TITLE"
  echo "AppImage: $APPIMAGE_PATH"
  echo "Desktop entry: $DESKTOP_FILE"
  echo "Icon: $ICON_FILE"
  echo
  echo "You can launch it from your app menu or run:"
  echo "  $APPIMAGE_PATH"
}

main "$@"

