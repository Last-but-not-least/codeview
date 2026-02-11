#!/bin/sh
# Install script for codeview
# Usage: curl -fsSL https://raw.githubusercontent.com/Last-but-not-least/codeview/main/install.sh | sh

set -eu

REPO="Last-but-not-least/codeview"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

get_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) echo "x86_64-unknown-linux-musl" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64|amd64) echo "x86_64-apple-darwin" ;;
        aarch64|arm64) echo "aarch64-apple-darwin" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *) echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac
}

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
}

main() {
  target="$(get_target)"
  version="${VERSION:-$(get_latest_version)}"

  echo "Installing codeview ${version} for ${target}..."

  url="https://github.com/${REPO}/releases/download/${version}/codeview-${version}-${target}.tar.gz"
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  curl -fsSL "$url" | tar xz -C "$tmpdir"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmpdir/codeview" "$INSTALL_DIR/codeview"
  else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "$tmpdir/codeview" "$INSTALL_DIR/codeview"
  fi

  chmod +x "$INSTALL_DIR/codeview"
  echo "Installed codeview to ${INSTALL_DIR}/codeview"
}

main
