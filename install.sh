#!/bin/sh
# Install goodissues CLI on macOS/Linux.
set -e

VERSION="${GOODISSUES_VERSION:-latest}"
INSTALL_DIR="${GOODISSUES_INSTALL_DIR:-$HOME/.local/bin}"
REPO_URL="${GOODISSUES_BASE_URL:-https://github.com/agoodway/goodissues_cli}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)  ARCH="amd64" ;;
    aarch64|arm64)  ARCH="arm64" ;;
    *)  echo "Error: unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
    darwin|linux) ;;
    msys*|mingw*|cygwin*)
        echo "Error: use install.ps1 for native Windows installs" >&2
        exit 1
        ;;
    *)  echo "Error: unsupported OS: $OS" >&2; exit 1 ;;
esac

BINARY="goodissues-${OS}-${ARCH}"

if [ "$VERSION" = "latest" ]; then
    URL="${REPO_URL}/releases/latest/download/${BINARY}"
else
    URL="${REPO_URL}/releases/download/v${VERSION}/${BINARY}"
fi

echo "Downloading goodissues for ${OS}/${ARCH}..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "${TMPDIR}/goodissues" "$URL"
elif command -v wget >/dev/null 2>&1; then
    wget -q -O "${TMPDIR}/goodissues" "$URL"
else
    echo "Error: curl or wget is required" >&2; exit 1
fi

chmod +x "${TMPDIR}/goodissues"

mkdir -p "$INSTALL_DIR"
mv "${TMPDIR}/goodissues" "${INSTALL_DIR}/goodissues"

echo "goodissues installed to ${INSTALL_DIR}/goodissues"

case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *) echo "NOTE: Add ${INSTALL_DIR} to your PATH:"; echo "  export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
esac

"${INSTALL_DIR}/goodissues" --version
